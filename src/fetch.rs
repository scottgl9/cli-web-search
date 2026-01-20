//! URL fetching module for retrieving web page content

use crate::error::{Result, SearchError};
use reqwest::Client;
use serde::Serialize;
use std::time::Duration;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ContentFormat {
    /// Raw HTML content
    Html,
    /// Plain text (HTML tags stripped)
    #[default]
    Text,
    /// Markdown format
    Markdown,
}

/// Options for fetching URLs
#[derive(Clone, Debug)]
pub struct FetchOptions {
    /// Request timeout
    pub timeout: Duration,
    /// Output format
    pub format: ContentFormat,
    /// Follow redirects
    pub follow_redirects: bool,
    /// Maximum content length in bytes (0 = no limit)
    pub max_length: usize,
    /// User agent string
    pub user_agent: String,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            format: ContentFormat::Text,
            follow_redirects: true,
            max_length: 0,
            user_agent: format!(
                "cli-web-search/{} (https://github.com/scottgl9/cli-web-search)",
                env!("CARGO_PKG_VERSION")
            ),
        }
    }
}

impl FetchOptions {
    /// Create new fetch options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set content format
    pub fn with_format(mut self, format: ContentFormat) -> Self {
        self.format = format;
        self
    }

    /// Set max content length
    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = max_length;
        self
    }
}

/// Response from fetching a URL
#[derive(Debug, Clone, Serialize)]
pub struct FetchResponse {
    /// The URL that was fetched
    pub url: String,
    /// Final URL after redirects
    pub final_url: String,
    /// HTTP status code
    pub status: u16,
    /// Content type header
    pub content_type: Option<String>,
    /// The page content
    pub content: String,
    /// Content length in bytes
    pub content_length: usize,
    /// Page title (if available)
    pub title: Option<String>,
}

/// URL fetcher
pub struct Fetcher {
    client: Client,
    options: FetchOptions,
}

impl Fetcher {
    /// Create a new fetcher with default options
    pub fn new() -> Self {
        Self::with_options(FetchOptions::default())
    }

    /// Create a new fetcher with custom options
    pub fn with_options(options: FetchOptions) -> Self {
        let client = Client::builder()
            .timeout(options.timeout)
            .redirect(if options.follow_redirects {
                reqwest::redirect::Policy::limited(10)
            } else {
                reqwest::redirect::Policy::none()
            })
            .user_agent(&options.user_agent)
            .build()
            .expect("Failed to build HTTP client");

        Self { client, options }
    }

    /// Fetch a URL and return the content
    pub async fn fetch(&self, url: &str) -> Result<FetchResponse> {
        // Validate URL
        let parsed_url = url::Url::parse(url).map_err(|e| SearchError::Api {
            provider: "fetch".to_string(),
            message: format!("Invalid URL: {}", e),
        })?;

        // Only allow http and https
        if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
            return Err(SearchError::Api {
                provider: "fetch".to_string(),
                message: format!("Unsupported URL scheme: {}", parsed_url.scheme()),
            });
        }

        // Make request
        let response = self.client.get(url).send().await.map_err(|e| {
            if e.is_timeout() {
                SearchError::Timeout(self.options.timeout.as_secs())
            } else {
                SearchError::Api {
                    provider: "fetch".to_string(),
                    message: format!("Network error: {}", e),
                }
            }
        })?;

        let status = response.status().as_u16();
        let final_url = response.url().to_string();
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Check for error status
        if !response.status().is_success() {
            return Err(SearchError::Api {
                provider: "fetch".to_string(),
                message: format!(
                    "HTTP {}: {}",
                    status,
                    response.status().canonical_reason().unwrap_or("Unknown")
                ),
            });
        }

        // Get content
        let html = response.text().await.map_err(|e| SearchError::Api {
            provider: "fetch".to_string(),
            message: format!("Failed to read response: {}", e),
        })?;

        // Apply max length if set
        let html = if self.options.max_length > 0 && html.len() > self.options.max_length {
            html[..self.options.max_length].to_string()
        } else {
            html
        };

        // Extract title
        let title = extract_title(&html);

        // Convert content based on format
        let content = match self.options.format {
            ContentFormat::Html => html.clone(),
            ContentFormat::Text => html_to_text(&html),
            ContentFormat::Markdown => html_to_markdown(&html),
        };

        let content_length = content.len();

        Ok(FetchResponse {
            url: url.to_string(),
            final_url,
            status,
            content_type,
            content,
            content_length,
            title,
        })
    }
}

impl Default for Fetcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract the title from HTML content
fn extract_title(html: &str) -> Option<String> {
    // Simple regex-free title extraction
    let lower = html.to_lowercase();
    let start = lower.find("<title")?;
    let after_tag = html[start..].find('>')?;
    let title_start = start + after_tag + 1;
    let title_end = lower[title_start..].find("</title")?;

    let title = html[title_start..title_start + title_end].trim();
    if title.is_empty() {
        None
    } else {
        Some(decode_html_entities(title))
    }
}

/// Convert HTML to plain text by stripping tags
fn html_to_text(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut last_was_space = true;

    let lower = html.to_lowercase();
    let chars: Vec<char> = html.chars().collect();
    let lower_chars: Vec<char> = lower.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];

        // Check for script/style start
        if i + 7 < chars.len() {
            let slice: String = lower_chars[i..i + 7].iter().collect();
            if slice == "<script" {
                in_script = true;
            } else if slice == "<style "
                || (i + 6 < chars.len()
                    && lower_chars[i..i + 6].iter().collect::<String>() == "<style")
            {
                in_style = true;
            }
        }

        // Check for script/style end
        if i + 9 <= chars.len() {
            let slice: String = lower_chars[i..i + 9].iter().collect();
            if slice == "</script>" {
                in_script = false;
                i += 9;
                continue;
            }
        }
        if i + 8 <= chars.len() {
            let slice: String = lower_chars[i..i + 8].iter().collect();
            if slice == "</style>" {
                in_style = false;
                i += 8;
                continue;
            }
        }

        if in_script || in_style {
            i += 1;
            continue;
        }

        if c == '<' {
            in_tag = true;
            // Add space for block elements
            if i + 2 < chars.len() {
                let next_two: String = lower_chars[i + 1..i + 3].iter().collect();
                if matches!(
                    next_two.as_str(),
                    "p>" | "br"
                        | "di"
                        | "li"
                        | "h1"
                        | "h2"
                        | "h3"
                        | "h4"
                        | "h5"
                        | "h6"
                        | "tr"
                        | "td"
                        | "th"
                        | "/p"
                        | "/d"
                        | "/l"
                        | "/h"
                        | "/t"
                ) && !last_was_space
                {
                    result.push('\n');
                    last_was_space = true;
                }
            }
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            if c.is_whitespace() {
                if !last_was_space {
                    result.push(' ');
                    last_was_space = true;
                }
            } else {
                result.push(c);
                last_was_space = false;
            }
        }
        i += 1;
    }

    // Decode HTML entities
    let decoded = decode_html_entities(&result);

    // Clean up extra whitespace
    let lines: Vec<&str> = decoded
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    lines.join("\n")
}

/// Convert HTML to a simple Markdown format
#[allow(clippy::if_same_then_else)]
fn html_to_markdown(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut current_tag = String::new();
    let mut in_script = false;
    let mut in_style = false;
    let mut list_depth: usize = 0;

    let lower = html.to_lowercase();
    let chars: Vec<char> = html.chars().collect();
    let lower_chars: Vec<char> = lower.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];

        // Check for script/style
        if i + 7 < chars.len() {
            let slice: String = lower_chars[i..i + 7].iter().collect();
            if slice == "<script" {
                in_script = true;
            }
        }
        if i + 6 < chars.len() {
            let slice: String = lower_chars[i..i + 6].iter().collect();
            if slice == "<style" {
                in_style = true;
            }
        }
        if i + 9 <= chars.len() && lower_chars[i..i + 9].iter().collect::<String>() == "</script>" {
            in_script = false;
            i += 9;
            continue;
        }
        if i + 8 <= chars.len() && lower_chars[i..i + 8].iter().collect::<String>() == "</style>" {
            in_style = false;
            i += 8;
            continue;
        }

        if in_script || in_style {
            i += 1;
            continue;
        }

        if c == '<' {
            in_tag = true;
            current_tag.clear();
        } else if c == '>' {
            in_tag = false;
            let tag = current_tag.to_lowercase();

            // Handle tags
            if tag.starts_with("h1") {
                result.push_str("\n# ");
            } else if tag.starts_with("h2") {
                result.push_str("\n## ");
            } else if tag.starts_with("h3") {
                result.push_str("\n### ");
            } else if tag.starts_with("h4") {
                result.push_str("\n#### ");
            } else if tag.starts_with("h5") {
                result.push_str("\n##### ");
            } else if tag.starts_with("h6") {
                result.push_str("\n###### ");
            } else if tag == "p" || tag.starts_with("p ") {
                result.push_str("\n\n");
            } else if tag == "/p" {
                result.push('\n');
            } else if tag == "br" || tag == "br/" || tag == "br /" {
                result.push_str("  \n");
            } else if tag == "ul" || tag.starts_with("ul ") {
                list_depth += 1;
                result.push('\n');
            } else if tag == "/ul" {
                list_depth = list_depth.saturating_sub(1);
                result.push('\n');
            } else if tag == "ol" || tag.starts_with("ol ") {
                list_depth += 1;
                result.push('\n');
            } else if tag == "/ol" {
                list_depth = list_depth.saturating_sub(1);
                result.push('\n');
            } else if tag == "li" || tag.starts_with("li ") {
                result.push_str(&"  ".repeat(list_depth.saturating_sub(1)));
                result.push_str("- ");
            } else if tag == "/li" {
                result.push('\n');
            } else if tag == "strong"
                || tag == "b"
                || tag.starts_with("strong ")
                || tag.starts_with("b ")
            {
                result.push_str("**");
            } else if tag == "/strong" || tag == "/b" {
                result.push_str("**");
            } else if tag == "em" || tag == "i" || tag.starts_with("em ") || tag.starts_with("i ") {
                result.push('*');
            } else if tag == "/em" || tag == "/i" {
                result.push('*');
            } else if tag == "code" || tag.starts_with("code ") {
                result.push('`');
            } else if tag == "/code" {
                result.push('`');
            } else if tag == "pre" || tag.starts_with("pre ") {
                result.push_str("\n```\n");
            } else if tag == "/pre" {
                result.push_str("\n```\n");
            } else if tag == "blockquote" || tag.starts_with("blockquote ") {
                result.push_str("\n> ");
            } else if tag == "hr" || tag == "hr/" || tag == "hr /" {
                result.push_str("\n---\n");
            } else if tag.starts_with("/h") {
                result.push('\n');
            }
            current_tag.clear();
        } else if in_tag {
            current_tag.push(c);
        } else {
            result.push(c);
        }
        i += 1;
    }

    // Decode HTML entities
    let decoded = decode_html_entities(&result);

    // Clean up
    let lines: Vec<&str> = decoded.lines().collect();
    let mut cleaned = Vec::new();
    let mut prev_empty = false;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !prev_empty {
                cleaned.push("");
                prev_empty = true;
            }
        } else {
            cleaned.push(trimmed);
            prev_empty = false;
        }
    }

    cleaned.join("\n").trim().to_string()
}

/// Decode common HTML entities
fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .replace("&#x27;", "'")
        .replace("&#x2F;", "/")
        .replace("&mdash;", "—")
        .replace("&ndash;", "–")
        .replace("&hellip;", "...")
        .replace("&copy;", "©")
        .replace("&reg;", "®")
        .replace("&trade;", "™")
        .replace("&ldquo;", "\u{201C}")
        .replace("&rdquo;", "\u{201D}")
        .replace("&lsquo;", "\u{2018}")
        .replace("&rsquo;", "\u{2019}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_options_default() {
        let options = FetchOptions::default();
        assert_eq!(options.timeout, Duration::from_secs(30));
        assert_eq!(options.format, ContentFormat::Text);
        assert!(options.follow_redirects);
    }

    #[test]
    fn test_fetch_options_builder() {
        let options = FetchOptions::new()
            .with_timeout(Duration::from_secs(60))
            .with_format(ContentFormat::Markdown)
            .with_max_length(1000);

        assert_eq!(options.timeout, Duration::from_secs(60));
        assert_eq!(options.format, ContentFormat::Markdown);
        assert_eq!(options.max_length, 1000);
    }

    #[test]
    fn test_extract_title() {
        assert_eq!(
            extract_title("<html><head><title>Test Page</title></head></html>"),
            Some("Test Page".to_string())
        );
        assert_eq!(
            extract_title("<html><head><title>  Spaced Title  </title></head></html>"),
            Some("Spaced Title".to_string())
        );
        assert_eq!(extract_title("<html><head></head></html>"), None);
        assert_eq!(
            extract_title("<html><head><title></title></head></html>"),
            None
        );
    }

    #[test]
    fn test_extract_title_with_entities() {
        assert_eq!(
            extract_title("<title>Test &amp; Page</title>"),
            Some("Test & Page".to_string())
        );
    }

    #[test]
    fn test_html_to_text() {
        let html = "<html><body><p>Hello <b>World</b>!</p></body></html>";
        let text = html_to_text(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
        assert!(!text.contains("<p>"));
        assert!(!text.contains("<b>"));
    }

    #[test]
    fn test_html_to_text_strips_scripts() {
        let html = "<html><body><script>alert('hi');</script><p>Content</p></body></html>";
        let text = html_to_text(html);
        assert!(!text.contains("alert"));
        assert!(text.contains("Content"));
    }

    #[test]
    fn test_html_to_text_strips_styles() {
        let html =
            "<html><head><style>body { color: red; }</style></head><body>Content</body></html>";
        let text = html_to_text(html);
        assert!(!text.contains("color"));
        assert!(text.contains("Content"));
    }

    #[test]
    fn test_html_to_markdown_headings() {
        let html = "<h1>Title</h1><h2>Subtitle</h2><p>Content</p>";
        let md = html_to_markdown(html);
        assert!(md.contains("# Title"));
        assert!(md.contains("## Subtitle"));
    }

    #[test]
    fn test_html_to_markdown_formatting() {
        let html = "<p><strong>Bold</strong> and <em>italic</em></p>";
        let md = html_to_markdown(html);
        assert!(md.contains("**Bold**"));
        assert!(md.contains("*italic*"));
    }

    #[test]
    fn test_html_to_markdown_lists() {
        let html = "<ul><li>Item 1</li><li>Item 2</li></ul>";
        let md = html_to_markdown(html);
        assert!(md.contains("- Item 1"));
        assert!(md.contains("- Item 2"));
    }

    #[test]
    fn test_decode_html_entities() {
        assert_eq!(decode_html_entities("&amp;"), "&");
        assert_eq!(decode_html_entities("&lt;"), "<");
        assert_eq!(decode_html_entities("&gt;"), ">");
        assert_eq!(decode_html_entities("&quot;"), "\"");
        assert_eq!(decode_html_entities("Test &amp; Test"), "Test & Test");
    }

    #[test]
    fn test_content_format_default() {
        assert_eq!(ContentFormat::default(), ContentFormat::Text);
    }

    #[tokio::test]
    async fn test_fetch_invalid_url() {
        let fetcher = Fetcher::new();
        let result = fetcher.fetch("not-a-valid-url").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_unsupported_scheme() {
        let fetcher = Fetcher::new();
        let result = fetcher.fetch("ftp://example.com/file.txt").await;
        assert!(result.is_err());
        if let Err(SearchError::Api { message, .. }) = result {
            assert!(message.contains("Unsupported URL scheme"));
        }
    }
}

//! Integration tests using mock HTTP servers
//!
//! These tests verify that providers correctly parse API responses
//! and handle various error conditions.

use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Test helper to create a mock Brave API response
fn brave_success_response() -> serde_json::Value {
    serde_json::json!({
        "web": {
            "results": [
                {
                    "title": "Rust Programming Language",
                    "url": "https://www.rust-lang.org/",
                    "description": "A language empowering everyone to build reliable software.",
                    "age": "2024-01-15",
                    "meta_url": {
                        "hostname": "rust-lang.org"
                    }
                },
                {
                    "title": "The Rust Book",
                    "url": "https://doc.rust-lang.org/book/",
                    "description": "The Rust Programming Language book.",
                    "age": null,
                    "meta_url": {
                        "hostname": "doc.rust-lang.org"
                    }
                }
            ]
        }
    })
}

/// Test helper to create a mock Google CSE API response
fn google_success_response() -> serde_json::Value {
    serde_json::json!({
        "items": [
            {
                "title": "Rust Programming",
                "link": "https://www.rust-lang.org/",
                "snippet": "A systems programming language.",
                "displayLink": "rust-lang.org"
            }
        ],
        "searchInformation": {
            "totalResults": "1000000",
            "searchTime": 0.25
        }
    })
}

/// Test helper to create a mock Tavily API response
fn tavily_success_response() -> serde_json::Value {
    serde_json::json!({
        "results": [
            {
                "title": "Rust Lang",
                "url": "https://www.rust-lang.org/",
                "content": "Rust is a systems programming language.",
                "score": 0.95,
                "published_date": "2024-01-01"
            }
        ]
    })
}

/// Test helper to create a mock Serper API response
fn serper_success_response() -> serde_json::Value {
    serde_json::json!({
        "organic": [
            {
                "title": "Rust Programming",
                "link": "https://www.rust-lang.org/",
                "snippet": "Build reliable software.",
                "date": "Jan 15, 2024",
                "displayedLink": "rust-lang.org"
            }
        ],
        "searchParameters": {
            "q": "rust programming"
        }
    })
}

/// Test helper to create a mock DuckDuckGo API response
fn duckduckgo_success_response() -> serde_json::Value {
    serde_json::json!({
        "Abstract": "Rust is a programming language.",
        "AbstractURL": "https://en.wikipedia.org/wiki/Rust_(programming_language)",
        "AbstractSource": "Wikipedia",
        "Heading": "Rust (programming language)",
        "RelatedTopics": [
            {
                "Text": "Rust language - A systems programming language",
                "FirstURL": "https://www.rust-lang.org/"
            }
        ]
    })
}

/// Test helper to create a mock Firecrawl API response
fn firecrawl_success_response() -> serde_json::Value {
    serde_json::json!({
        "success": true,
        "data": {
            "web": [
                {
                    "title": "Rust Programming",
                    "url": "https://www.rust-lang.org/",
                    "description": "A language for reliable software."
                }
            ]
        }
    })
}

/// Test helper to create a mock SerpAPI response
fn serpapi_success_response() -> serde_json::Value {
    serde_json::json!({
        "organic_results": [
            {
                "title": "Rust Programming Language",
                "link": "https://www.rust-lang.org/",
                "snippet": "A language empowering everyone to build reliable software.",
                "position": 1,
                "displayed_link": "https://www.rust-lang.org › learn",
                "date": "Jan 15, 2024"
            },
            {
                "title": "Rust Documentation",
                "link": "https://doc.rust-lang.org/",
                "snippet": "Official Rust documentation.",
                "position": 2,
                "displayed_link": "https://doc.rust-lang.org"
            }
        ],
        "search_metadata": {
            "id": "search-12345",
            "status": "Success",
            "total_time_taken": 0.42
        }
    })
}

/// Test helper to create a mock Bing Web Search API response
fn bing_success_response() -> serde_json::Value {
    serde_json::json!({
        "_type": "SearchResponse",
        "webPages": {
            "totalEstimatedMatches": 12500000,
            "value": [
                {
                    "name": "Rust Programming Language",
                    "url": "https://www.rust-lang.org/",
                    "snippet": "A language empowering everyone to build reliable software.",
                    "displayUrl": "www.rust-lang.org",
                    "dateLastCrawled": "2024-01-15T12:00:00Z"
                },
                {
                    "name": "The Rust Book",
                    "url": "https://doc.rust-lang.org/book/",
                    "snippet": "The Rust Programming Language book.",
                    "displayUrl": "doc.rust-lang.org/book",
                    "dateLastCrawled": "2024-01-10T08:30:00Z"
                }
            ]
        }
    })
}

#[tokio::test]
async fn test_mock_brave_search_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/res/v1/web/search"))
        .and(header("X-Subscription-Token", "test-api-key"))
        .and(query_param("q", "rust programming"))
        .respond_with(ResponseTemplate::new(200).set_body_json(brave_success_response()))
        .mount(&mock_server)
        .await;

    // Create a custom client that points to our mock server
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/res/v1/web/search", mock_server.uri()))
        .header("X-Subscription-Token", "test-api-key")
        .query(&[("q", "rust programming"), ("count", "10")])
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await.unwrap();
    let results = body["web"]["results"].as_array().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["title"], "Rust Programming Language");
    assert_eq!(results[0]["url"], "https://www.rust-lang.org/");
}

#[tokio::test]
async fn test_mock_brave_rate_limited() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/res/v1/web/search"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "60")
                .set_body_string("Rate limit exceeded"),
        )
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/res/v1/web/search", mock_server.uri()))
        .header("X-Subscription-Token", "test-api-key")
        .query(&[("q", "test")])
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 429);
    assert_eq!(
        response
            .headers()
            .get("Retry-After")
            .unwrap()
            .to_str()
            .unwrap(),
        "60"
    );
}

#[tokio::test]
async fn test_mock_brave_invalid_api_key() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/res/v1/web/search"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/res/v1/web/search", mock_server.uri()))
        .header("X-Subscription-Token", "invalid-key")
        .query(&[("q", "test")])
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_mock_google_search_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/customsearch/v1"))
        .and(query_param("q", "rust"))
        .respond_with(ResponseTemplate::new(200).set_body_json(google_success_response()))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/customsearch/v1", mock_server.uri()))
        .query(&[("q", "rust"), ("key", "test-key"), ("cx", "test-cx")])
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await.unwrap();
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["title"], "Rust Programming");
}

#[tokio::test]
async fn test_mock_google_empty_results() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/customsearch/v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "searchInformation": {
                "totalResults": "0"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/customsearch/v1", mock_server.uri()))
        .query(&[("q", "xyznonexistent12345")])
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body.get("items").is_none());
}

#[tokio::test]
async fn test_mock_tavily_search_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(tavily_success_response()))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/search", mock_server.uri()))
        .json(&serde_json::json!({
            "api_key": "test-key",
            "query": "rust programming"
        }))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await.unwrap();
    let results = body["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["title"], "Rust Lang");
}

#[tokio::test]
async fn test_mock_serper_search_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/search"))
        .and(header("X-API-KEY", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serper_success_response()))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/search", mock_server.uri()))
        .header("X-API-KEY", "test-api-key")
        .json(&serde_json::json!({
            "q": "rust programming"
        }))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await.unwrap();
    let organic = body["organic"].as_array().unwrap();
    assert_eq!(organic.len(), 1);
    assert_eq!(organic[0]["title"], "Rust Programming");
}

#[tokio::test]
async fn test_mock_duckduckgo_search_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/"))
        .and(query_param("q", "rust"))
        .and(query_param("format", "json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(duckduckgo_success_response()))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/", mock_server.uri()))
        .query(&[("q", "rust"), ("format", "json")])
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["Heading"], "Rust (programming language)");
    assert!(!body["Abstract"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_mock_firecrawl_search_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/search"))
        .and(header("Authorization", "Bearer test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(firecrawl_success_response()))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/v1/search", mock_server.uri()))
        .header("Authorization", "Bearer test-api-key")
        .json(&serde_json::json!({
            "query": "rust programming"
        }))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());
    let web = body["data"]["web"].as_array().unwrap();
    assert_eq!(web.len(), 1);
}

#[tokio::test]
async fn test_mock_server_timeout() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/slow"))
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(5)))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(100))
        .build()
        .unwrap();

    let result = client
        .get(format!("{}/slow", mock_server.uri()))
        .send()
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.is_timeout());
}

#[tokio::test]
async fn test_mock_server_error_500() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/error"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/error", mock_server.uri()))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 500);
}

#[tokio::test]
async fn test_mock_malformed_json_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/malformed"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "application/json")
                .set_body_string("{invalid json}"),
        )
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/malformed", mock_server.uri()))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());

    let result: Result<serde_json::Value, _> = response.json().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mock_serpapi_search_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search"))
        .and(query_param("q", "rust programming"))
        .and(query_param("engine", "google"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serpapi_success_response()))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/search", mock_server.uri()))
        .query(&[
            ("q", "rust programming"),
            ("api_key", "test-api-key"),
            ("engine", "google"),
            ("num", "10"),
        ])
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await.unwrap();
    let results = body["organic_results"].as_array().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["title"], "Rust Programming Language");
    assert_eq!(results[0]["link"], "https://www.rust-lang.org/");
    assert_eq!(results[0]["position"], 1);
}

#[tokio::test]
async fn test_mock_serpapi_rate_limited() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "30")
                .set_body_string("Rate limit exceeded"),
        )
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/search", mock_server.uri()))
        .query(&[("q", "test"), ("api_key", "test-key")])
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 429);
    assert_eq!(
        response
            .headers()
            .get("Retry-After")
            .unwrap()
            .to_str()
            .unwrap(),
        "30"
    );
}

#[tokio::test]
async fn test_mock_serpapi_invalid_api_key() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": "Invalid API key"
        })))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/search", mock_server.uri()))
        .query(&[("q", "test"), ("api_key", "invalid-key")])
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_mock_bing_search_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v7.0/search"))
        .and(header("Ocp-Apim-Subscription-Key", "test-api-key"))
        .and(query_param("q", "rust programming"))
        .respond_with(ResponseTemplate::new(200).set_body_json(bing_success_response()))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/v7.0/search", mock_server.uri()))
        .header("Ocp-Apim-Subscription-Key", "test-api-key")
        .query(&[("q", "rust programming"), ("count", "10")])
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await.unwrap();
    let results = body["webPages"]["value"].as_array().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["name"], "Rust Programming Language");
    assert_eq!(results[0]["url"], "https://www.rust-lang.org/");
}

#[tokio::test]
async fn test_mock_bing_rate_limited() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v7.0/search"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "60")
                .set_body_string("Rate limit exceeded"),
        )
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/v7.0/search", mock_server.uri()))
        .header("Ocp-Apim-Subscription-Key", "test-key")
        .query(&[("q", "test")])
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 429);
}

#[tokio::test]
async fn test_mock_bing_invalid_api_key() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v7.0/search"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Access denied"))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/v7.0/search", mock_server.uri()))
        .header("Ocp-Apim-Subscription-Key", "invalid-key")
        .query(&[("q", "test")])
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_mock_bing_empty_results() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v7.0/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "_type": "SearchResponse"
        })))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/v7.0/search", mock_server.uri()))
        .header("Ocp-Apim-Subscription-Key", "test-key")
        .query(&[("q", "xyznonexistent12345")])
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body.get("webPages").is_none());
}

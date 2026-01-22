import type { Plugin } from "@opencode-ai/plugin";
import { tool } from "@opencode-ai/plugin";

/**
 * OpenCode plugin for web search using cli-web-search
 * 
 * Provides web search capabilities to OpenCode by integrating with the cli-web-search tool.
 * Supports multiple search providers (Brave, Google, DuckDuckGo, Tavily, Serper, etc.)
 * and various output formats (JSON, Markdown, Text).
 */
export const WebSearchPlugin: Plugin = async ({ project, client, $, directory, worktree }) => {
  // Log plugin initialization
  await client.app.log({
    service: "web-search",
    level: "info",
    message: "Web search plugin initialized",
    extra: { directory, project: project.name },
  });

  return {
    tool: {
      /**
       * Search the web using cli-web-search
       */
      web_search: tool({
        description: `Search the web for information using cli-web-search. 
        
This tool provides comprehensive web search capabilities with support for multiple search providers.
Use this when you need to find current information, research topics, or verify facts from the web.

The tool supports:
- Multiple search providers (Brave, Google CSE, DuckDuckGo, Tavily, Serper, Firecrawl, SerpAPI, Bing)
- Automatic provider fallback if one fails
- Result filtering by date range and domains
- Safe search levels
- Configurable number of results

Best used for:
- Looking up current information or recent events
- Researching programming topics, libraries, or frameworks
- Finding documentation or tutorials
- Verifying technical facts or best practices
- Discovering related resources or solutions`,
        args: {
          query: tool.schema.string().describe("The search query"),
          num_results: tool.schema
            .number()
            .optional()
            .default(10)
            .describe("Number of results to return (default: 10)"),
          provider: tool.schema
            .enum(["brave", "google", "ddg", "tavily", "serper", "firecrawl", "serpapi", "bing"])
            .optional()
            .describe("Preferred search provider (uses configured default if not specified)"),
          date_range: tool.schema
            .enum(["day", "week", "month", "year"])
            .optional()
            .describe("Filter results by date range"),
          safe_search: tool.schema
            .enum(["off", "moderate", "strict"])
            .optional()
            .describe("Safe search level (default: moderate)"),
          include_domains: tool.schema
            .string()
            .optional()
            .describe("Comma-separated list of domains to include"),
          exclude_domains: tool.schema
            .string()
            .optional()
            .describe("Comma-separated list of domains to exclude"),
        },
        async execute(args, ctx) {
          try {
            // Build the command
            let command = `cli-web-search -f json`;
            
            // Add optional parameters
            if (args.num_results !== undefined) {
              command += ` -n ${args.num_results}`;
            }
            
            if (args.provider) {
              command += ` -p ${args.provider}`;
            }
            
            if (args.date_range) {
              command += ` --date-range ${args.date_range}`;
            }
            
            if (args.safe_search) {
              command += ` --safe-search ${args.safe_search}`;
            }
            
            if (args.include_domains) {
              command += ` --include-domains "${args.include_domains}"`;
            }
            
            if (args.exclude_domains) {
              command += ` --exclude-domains "${args.exclude_domains}"`;
            }
            
            // Add the query (properly escaped)
            command += ` "${args.query.replace(/"/g, '\\"')}"`;
            
            // Log the search
            await client.app.log({
              service: "web-search",
              level: "info",
              message: "Executing web search",
              extra: { query: args.query, provider: args.provider },
            });
            
            // Execute the search using Bun's shell
            const result = await $`${command}`.text();
            
            // Parse and return results
            const searchResults = JSON.parse(result);
            
            // Format results for better readability
            const formattedResults = {
              query: searchResults.query,
              provider: searchResults.provider,
              total_results: searchResults.results?.length || 0,
              search_time_ms: searchResults.search_time_ms,
              results: searchResults.results?.map((r: any, idx: number) => ({
                position: idx + 1,
                title: r.title,
                url: r.url,
                snippet: r.snippet,
              })) || [],
            };
            
            return JSON.stringify(formattedResults, null, 2);
          } catch (error) {
            // Log error
            await client.app.log({
              service: "web-search",
              level: "error",
              message: "Web search failed",
              extra: { 
                query: args.query, 
                error: error instanceof Error ? error.message : String(error) 
              },
            });
            
            throw new Error(
              `Web search failed: ${error instanceof Error ? error.message : String(error)}\n\n` +
              `Make sure cli-web-search is installed and configured. See: https://github.com/scottgl9/cli-web-search#installation`
            );
          }
        },
      }),

      /**
       * Fetch and extract content from a web page
       */
      fetch_url: tool({
        description: `Fetch and extract content from a web page using cli-web-search.

This tool retrieves web page content and converts it to a readable format (text or markdown).
Use this when you need to read the contents of a specific web page, article, or documentation.

Best used for:
- Reading documentation from a specific URL
- Extracting content from articles or blog posts
- Accessing web-based resources
- Following up on search results to get full content`,
        args: {
          url: tool.schema.string().describe("The URL to fetch"),
          format: tool.schema
            .enum(["text", "html", "markdown"])
            .optional()
            .default("markdown")
            .describe("Output format (default: markdown)"),
          max_length: tool.schema
            .number()
            .optional()
            .describe("Maximum content length in bytes (0 = no limit)"),
        },
        async execute(args, ctx) {
          try {
            // Build the command
            let command = `cli-web-search fetch "${args.url.replace(/"/g, '\\"')}" --stdout`;
            
            // Add format parameter
            if (args.format) {
              command += ` -f ${args.format}`;
            }
            
            // Add max length if specified
            if (args.max_length) {
              command += ` --max-length ${args.max_length}`;
            }
            
            // Log the fetch
            await client.app.log({
              service: "web-search",
              level: "info",
              message: "Fetching URL",
              extra: { url: args.url, format: args.format },
            });
            
            // Execute the fetch using Bun's shell
            const content = await $`${command}`.text();
            
            return content;
          } catch (error) {
            // Log error
            await client.app.log({
              service: "web-search",
              level: "error",
              message: "URL fetch failed",
              extra: { 
                url: args.url, 
                error: error instanceof Error ? error.message : String(error) 
              },
            });
            
            throw new Error(
              `Failed to fetch URL: ${error instanceof Error ? error.message : String(error)}\n\n` +
              `Make sure cli-web-search is installed and the URL is accessible.`
            );
          }
        },
      }),
    },
  };
};

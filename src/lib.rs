use anyhow::Result;
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock},
    schemars, tool, tool_handler, tool_router,
};
use serde::Deserialize;
use tokio::process::Command;

pub mod cli;

#[derive(Clone)]
pub struct Inventory {}

impl Inventory {
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchRequest {
    /// Case-insensitive substring to match against binary names and manual pages.
    pub query: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DescribeRequest {
    /// Exact, case-sensitive binary name.
    pub name: String,
}

#[tool_handler(
    name = "inventory-mcp",
    instructions = "Use this server to discover local command-line tools and read their installed manual pages before choosing a command or its flags. Use search with a task keyword or binary name to find candidates (default limit: 20). Use describe with an exact binary name to read its full manual. Use list when you need the complete inventory. Results reflect the server's startup PATH and installed manuals. Entries without a readable manual are omitted. This server does not execute discovered commands."
)]
impl rmcp::ServerHandler for Inventory {}

#[tool_router]
impl Inventory {
    #[tool(
        description = "Search binary names and manual pages by case-insensitive substring; defaults to 20 results sorted by name"
    )]
    pub async fn search(&self, Parameters(request): Parameters<SearchRequest>) -> CallToolResult {
        let output = match Command::new("man")
            .args(["-k", &request.query.to_lowercase()])
            .output()
            .await
        {
            Ok(o) => o,
            Err(err) => return CallToolResult::error(vec![ContentBlock::text(err.to_string())]),
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            CallToolResult::success(
                stdout
                    .split("\n")
                    .filter(|s| !s.is_empty())
                    .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
                    .map(ContentBlock::text)
                    .collect(),
            )
        } else {
            CallToolResult::error(vec![ContentBlock::text(stderr)])
        }
    }

    #[tool(description = "Get the full manual page for an exact, case-sensitive binary name")]
    pub async fn describe(
        &self,
        Parameters(request): Parameters<DescribeRequest>,
    ) -> CallToolResult {
        let output = match Command::new("man").args(vec![&request.name]).output().await {
            Ok(o) => o,
            Err(err) => return CallToolResult::error(vec![ContentBlock::text(err.to_string())]),
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            CallToolResult::success(vec![ContentBlock::text(stdout.to_string())])
        } else {
            CallToolResult::error(vec![ContentBlock::text(stderr)])
        }
    }
}

#[cfg(test)]
mod tests;

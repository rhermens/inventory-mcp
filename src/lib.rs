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
    /// Keyword to pass to `man -k`. The keyword is lowercased first.
    pub query: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DescribeRequest {
    /// Manual page name to pass to `man`, such as `git`.
    pub name: String,
}

#[tool_handler(
    name = "inventory-mcp",
    instructions = "Use this server to find local command-line tools and read their manual pages before choosing a command or its flags. Use search with a task keyword or command name to find candidate manual pages; it runs `man -k`. Use describe with a manual page name to read the full page; it runs `man`. Results come from the manual pages installed in the server's environment. This server does not execute the commands it finds."
)]
impl rmcp::ServerHandler for Inventory {}

#[tool_router]
impl Inventory {
    #[tool(
        description = "Search manual page names and short descriptions with `man -k`; returns one result per matching page"
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

    #[tool(description = "Get the full manual page for a name with `man`")]
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

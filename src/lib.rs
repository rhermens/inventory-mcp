use std::{env, ffi::OsString, fs::DirEntry};

use anyhow::Result;
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock},
    schemars, tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use tokio::{io::stdout, process::Command};

#[derive(Clone)]
pub struct Inventory {}

impl Inventory {
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }
}

#[derive(Debug, Eq, PartialEq, Serialize, Hash, Clone)]
pub struct Binary {
    pub binary: OsString,
    pub description: String,
}

impl Binary {
    async fn try_new(value: DirEntry) -> Result<Self> {
        let output = Command::new("man")
            .args([value.file_name().to_str().expect("No file name")])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            Ok(Binary {
                binary: value.file_name(),
                description: stdout.to_string(),
            })
        } else {
            Err(anyhow::Error::msg(stderr.to_owned().to_string()))
        }
    }
}

impl From<&Binary> for ContentBlock {
    fn from(val: &Binary) -> Self {
        ContentBlock::text(format!(
            "{} - {}",
            val.binary
                .to_owned()
                .into_string()
                .expect("Failed to read binary string"),
            val.description
        ))
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SearchRequest {
    /// Case-insensitive substring to match against binary names and manual pages.
    query: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct DescribeRequest {
    /// Exact, case-sensitive binary name.
    name: String,
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
    async fn search(&self, Parameters(request): Parameters<SearchRequest>) -> CallToolResult {
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
    async fn describe(&self, Parameters(request): Parameters<DescribeRequest>) -> CallToolResult {
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

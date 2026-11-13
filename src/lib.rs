use std::{
    collections::HashSet,
    env,
    ffi::OsString,
    fs::{self, DirEntry},
    path::PathBuf,
};

use anyhow::Result;
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock},
    schemars, tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use tokio::{process::Command, task::JoinSet};

#[derive(Clone)]
pub struct Inventory {
    pub binaries: HashSet<Binary>,
}

impl Inventory {
    async fn scan_dir_binaries(directory: PathBuf) -> Result<HashSet<Binary>> {
        let entries = tokio::task::spawn_blocking(move || {
            let Ok(entries) = fs::read_dir(directory) else {
                return Vec::new();
            };

            entries
                .flatten()
                .filter(|entry| fs::metadata(entry.path()).is_ok_and(|metadata| metadata.is_file()))
                .collect::<Vec<_>>()
        })
        .await?;

        let mut tasks = JoinSet::new();
        let mut binaries = HashSet::new();

        for entry in entries {
            tasks.spawn(async move { Binary::try_new(entry).await });

            if tasks.len() >= 16
                && let Some(result) = tasks.join_next().await
                && let Ok(binary) = result?
            {
                binaries.insert(binary);
            }
        }

        while let Some(result) = tasks.join_next().await {
            if let Ok(binary) = result? {
                binaries.insert(binary);
            }
        }

        Ok(binaries)
    }

    async fn scan_binaries() -> Result<HashSet<Binary>> {
        let path = env::var_os("PATH").unwrap_or_default();
        let mut binaries = HashSet::new();

        for directory in env::split_paths(&path) {
            let directory = if directory.as_os_str().is_empty() {
                ".".into()
            } else {
                directory
            };

            binaries.extend(Self::scan_dir_binaries(directory).await?);
        }

        Ok(binaries)
    }

    pub async fn new() -> Result<Self> {
        Ok(Self {
            binaries: Self::scan_binaries().await?,
        })
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
    /// Maximum number of results. Defaults to 20; zero returns no results.
    limit: Option<usize>,
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
    #[tool(description = "List available user binaries")]
    fn list(&self) -> CallToolResult {
        CallToolResult::success(self.binaries.iter().map(|e| e.into()).collect())
    }

    #[tool(
        description = "Search binary names and manual pages by case-insensitive substring; defaults to 20 results sorted by name"
    )]
    fn search(&self, Parameters(request): Parameters<SearchRequest>) -> CallToolResult {
        let query = request.query.to_lowercase();
        let mut matches: Vec<_> = self
            .binaries
            .iter()
            .filter(|binary| {
                binary
                    .binary
                    .to_string_lossy()
                    .to_lowercase()
                    .contains(&query)
                    || binary.description.to_lowercase().contains(&query)
            })
            .collect();
        matches.sort_unstable_by(|left, right| {
            left.binary
                .cmp(&right.binary)
                .then_with(|| left.description.cmp(&right.description))
        });

        CallToolResult::success(
            matches
                .into_iter()
                .take(request.limit.unwrap_or(20))
                .map(ContentBlock::from)
                .collect(),
        )
    }

    #[tool(description = "Get the full manual page for an exact, case-sensitive binary name")]
    fn describe(&self, Parameters(request): Parameters<DescribeRequest>) -> CallToolResult {
        let name = OsString::from(&request.name);
        match self
            .binaries
            .iter()
            .filter(|binary| binary.binary == name)
            .min_by(|left, right| left.description.cmp(&right.description))
        {
            Some(binary) => CallToolResult::success(vec![binary.into()]),
            None => CallToolResult::error(vec![ContentBlock::text(format!(
                "Binary not found: {}",
                request.name
            ))]),
        }
    }
}

#[cfg(test)]
mod tests;

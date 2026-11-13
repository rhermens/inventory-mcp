use std::{
    collections::HashSet,
    env,
    ffi::OsString,
    fs::{self, DirEntry},
    path::PathBuf,
};

use anyhow::Result;
use rmcp::{
    model::{CallToolResult, ContentBlock},
    tool, tool_router,
};
use serde::Serialize;
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

#[tool_router(server_handler)]
impl Inventory {
    #[tool(description = "List available user binaries")]
    fn list(&self) -> CallToolResult {
        CallToolResult::success(self.binaries.iter().map(|e| e.into()).collect())
    }
}

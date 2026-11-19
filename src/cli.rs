use clap::{Parser, Subcommand};

/// MCP server that wraps man for searching and reading manual pages.
///
/// inventory-mcp exposes two MCP tools. The search tool runs `man -k`. The
/// describe tool runs `man`. Results come from the manual pages that man
/// finds in the server's environment.
///
/// Set PATH to choose the man binary. Set MANPATH to choose the manual pages.
#[derive(Debug, Parser)]
#[command(name = "inventory-mcp", version, about, long_about)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run the MCP server over stdio.
    ///
    /// Reads MCP requests from stdin and writes responses to stdout. The server
    /// provides the search and describe tools.
    Serve,
    /// Run the search tool once and print the result.
    ///
    /// Runs `man -k` with the lowercased SEARCH and prints the MCP tool result
    /// in debug format. Use this command to test the search tool.
    Query {
        /// Keyword to pass to `man -k`.
        search: String,
    },
}

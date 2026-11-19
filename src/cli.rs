use clap::{Parser, Subcommand};

/// MCP server that lists binaries on $PATH alongside their manual pages.
///
/// inventory-mcp scans the directories in $PATH at startup and indexes each
/// binary that has a readable manual page. Use the serve command to expose the
/// inventory to an MCP client over stdio. Use the query command to search the
/// inventory from the terminal.
///
/// Set PATH and MANPATH to control which binaries and manual pages are found.
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
    /// provides the search, describe, and list tools. It never executes the
    /// binaries it discovers.
    Serve,
    /// Search binary names and manual pages from the terminal.
    ///
    /// Matches SEARCH as a case-insensitive substring against binary names and
    /// manual page contents. Prints the matching entries sorted by name.
    Query {
        /// Text to match against binary names and manual pages.
        ///
        /// An empty string matches every entry.
        search: String,
    },
}

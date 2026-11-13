# inventory-mcp

A [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server that lists files found on `$PATH` alongside their manual pages.

## Requirements

- `man` available on the server's `$PATH`.
- Installed manual pages for the commands you want to discover.

The server uses the environment inherited from its MCP client. Set `PATH` and `MANPATH` in that environment to control discovery and manual lookup.

## Build

```bash
cargo build --release --locked
```

The executable is `target/release/inventory-mcp`.

## Configure an MCP client

Add a stdio server to your client's MCP configuration. Replace the executable path with your local absolute path:

```json
{
  "mcpServers": {
    "inventory": {
      "command": "/absolute/path/to/inventory-mcp/target/release/inventory-mcp",
      "args": [],
      "type": "stdio"
    }
  }
}
```

For development, run the server through Cargo, as shown in [`.pi/mcp.json`](.pi/mcp.json):

```json
{
  "mcpServers": {
    "inventory": {
      "command": "cargo",
      "args": [
        "run",
        "--locked",
        "--manifest-path",
        "/absolute/path/to/inventory-mcp/Cargo.toml"
      ],
      "type": "stdio"
    }
  }
}
```

The server communicates through stdin and stdout. It is not an interactive CLI.

During MCP initialization, the server provides agent instructions for discovering tools with `search`, reading manuals with `describe`, and requesting the complete inventory with `list`.

## Tools

### `list`

Takes no arguments. Returns one MCP text content block per discovered entry:

```text
<file name> - <full manual page text>
```

The description contains the full output of `man <file name>`, not only the `NAME` or `SYNOPSIS` section. Results have no guaranteed order.

### `search`

Finds case-insensitive substring matches in file names and full manual page text:

```json
{ "query": "version control", "limit": 5 }
```

- `query` is required. An empty query matches every entry.
- `limit` is optional and defaults to 20. Zero returns no results.
- Matches are sorted by file name, then manual page text, before applying the limit.
- Each result uses the same full-text format as `list`. No matches returns an empty successful result.

### `describe`

Returns the full manual page for an exact, case-sensitive file name:

```json
{ "name": "git" }
```

An unknown name returns an MCP tool error with `Binary not found: <name>`. If several entries share a name, the first manual page in lexical order is returned.

Both tools use the startup inventory and perform no additional manual lookups.

## Discovery behavior

- Scans `$PATH` once at startup, before accepting MCP requests.
- Treats empty `$PATH` entries as the current directory.
- Visits directories sequentially and follows file symlinks.
- Runs up to 16 manual lookups concurrently.
- Skips unreadable directories and entries whose manual lookup fails.
- Deduplicates entries by file name and manual page text.

The scanner currently checks whether an entry is a regular file, not whether it has executable permission. Manual lookup uses the file name, not its full path.

Manual lookups have no timeout. Startup waits for all lookups to finish. Restart the server to refresh the inventory.

## Nix

Build the default package:

```bash
nix build
```

The executable is `result/bin/inventory-mcp`. The package uses `man` and manual pages from the runtime environment.

Enter the development shell:

```bash
nix develop
```

## Development checks

```bash
cargo fmt --check
cargo test --locked
cargo clippy --all-targets --locked
```

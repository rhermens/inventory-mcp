# inventory-mcp

A [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server that wraps `man`. Agents use it to search for local commands and read their manual pages.

## Requirements

- `man` available on the server's `$PATH`.
- Installed manual pages for the commands you want to find.
- A populated `man -k` (apropos) database, for the `search` tool.

The server uses the environment inherited from its MCP client. Set `PATH` to choose the `man` binary. Set `MANPATH` to choose the manual pages.

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

The server communicates through stdin and stdout.

During MCP initialization, the server provides agent instructions for finding commands with `search` and reading manual pages with `describe`.

## Tools

Each tool call runs `man` as a child process. The server keeps no state between calls.

### `search`

Runs `man -k <query>` with the query lowercased:

```json
{ "query": "version control" }
```

- `query` is required.
- `man -k` matches manual page names and short descriptions, not full page text.
- Each output line becomes one MCP text content block, with runs of whitespace collapsed to one space.
- If `man -k` exits with a nonzero status, the tool returns an error with its stderr. Most `man` implementations exit nonzero when nothing matches.

### `describe`

Runs `man <name>`:

```json
{ "name": "git" }
```

- Returns the full `man` output as one MCP text content block.
- If `man` exits with a nonzero status, the tool returns an error with its stderr.

## CLI

```bash
inventory-mcp serve           # Run the MCP server over stdio.
inventory-mcp query <SEARCH>  # Run the search tool once and print the result.
```

`query` prints the MCP tool result in Rust debug format. Use it to test `search` from the terminal.

The build script generates man pages for the CLI with `clap_mangen`. They are written to Cargo's `OUT_DIR`.

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

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

## Tools

### `list`

Takes no arguments. Returns one MCP text content block per discovered entry:

```text
<file name> - <full manual page text>
```

The description contains the full output of `man <file name>`, not only the `NAME` or `SYNOPSIS` section. Results have no guaranteed order.

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

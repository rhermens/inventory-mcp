use anyhow::Ok;
use clap::Parser;
use inventory_mcp::{Inventory, SearchRequest, cli::CliArgs};
use rmcp::{ServiceExt, handler::server::wrapper::Parameters, transport::stdio};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    let service = Inventory::new().await?;

    match args.command {
        inventory_mcp::cli::Command::Serve => {
            let server = service.serve(stdio()).await?;
            server.waiting().await?;
        }
        inventory_mcp::cli::Command::Query { search } => {
            let res = service
                .search(Parameters(SearchRequest { query: search }))
                .await;
            println!("{:?}", res);
        }
    }
    Ok(())
}

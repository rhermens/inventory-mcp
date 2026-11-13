use inventory_mcp::Inventory;
use rmcp::{ServiceExt, transport::stdio};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service = Inventory::new().await?.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

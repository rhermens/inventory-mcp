use inventory_mcp::Inventory;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service = Inventory::new().await?;

    dbg!(service.binaries);

    Ok(())
}

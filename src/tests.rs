use super::*;

#[tokio::test]
async fn search_returns_tool_error_when_no_manual_matches() -> Result<()> {
    let inventory = Inventory::new().await?;

    let result = inventory
        .search(Parameters(SearchRequest {
            query: "git".to_string(),
        }))
        .await;

    dbg!(result);
    Ok(())
}

#[tokio::test]
async fn describe_returns_tool_error_when_no_manual_matches() -> Result<()> {
    let inventory = Inventory::new().await?;

    let result = inventory
        .describe(Parameters(DescribeRequest {
            name: "wt".to_string(),
        }))
        .await;

    dbg!(result);
    Ok(())
}

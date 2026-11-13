use super::*;

fn inventory(entries: &[(&str, &str)]) -> Inventory {
    Inventory {
        binaries: entries
            .iter()
            .map(|(name, description)| Binary {
                binary: OsString::from(name),
                description: description.to_string(),
            })
            .collect(),
    }
}

fn text(result: &CallToolResult) -> Vec<&str> {
    result
        .content
        .iter()
        .map(|content| content.as_text().unwrap().text.as_str())
        .collect()
}

fn search(inventory: &Inventory, query: &str, limit: Option<usize>) -> CallToolResult {
    inventory.search(Parameters(SearchRequest {
        query: query.to_string(),
        limit,
    }))
}

fn describe(inventory: &Inventory, name: &str) -> CallToolResult {
    inventory.describe(Parameters(DescribeRequest {
        name: name.to_string(),
    }))
}

#[test]
fn server_info_guides_agents_to_inventory_tools() {
    let info = rmcp::ServerHandler::get_info(&inventory(&[]));
    let instructions = info.instructions.expect("Agent instructions are required");

    assert_eq!(info.server_info.name, "inventory-mcp");
    assert!(info.capabilities.tools.is_some());
    assert!(instructions.contains("search"));
    assert!(instructions.contains("describe"));
    assert!(instructions.contains("list"));
    assert!(instructions.contains("manual pages"));
    assert!(instructions.contains("startup"));
}

#[test]
fn search_matches_names_case_insensitively() {
    let inventory = inventory(&[("Git", "Version control"), ("cargo", "Rust packages")]);
    let result = search(&inventory, "gIT", None);

    assert_eq!(result.is_error, Some(false));
    assert_eq!(text(&result), ["Git - Version control"]);
}

#[test]
fn search_matches_manual_text_case_insensitively() {
    let inventory = inventory(&[("git", "Distributed VERSION control"), ("cargo", "Rust")]);
    let result = search(&inventory, "version", None);

    assert_eq!(text(&result), ["git - Distributed VERSION control"]);
}

#[test]
fn search_sorts_matches_before_applying_limit() {
    let inventory = inventory(&[("zeta", "tool"), ("beta", "tool"), ("alpha", "tool")]);
    let result = search(&inventory, "tool", Some(2));

    assert_eq!(text(&result), ["alpha - tool", "beta - tool"]);
}

#[test]
fn search_defaults_to_twenty_results() {
    let inventory = Inventory {
        binaries: (0..25)
            .map(|index| Binary {
                binary: OsString::from(format!("tool-{index:02}")),
                description: "Manual".to_string(),
            })
            .collect(),
    };
    let result = search(&inventory, "tool", None);

    assert_eq!(result.content.len(), 20);
    assert_eq!(text(&result)[0], "tool-00 - Manual");
    assert_eq!(text(&result)[19], "tool-19 - Manual");
}

#[test]
fn search_accepts_zero_limit() {
    let inventory = inventory(&[("git", "Version control")]);
    let result = search(&inventory, "git", Some(0));

    assert!(result.content.is_empty());
    assert_eq!(result.is_error, Some(false));
}

#[test]
fn search_returns_all_matches_when_limit_exceeds_inventory() {
    let inventory = inventory(&[("git", "Version control")]);
    let result = search(&inventory, "git", Some(usize::MAX));

    assert_eq!(text(&result), ["git - Version control"]);
}

#[test]
fn search_without_matches_returns_empty_success() {
    let inventory = inventory(&[("git", "Version control")]);
    let result = search(&inventory, "missing", None);

    assert!(result.content.is_empty());
    assert_eq!(result.is_error, Some(false));
}

#[test]
fn empty_query_matches_all_entries() {
    let inventory = inventory(&[("git", "Version control"), ("cargo", "Rust")]);
    let result = search(&inventory, "", None);

    assert_eq!(text(&result), ["cargo - Rust", "git - Version control"]);
}

#[test]
fn empty_inventory_returns_empty_search_results() {
    let result = search(&inventory(&[]), "git", None);

    assert!(result.content.is_empty());
    assert_eq!(result.is_error, Some(false));
}

#[test]
fn describe_returns_full_manual_for_exact_name() {
    let manual = "NAME\n    git - version control\nSYNOPSIS\n    git [options]\n";
    let inventory = inventory(&[("git", manual), ("git-lfs", "Large file storage")]);
    let result = describe(&inventory, "git");

    assert_eq!(result.is_error, Some(false));
    assert_eq!(text(&result), [format!("git - {manual}")]);
}

#[test]
fn describe_unknown_name_returns_tool_error() {
    let result = describe(&inventory(&[("git", "Version control")]), "missing");

    assert_eq!(result.is_error, Some(true));
    assert_eq!(text(&result), ["Binary not found: missing"]);
}

#[test]
fn describe_requires_case_sensitive_name() {
    let result = describe(&inventory(&[("git", "Version control")]), "GIT");

    assert_eq!(result.is_error, Some(true));
}

#[test]
fn duplicate_names_have_deterministic_results() {
    let inventory = inventory(&[("git", "Z manual"), ("git", "A manual")]);

    assert_eq!(
        text(&search(&inventory, "git", None)),
        ["git - A manual", "git - Z manual"]
    );
    assert_eq!(text(&describe(&inventory, "git")), ["git - A manual"]);
}

#[test]
fn search_parameters_require_query_but_not_limit() {
    let request: SearchRequest = rmcp::serde_json::from_str(r#"{"query":"git"}"#).unwrap();

    assert_eq!(request.query, "git");
    assert_eq!(request.limit, None);
    assert!(rmcp::serde_json::from_str::<SearchRequest>(r#"{}"#).is_err());
    assert!(rmcp::serde_json::from_str::<SearchRequest>(r#"{"query":"git","limit":-1}"#).is_err());
}

#[test]
fn describe_parameters_require_name() {
    assert!(rmcp::serde_json::from_str::<DescribeRequest>(r#"{}"#).is_err());
}

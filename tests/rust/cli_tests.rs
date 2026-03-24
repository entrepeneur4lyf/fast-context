use serde_json::Value;
use std::process::Command;

fn run_cli(args: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_fast-context"))
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("CLI should execute");

    assert!(
        output.status.success(),
        "CLI failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("CLI should emit JSON")
}

#[test]
fn test_cli_analyze_json_output() {
    let value = run_cli(&["--format", "json", "analyze", "."]);
    assert!(value["project_path"]
        .as_str()
        .unwrap()
        .ends_with(env!("CARGO_MANIFEST_DIR")));
    assert!(value["file_count"].as_u64().unwrap() > 0);
    assert!(value["symbol_count"].as_u64().unwrap() > 0);
    assert!(value["relationship_count"].as_u64().unwrap() > 0);
}

#[test]
fn test_cli_stats_languages_json_output() {
    let value = run_cli(&["--format", "json", "stats", "languages", "."]);
    let languages = value["languages"].as_array().unwrap();
    assert!(!languages.is_empty());
    assert!(languages
        .iter()
        .any(|language| language["language"].as_str() == Some("rust")));
}

#[test]
fn test_cli_mcp_init_stdout_json_output() {
    let value = run_cli(&["--format", "json", "mcp", "init", "--stdout"]);
    assert_eq!(value["written"].as_bool(), Some(false));
    assert!(value["server_command"].as_str().is_some());
    assert_eq!(
        value["preview"]["mcpServers"]["fast-context"]["command"].as_str(),
        value["server_command"].as_str()
    );
}

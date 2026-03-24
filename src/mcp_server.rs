use crate::core::{CoreAnalysisSummary, CoreAnalyzer, CoreAnalyzerOptions};
use crate::validation::{validate_directory_path, validate_file_path};
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars,
    schemars::JsonSchema,
    tool, tool_handler, tool_router, ErrorData, ServerHandler,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AnalyzerRequestOptions {
    pub project_path: String,
    pub languages: Option<Vec<String>>,
    pub ignore_patterns: Option<Vec<String>>,
    pub max_files: Option<usize>,
    pub parallel_processing: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AnalyzeCodebaseRequest {
    #[serde(flatten)]
    pub options: AnalyzerRequestOptions,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct FindSymbolsByKindRequest {
    #[serde(flatten)]
    pub options: AnalyzerRequestOptions,
    pub symbol_kind: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct FindSymbolsInFileRequest {
    #[serde(flatten)]
    pub options: AnalyzerRequestOptions,
    pub file_path: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct FindDependenciesRequest {
    #[serde(flatten)]
    pub options: AnalyzerRequestOptions,
    pub symbol_name: String,
}

#[derive(Debug, Clone, Serialize)]
struct SkippedFileView {
    file_path: String,
    stage: String,
    reason: String,
}

#[derive(Debug, Clone, Default)]
pub struct FastContextMcpServer {
    tool_router: ToolRouter<Self>,
}

impl FastContextMcpServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    fn build_analyzer(&self, options: &AnalyzerRequestOptions) -> Result<CoreAnalyzer, ErrorData> {
        validate_directory_path(&options.project_path)
            .map_err(|err| ErrorData::invalid_params(err.to_string(), None))?;

        Ok(CoreAnalyzer::with_options(
            options.project_path.clone(),
            options.languages.clone(),
            options.ignore_patterns.clone(),
            CoreAnalyzerOptions {
                max_files: options.max_files,
                parallel_processing: options.parallel_processing.unwrap_or(true),
            },
        ))
    }

    fn validate_project_file(&self, project_path: &str, file_path: &str) -> Result<(), ErrorData> {
        let resolved = PathBuf::from(project_path).join(file_path);
        validate_file_path(&resolved.to_string_lossy())
            .map_err(|err| ErrorData::invalid_params(err.to_string(), None))?;
        Ok(())
    }

    fn serialize_summary(
        &self,
        project_path: &str,
        summary: CoreAnalysisSummary,
    ) -> Result<String, ErrorData> {
        let skipped_files = summary
            .skipped_files
            .into_iter()
            .map(|entry| SkippedFileView {
                file_path: entry.file_path,
                stage: entry.stage,
                reason: entry.reason,
            })
            .collect::<Vec<_>>();

        serde_json::to_string_pretty(&json!({
            "projectPath": project_path,
            "fileCount": summary.file_count,
            "symbolCount": summary.symbol_count,
            "relationshipCount": summary.relationships.len(),
            "languages": summary.languages,
            "durationMs": summary.duration_ms,
            "skippedFileCount": skipped_files.len(),
            "skippedFiles": skipped_files
        }))
        .map_err(|err| ErrorData::internal_error(err.to_string(), None))
    }
}

#[tool_router]
impl FastContextMcpServer {
    #[tool(
        name = "analyze_codebase",
        description = "Analyze a project directory and return summary metrics."
    )]
    async fn analyze_codebase(
        &self,
        Parameters(request): Parameters<AnalyzeCodebaseRequest>,
    ) -> Result<String, ErrorData> {
        let analyzer = self.build_analyzer(&request.options)?;
        let summary = analyzer
            .analyze_summary()
            .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;
        self.serialize_summary(&request.options.project_path, summary)
    }

    #[tool(
        name = "find_symbols_by_kind",
        description = "Return symbol names for a given symbol kind in the project."
    )]
    async fn find_symbols_by_kind(
        &self,
        Parameters(request): Parameters<FindSymbolsByKindRequest>,
    ) -> Result<String, ErrorData> {
        let analyzer = self.build_analyzer(&request.options)?;
        let symbols = analyzer
            .find_symbols_by_kind(request.symbol_kind.clone())
            .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;

        serde_json::to_string_pretty(&json!({
            "projectPath": request.options.project_path,
            "symbolKind": request.symbol_kind,
            "count": symbols.len(),
            "symbols": symbols
        }))
        .map_err(|err| ErrorData::internal_error(err.to_string(), None))
    }

    #[tool(
        name = "find_symbols_in_file",
        description = "Return symbol names for a specific file in the project."
    )]
    async fn find_symbols_in_file(
        &self,
        Parameters(request): Parameters<FindSymbolsInFileRequest>,
    ) -> Result<String, ErrorData> {
        self.validate_project_file(&request.options.project_path, &request.file_path)?;

        let analyzer = self.build_analyzer(&request.options)?;
        let symbols = analyzer
            .find_symbols_in_file(request.file_path.clone())
            .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;

        serde_json::to_string_pretty(&json!({
            "projectPath": request.options.project_path,
            "filePath": request.file_path,
            "count": symbols.len(),
            "symbols": symbols
        }))
        .map_err(|err| ErrorData::internal_error(err.to_string(), None))
    }

    #[tool(
        name = "find_dependencies",
        description = "Return dependency symbol names for a specific symbol."
    )]
    async fn find_dependencies(
        &self,
        Parameters(request): Parameters<FindDependenciesRequest>,
    ) -> Result<String, ErrorData> {
        let analyzer = self.build_analyzer(&request.options)?;
        let dependencies = analyzer
            .find_dependencies(request.symbol_name.clone())
            .map_err(|err| ErrorData::internal_error(err.to_string(), None))?;

        serde_json::to_string_pretty(&json!({
            "projectPath": request.options.project_path,
            "symbolName": request.symbol_name,
            "count": dependencies.len(),
            "dependencies": dependencies
        }))
        .map_err(|err| ErrorData::internal_error(err.to_string(), None))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for FastContextMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("Fast-Context MCP server for codebase analysis.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::{
        model::{CallToolRequestParams, ClientInfo},
        ClientHandler, ServiceExt,
    };
    use serde_json::Value;
    use std::fs;
    use tempfile::tempdir;

    #[derive(Default)]
    struct DummyClientHandler;

    impl ClientHandler for DummyClientHandler {
        fn get_info(&self) -> ClientInfo {
            ClientInfo::default()
        }
    }

    #[test]
    fn registers_expected_tools() {
        let tools = FastContextMcpServer::tool_router().list_all();
        let names = tools
            .iter()
            .map(|tool| tool.name.as_ref().to_string())
            .collect::<Vec<_>>();

        assert_eq!(
            names,
            vec![
                "analyze_codebase",
                "find_dependencies",
                "find_symbols_by_kind",
                "find_symbols_in_file",
            ]
        );
    }

    #[tokio::test]
    async fn analyze_codebase_returns_real_summary() {
        let temp_dir = tempdir().expect("create temp dir");
        let source_path = temp_dir.path().join("main.rs");
        fs::write(&source_path, "fn main() { println!(\"hi\"); }\n").expect("write source");

        let server = FastContextMcpServer::new();
        let result = server
            .analyze_codebase(Parameters(AnalyzeCodebaseRequest {
                options: AnalyzerRequestOptions {
                    project_path: temp_dir.path().to_string_lossy().to_string(),
                    languages: Some(vec!["rust".to_string()]),
                    ignore_patterns: None,
                    max_files: None,
                    parallel_processing: Some(false),
                },
            }))
            .await
            .expect("analyze codebase");

        let json: Value = serde_json::from_str(&result).expect("valid json");
        assert_eq!(json["fileCount"], 1);
        assert!(json["symbolCount"].as_u64().unwrap_or_default() >= 1);
        assert_eq!(json["skippedFileCount"], 0);
    }

    #[tokio::test]
    async fn rmcp_round_trip_exposes_tools_and_handles_calls() {
        let temp_dir = tempdir().expect("create temp dir");
        fs::write(
            temp_dir.path().join("lib.rs"),
            "pub fn helper() -> usize { 1 }\n",
        )
        .expect("write source");

        let (server_transport, client_transport) = tokio::io::duplex(8192);
        let server = FastContextMcpServer::new();

        let server_task = tokio::spawn(async move {
            server
                .serve(server_transport)
                .await
                .expect("serve server")
                .waiting()
                .await
                .expect("server waiting");
        });

        let client = DummyClientHandler
            .serve(client_transport)
            .await
            .expect("serve client");

        let tools = client.list_all_tools().await.expect("list tools");
        let tool_names = tools
            .iter()
            .map(|tool| tool.name.to_string())
            .collect::<Vec<_>>();
        assert!(tool_names.contains(&"analyze_codebase".to_string()));
        assert!(tool_names.contains(&"find_symbols_by_kind".to_string()));

        let result = client
            .call_tool(
                CallToolRequestParams::new("analyze_codebase").with_arguments(
                    json!({
                        "project_path": temp_dir.path().to_string_lossy().to_string(),
                        "languages": ["rust"],
                        "parallel_processing": false
                    })
                    .as_object()
                    .expect("json object")
                    .clone(),
                ),
            )
            .await
            .expect("call analyze_codebase");

        let result_text = result
            .content
            .first()
            .and_then(|content| content.raw.as_text())
            .map(|text| text.text.as_str())
            .expect("expected text content");

        let json: Value = serde_json::from_str(result_text).expect("valid json result");
        assert_eq!(json["fileCount"], 1);
        assert!(json["symbolCount"].as_u64().unwrap_or_default() >= 1);

        client.cancel().await.expect("cancel client");
        server_task.await.expect("join server task");
    }
}

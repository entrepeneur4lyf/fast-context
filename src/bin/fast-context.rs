use clap::{Parser, Subcommand, ValueEnum};
use fast_context::core::{CoreAnalysisSummary, CoreAnalyzer, CoreAnalyzerOptions};
use fast_context::parsers::LanguageId;
use fast_context::utils::{detect_language_id, merged_ignore_patterns, should_ignore_file};
use fast_context::validation::{validate_directory_path, validate_file_path};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use walkdir::WalkDir;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match &cli.command {
        Commands::Analyze { path } => {
            let project_root = resolve_project_root(path, cli.project_root.as_ref())?;
            let analyzer = build_analyzer(&project_root, &cli);
            let summary = analyzer.analyze_summary().map_err(|err| err.to_string())?;
            let output = AnalyzeOutput::from_summary(project_root, summary);
            print_output(&cli.output_format, cli.quiet, &output, render_analyze_text)
        }
        Commands::Symbols { command } => match command {
            SymbolCommands::ByKind { kind, path } => {
                let project_root = resolve_project_root(path, cli.project_root.as_ref())?;
                let analyzer = build_analyzer(&project_root, &cli);
                let symbols = analyzer
                    .find_symbols_by_kind(kind.clone())
                    .map_err(|err| err.to_string())?;
                let output = SymbolsByKindOutput {
                    project_path: display_path(&project_root),
                    symbol_kind: kind.clone(),
                    count: symbols.len(),
                    symbols,
                };
                print_output(
                    &cli.output_format,
                    cli.quiet,
                    &output,
                    render_symbols_by_kind_text,
                )
            }
            SymbolCommands::InFile { file, path } => {
                let project_root = resolve_project_root(path, cli.project_root.as_ref())?;
                let resolved_file = resolve_project_file(&project_root, file);
                validate_file_path(&display_path(&resolved_file)).map_err(|err| err.to_string())?;
                let analyzer = build_analyzer(&project_root, &cli);
                let symbols = analyzer
                    .find_symbols_in_file(display_path(&resolved_file))
                    .map_err(|err| err.to_string())?;
                let output = SymbolsInFileOutput {
                    project_path: display_path(&project_root),
                    file_path: file.to_string_lossy().to_string(),
                    count: symbols.len(),
                    symbols,
                };
                print_output(
                    &cli.output_format,
                    cli.quiet,
                    &output,
                    render_symbols_in_file_text,
                )
            }
        },
        Commands::Deps { symbol, path } => {
            let project_root = resolve_project_root(path, cli.project_root.as_ref())?;
            let analyzer = build_analyzer(&project_root, &cli);
            let dependencies = analyzer
                .find_dependencies(symbol.clone())
                .map_err(|err| err.to_string())?;
            let output = DependenciesOutput {
                project_path: display_path(&project_root),
                symbol_name: symbol.clone(),
                count: dependencies.len(),
                dependencies,
            };
            print_output(
                &cli.output_format,
                cli.quiet,
                &output,
                render_dependencies_text,
            )
        }
        Commands::Stats { command } => match command {
            StatsCommands::Languages { path } => {
                let project_root = resolve_project_root(path, cli.project_root.as_ref())?;
                let report = collect_stats(&project_root, &cli)?;
                let output = report.into_language_output(project_root);
                print_output(
                    &cli.output_format,
                    cli.quiet,
                    &output,
                    render_language_stats_text,
                )
            }
            StatsCommands::Loc { path } => {
                let project_root = resolve_project_root(path, cli.project_root.as_ref())?;
                let report = collect_stats(&project_root, &cli)?;
                let output = report.into_loc_output(project_root);
                print_output(
                    &cli.output_format,
                    cli.quiet,
                    &output,
                    render_loc_stats_text,
                )
            }
        },
        Commands::Mcp { command } => match command {
            McpCommands::Init {
                config_path,
                default_location,
                stdout,
                server_name,
            } => {
                let project_root = cli.project_root.clone().unwrap_or_else(|| {
                    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                });
                let output = run_mcp_init(
                    &project_root,
                    config_path.as_ref(),
                    *default_location,
                    *stdout,
                    server_name,
                )?;
                print_output(&cli.output_format, cli.quiet, &output, render_mcp_init_text)
            }
            McpCommands::Doctor { binary_path } => {
                let project_root = cli.project_root.clone().unwrap_or_else(|| {
                    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                });
                let output = run_mcp_doctor(&project_root, binary_path.as_ref());
                print_output(
                    &cli.output_format,
                    cli.quiet,
                    &output,
                    render_mcp_doctor_text,
                )
            }
        },
        Commands::Doctor => {
            let project_root = cli
                .project_root
                .clone()
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
            let output = run_doctor(&project_root);
            print_output(&cli.output_format, cli.quiet, &output, render_doctor_text)
        }
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "fast-context",
    about = "Codebase analysis and developer tooling"
)]
struct Cli {
    #[arg(long = "format", global = true, value_enum, default_value_t = OutputFormat::Text)]
    output_format: OutputFormat,
    #[arg(long, global = true)]
    quiet: bool,
    #[arg(long, global = true)]
    project_root: Option<PathBuf>,
    #[arg(long = "language", global = true)]
    languages: Vec<String>,
    #[arg(long = "ignore", global = true)]
    ignore_patterns: Vec<String>,
    #[arg(long = "max-files", global = true)]
    max_files: Option<usize>,
    #[arg(long = "serial", global = true)]
    serial: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
    Yaml,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Analyze {
        path: PathBuf,
    },
    Symbols {
        #[command(subcommand)]
        command: SymbolCommands,
    },
    Deps {
        symbol: String,
        path: PathBuf,
    },
    Stats {
        #[command(subcommand)]
        command: StatsCommands,
    },
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },
    Doctor,
}

#[derive(Debug, Subcommand)]
enum SymbolCommands {
    ByKind { kind: String, path: PathBuf },
    InFile { file: PathBuf, path: PathBuf },
}

#[derive(Debug, Subcommand)]
enum StatsCommands {
    Languages { path: PathBuf },
    Loc { path: PathBuf },
}

#[derive(Debug, Subcommand)]
enum McpCommands {
    Init {
        #[arg(long)]
        config_path: Option<PathBuf>,
        #[arg(long)]
        default_location: bool,
        #[arg(long)]
        stdout: bool,
        #[arg(long, default_value = "fast-context")]
        server_name: String,
    },
    Doctor {
        #[arg(long)]
        binary_path: Option<PathBuf>,
    },
}

#[derive(Debug, Serialize)]
struct AnalyzeOutput {
    project_path: String,
    file_count: u32,
    symbol_count: u32,
    relationship_count: usize,
    languages: Vec<String>,
    duration_ms: u32,
    skipped_file_count: usize,
    skipped_files: Vec<SkippedFileOutput>,
}

impl AnalyzeOutput {
    fn from_summary(project_path: PathBuf, summary: CoreAnalysisSummary) -> Self {
        Self {
            project_path: display_path(&project_path),
            file_count: summary.file_count,
            symbol_count: summary.symbol_count,
            relationship_count: summary.relationships.len(),
            languages: summary.languages,
            duration_ms: summary.duration_ms,
            skipped_file_count: summary.skipped_files.len(),
            skipped_files: summary
                .skipped_files
                .into_iter()
                .map(SkippedFileOutput::from)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
struct SkippedFileOutput {
    file_path: String,
    stage: String,
    reason: String,
}

impl From<fast_context::core::SkippedFileDiagnostic> for SkippedFileOutput {
    fn from(value: fast_context::core::SkippedFileDiagnostic) -> Self {
        Self {
            file_path: value.file_path,
            stage: value.stage,
            reason: value.reason,
        }
    }
}

#[derive(Debug, Serialize)]
struct SymbolsByKindOutput {
    project_path: String,
    symbol_kind: String,
    count: usize,
    symbols: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SymbolsInFileOutput {
    project_path: String,
    file_path: String,
    count: usize,
    symbols: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DependenciesOutput {
    project_path: String,
    symbol_name: String,
    count: usize,
    dependencies: Vec<String>,
}

#[derive(Debug, Serialize)]
struct LanguageStatsOutput {
    project_path: String,
    total_files: usize,
    total_lines: usize,
    languages: Vec<LanguageStat>,
}

#[derive(Debug, Serialize)]
struct LanguageStat {
    language: String,
    file_count: usize,
    line_count: usize,
    percentage: f64,
}

#[derive(Debug, Serialize)]
struct LocStatsOutput {
    project_path: String,
    total_lines: usize,
    code_lines: usize,
    comment_lines: usize,
    blank_lines: usize,
    languages: Vec<LocLanguageStat>,
}

#[derive(Debug, Serialize)]
struct LocLanguageStat {
    language: String,
    file_count: usize,
    total_lines: usize,
    code_lines: usize,
    comment_lines: usize,
    blank_lines: usize,
}

#[derive(Debug, Serialize)]
struct DoctorOutput {
    status: String,
    checks: Vec<DoctorCheck>,
    errors: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
struct McpInitOutput {
    config_path: Option<String>,
    server_command: String,
    arguments: Vec<String>,
    written: bool,
    preview: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct McpDoctorOutput {
    status: String,
    binary_path: Option<String>,
    server_command: String,
    arguments: Vec<String>,
    checks: Vec<DoctorCheck>,
    errors: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DoctorCheck {
    name: String,
    status: String,
    detail: String,
}

#[derive(Debug, Default)]
struct ProjectStatsReport {
    total_files: usize,
    total_lines: usize,
    code_lines: usize,
    comment_lines: usize,
    blank_lines: usize,
    languages: BTreeMap<String, LanguageAccumulator>,
}

#[derive(Debug, Default)]
struct LanguageAccumulator {
    file_count: usize,
    total_lines: usize,
    code_lines: usize,
    comment_lines: usize,
    blank_lines: usize,
}

impl ProjectStatsReport {
    fn record_file(&mut self, language: &str, loc: LineCount) {
        self.total_files += 1;
        self.total_lines += loc.total_lines;
        self.code_lines += loc.code_lines;
        self.comment_lines += loc.comment_lines;
        self.blank_lines += loc.blank_lines;

        let entry = self.languages.entry(language.to_string()).or_default();
        entry.file_count += 1;
        entry.total_lines += loc.total_lines;
        entry.code_lines += loc.code_lines;
        entry.comment_lines += loc.comment_lines;
        entry.blank_lines += loc.blank_lines;
    }

    fn into_language_output(self, project_path: PathBuf) -> LanguageStatsOutput {
        let total_lines = self.total_lines.max(1);
        let languages = self
            .languages
            .into_iter()
            .map(|(language, stats)| LanguageStat {
                percentage: ((stats.total_lines as f64 / total_lines as f64) * 10000.0).round()
                    / 100.0,
                language,
                file_count: stats.file_count,
                line_count: stats.total_lines,
            })
            .collect();

        LanguageStatsOutput {
            project_path: display_path(&project_path),
            total_files: self.total_files,
            total_lines: self.total_lines,
            languages,
        }
    }

    fn into_loc_output(self, project_path: PathBuf) -> LocStatsOutput {
        let languages = self
            .languages
            .into_iter()
            .map(|(language, stats)| LocLanguageStat {
                language,
                file_count: stats.file_count,
                total_lines: stats.total_lines,
                code_lines: stats.code_lines,
                comment_lines: stats.comment_lines,
                blank_lines: stats.blank_lines,
            })
            .collect();

        LocStatsOutput {
            project_path: display_path(&project_path),
            total_lines: self.total_lines,
            code_lines: self.code_lines,
            comment_lines: self.comment_lines,
            blank_lines: self.blank_lines,
            languages,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct LineCount {
    total_lines: usize,
    code_lines: usize,
    comment_lines: usize,
    blank_lines: usize,
}

fn build_analyzer(project_root: &Path, cli: &Cli) -> CoreAnalyzer {
    let languages = (!cli.languages.is_empty()).then_some(cli.languages.clone());
    let ignore_patterns = (!cli.ignore_patterns.is_empty()).then_some(cli.ignore_patterns.clone());

    CoreAnalyzer::with_options(
        display_path(project_root),
        languages,
        ignore_patterns,
        CoreAnalyzerOptions {
            max_files: cli.max_files,
            parallel_processing: !cli.serial,
        },
    )
}

fn resolve_project_root(path: &Path, override_root: Option<&PathBuf>) -> Result<PathBuf, String> {
    let candidate = override_root.cloned().unwrap_or_else(|| path.to_path_buf());
    validate_directory_path(&display_path(&candidate)).map_err(|err| err.to_string())?;
    std::fs::canonicalize(candidate).map_err(|err| err.to_string())
}

fn resolve_project_file(project_root: &Path, file: &Path) -> PathBuf {
    if file.is_absolute() {
        file.to_path_buf()
    } else {
        project_root.join(file)
    }
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn print_output<T: Serialize>(
    format: &OutputFormat,
    quiet: bool,
    value: &T,
    text_renderer: fn(&T, bool) -> String,
) -> Result<(), String> {
    match format {
        OutputFormat::Text => {
            println!("{}", text_renderer(value, quiet));
            Ok(())
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(value).map_err(|err| err.to_string())?
            );
            Ok(())
        }
        OutputFormat::Yaml => {
            print!(
                "{}",
                serde_yaml::to_string(value).map_err(|err| err.to_string())?
            );
            Ok(())
        }
    }
}

fn render_analyze_text(output: &AnalyzeOutput, quiet: bool) -> String {
    let mut lines = Vec::new();
    if !quiet {
        lines.push(format!("project: {}", output.project_path));
    }
    lines.push(format!("files: {}", output.file_count));
    lines.push(format!("symbols: {}", output.symbol_count));
    lines.push(format!("relationships: {}", output.relationship_count));
    lines.push(format!("languages: {}", output.languages.join(", ")));
    lines.push(format!("duration_ms: {}", output.duration_ms));
    lines.push(format!("skipped_files: {}", output.skipped_file_count));
    if !quiet && !output.skipped_files.is_empty() {
        lines.push("skipped details:".to_string());
        for skipped in &output.skipped_files {
            lines.push(format!(
                "- {} [{}] {}",
                skipped.file_path, skipped.stage, skipped.reason
            ));
        }
    }
    lines.join("\n")
}

fn render_symbols_by_kind_text(output: &SymbolsByKindOutput, quiet: bool) -> String {
    let mut lines = Vec::new();
    if !quiet {
        lines.push(format!(
            "project: {}\nkind: {}\ncount: {}",
            output.project_path, output.symbol_kind, output.count
        ));
    }
    lines.extend(output.symbols.iter().cloned());
    lines.join("\n")
}

fn render_symbols_in_file_text(output: &SymbolsInFileOutput, quiet: bool) -> String {
    let mut lines = Vec::new();
    if !quiet {
        lines.push(format!(
            "project: {}\nfile: {}\ncount: {}",
            output.project_path, output.file_path, output.count
        ));
    }
    lines.extend(output.symbols.iter().cloned());
    lines.join("\n")
}

fn render_dependencies_text(output: &DependenciesOutput, quiet: bool) -> String {
    let mut lines = Vec::new();
    if !quiet {
        lines.push(format!(
            "project: {}\nsymbol: {}\ncount: {}",
            output.project_path, output.symbol_name, output.count
        ));
    }
    lines.extend(output.dependencies.iter().cloned());
    lines.join("\n")
}

fn render_language_stats_text(output: &LanguageStatsOutput, quiet: bool) -> String {
    let mut lines = Vec::new();
    if !quiet {
        lines.push(format!(
            "project: {}\ntotal_files: {}\ntotal_lines: {}",
            output.project_path, output.total_files, output.total_lines
        ));
    }
    for language in &output.languages {
        lines.push(format!(
            "{:<15} {:>6} files {:>8} lines {:>7.2}%",
            language.language, language.file_count, language.line_count, language.percentage
        ));
    }
    lines.join("\n")
}

fn render_loc_stats_text(output: &LocStatsOutput, quiet: bool) -> String {
    let mut lines = Vec::new();
    lines.push(format!("total_lines: {}", output.total_lines));
    lines.push(format!("code_lines: {}", output.code_lines));
    lines.push(format!("comment_lines: {}", output.comment_lines));
    lines.push(format!("blank_lines: {}", output.blank_lines));
    if !quiet {
        lines.push("per-language:".to_string());
        for language in &output.languages {
            lines.push(format!(
                "{:<15} files {:>5} total {:>7} code {:>7} comment {:>7} blank {:>7}",
                language.language,
                language.file_count,
                language.total_lines,
                language.code_lines,
                language.comment_lines,
                language.blank_lines
            ));
        }
    }
    lines.join("\n")
}

fn render_doctor_text(output: &DoctorOutput, quiet: bool) -> String {
    let mut lines = vec![format!("status: {}", output.status)];
    if !quiet {
        for check in &output.checks {
            lines.push(format!(
                "{} [{}] {}",
                check.name, check.status, check.detail
            ));
        }
        for warning in &output.warnings {
            lines.push(format!("warning: {warning}"));
        }
        for error in &output.errors {
            lines.push(format!("error: {error}"));
        }
    }
    lines.join("\n")
}

fn render_mcp_init_text(output: &McpInitOutput, quiet: bool) -> String {
    let mut lines = Vec::new();
    if !quiet {
        if let Some(config_path) = &output.config_path {
            lines.push(format!("config_path: {config_path}"));
        }
        lines.push(format!("command: {}", output.server_command));
        lines.push(format!("args: {}", output.arguments.join(" ")));
    }
    lines.push(format!("written: {}", output.written));
    lines.join("\n")
}

fn render_mcp_doctor_text(output: &McpDoctorOutput, quiet: bool) -> String {
    let mut lines = vec![format!("status: {}", output.status)];
    if !quiet {
        if let Some(binary_path) = &output.binary_path {
            lines.push(format!("binary_path: {binary_path}"));
        }
        lines.push(format!("command: {}", output.server_command));
        lines.push(format!("args: {}", output.arguments.join(" ")));
        for check in &output.checks {
            lines.push(format!(
                "{} [{}] {}",
                check.name, check.status, check.detail
            ));
        }
        for warning in &output.warnings {
            lines.push(format!("warning: {warning}"));
        }
        for error in &output.errors {
            lines.push(format!("error: {error}"));
        }
    }
    lines.join("\n")
}

fn collect_stats(project_root: &Path, cli: &Cli) -> Result<ProjectStatsReport, String> {
    let ignore_patterns = merged_ignore_patterns(
        (!cli.ignore_patterns.is_empty()).then_some(cli.ignore_patterns.clone()),
    );
    let requested_languages = normalized_requested_languages(&cli.languages)?;
    let mut report = ProjectStatsReport::default();
    let mut processed_files = 0usize;

    for entry in WalkDir::new(project_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        if cli
            .max_files
            .is_some_and(|max_files| processed_files >= max_files)
        {
            break;
        }

        let path = entry.path();
        if is_excluded_source_artifact(path) {
            continue;
        }

        let path_string = display_path(path);
        if should_ignore_file(&path_string, &ignore_patterns) {
            continue;
        }

        let Some(language) = detect_language_id(&path_string) else {
            continue;
        };
        let language_name = language.to_lowercase_string();

        if !requested_languages.is_empty() && !requested_languages.contains(&language_name) {
            continue;
        }

        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => continue,
        };

        let counts = count_lines(&content, language);
        report.record_file(&language_name, counts);
        processed_files += 1;
    }

    Ok(report)
}

fn normalized_requested_languages(languages: &[String]) -> Result<Vec<String>, String> {
    languages
        .iter()
        .map(|language| {
            LanguageId::from_string(language)
                .map(|language_id| language_id.to_lowercase_string())
                .ok_or_else(|| format!("unsupported language filter: {language}"))
        })
        .collect()
}

fn is_excluded_source_artifact(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    file_name.ends_with(".d.ts") || file_name.ends_with(".d.mts") || file_name.ends_with(".d.cts")
}

fn count_lines(content: &str, language: LanguageId) -> LineCount {
    let comment_style = comment_style(language);
    let mut counts = LineCount::default();
    let mut in_block_comment = false;

    for line in content.lines() {
        counts.total_lines += 1;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            counts.blank_lines += 1;
            continue;
        }

        if in_block_comment {
            counts.comment_lines += 1;
            if let Some(end) = comment_style.block_end {
                if trimmed.contains(end) {
                    in_block_comment = false;
                }
            }
            continue;
        }

        if comment_style
            .line_prefixes
            .iter()
            .any(|prefix| trimmed.starts_with(prefix))
        {
            counts.comment_lines += 1;
            continue;
        }

        if let Some(start) = comment_style.block_start {
            if trimmed.starts_with(start) {
                counts.comment_lines += 1;
                if let Some(end) = comment_style.block_end {
                    if !trimmed.contains(end) {
                        in_block_comment = true;
                    }
                }
                continue;
            }
        }

        counts.code_lines += 1;
    }

    counts
}

struct CommentStyle {
    line_prefixes: &'static [&'static str],
    block_start: Option<&'static str>,
    block_end: Option<&'static str>,
}

fn comment_style(language: LanguageId) -> CommentStyle {
    match language {
        LanguageId::Python | LanguageId::Ruby | LanguageId::Bash | LanguageId::YAML => {
            CommentStyle {
                line_prefixes: &["#"],
                block_start: None,
                block_end: None,
            }
        }
        LanguageId::Lua => CommentStyle {
            line_prefixes: &["--"],
            block_start: Some("--[["),
            block_end: Some("]]"),
        },
        LanguageId::HTML | LanguageId::XML | LanguageId::Markdown => CommentStyle {
            line_prefixes: &[],
            block_start: Some("<!--"),
            block_end: Some("-->"),
        },
        LanguageId::JSON => CommentStyle {
            line_prefixes: &[],
            block_start: None,
            block_end: None,
        },
        _ => CommentStyle {
            line_prefixes: &["//"],
            block_start: Some("/*"),
            block_end: Some("*/"),
        },
    }
}

fn run_doctor(project_root: &Path) -> DoctorOutput {
    let mut checks = Vec::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let project_result = validate_directory_path(&display_path(project_root));
    match project_result {
        Ok(resolved_path) => checks.push(DoctorCheck {
            name: "project_root".to_string(),
            status: "ok".to_string(),
            detail: display_path(&resolved_path),
        }),
        Err(err) => {
            checks.push(DoctorCheck {
                name: "project_root".to_string(),
                status: "error".to_string(),
                detail: err.to_string(),
            });
        }
    }

    match std::env::current_exe() {
        Ok(path) => checks.push(DoctorCheck {
            name: "binary".to_string(),
            status: "ok".to_string(),
            detail: display_path(&path),
        }),
        Err(err) => {
            warnings.push(format!("failed to resolve current executable: {err}"));
            checks.push(DoctorCheck {
                name: "binary".to_string(),
                status: "warning".to_string(),
                detail: err.to_string(),
            });
        }
    }

    checks.push(command_check("node", &["--version"]));
    checks.push(command_check("python", &["--version"]));
    checks.push(command_check("rustc", &["--version"]));
    checks.push(command_check("cargo", &["--version"]));

    for check in &checks {
        if check.status == "error" {
            errors.push(format!("{}: {}", check.name, check.detail));
        } else if check.status == "warning" {
            warnings.push(format!("{}: {}", check.name, check.detail));
        }
    }

    DoctorOutput {
        status: if errors.is_empty() {
            if warnings.is_empty() {
                "ok".to_string()
            } else {
                "warning".to_string()
            }
        } else {
            "error".to_string()
        },
        checks,
        errors,
        warnings,
    }
}

fn command_check(command: &str, args: &[&str]) -> DoctorCheck {
    match Command::new(command).args(args).output() {
        Ok(output) if output.status.success() => {
            let detail = if output.stdout.is_empty() {
                String::from_utf8_lossy(&output.stderr).trim().to_string()
            } else {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            };
            DoctorCheck {
                name: command.to_string(),
                status: "ok".to_string(),
                detail,
            }
        }
        Ok(output) => DoctorCheck {
            name: command.to_string(),
            status: "warning".to_string(),
            detail: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        },
        Err(err) => DoctorCheck {
            name: command.to_string(),
            status: "warning".to_string(),
            detail: err.to_string(),
        },
    }
}

#[derive(Debug, Clone)]
struct McpLaunchConfig {
    command: String,
    args: Vec<String>,
    cwd: Option<String>,
    binary_path: Option<String>,
}

fn run_mcp_init(
    project_root: &Path,
    config_path: Option<&PathBuf>,
    default_location: bool,
    stdout: bool,
    server_name: &str,
) -> Result<McpInitOutput, String> {
    if !stdout && config_path.is_none() && !default_location {
        return Err(
            "mcp init requires --stdout, --config-path <path>, or --default-location".to_string(),
        );
    }

    let launch = determine_mcp_launch(project_root, None);
    let preview = build_mcp_config_preview(server_name, &launch);

    let resolved_config_path = if stdout {
        None
    } else if let Some(config_path) = config_path {
        Some(config_path.clone())
    } else if default_location {
        Some(default_mcp_config_path()?)
    } else {
        None
    };

    let mut written = false;
    if let Some(config_path) = resolved_config_path.as_ref() {
        write_mcp_config(config_path, server_name, &launch)?;
        written = true;
    }

    Ok(McpInitOutput {
        config_path: resolved_config_path.as_ref().map(|path| display_path(path)),
        server_command: launch.command,
        arguments: launch.args,
        written,
        preview,
    })
}

fn run_mcp_doctor(project_root: &Path, binary_path: Option<&PathBuf>) -> McpDoctorOutput {
    let launch = determine_mcp_launch(project_root, binary_path);
    let mut checks = Vec::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    checks.push(DoctorCheck {
        name: "project_root".to_string(),
        status: if project_root.exists() && project_root.is_dir() {
            "ok".to_string()
        } else {
            "error".to_string()
        },
        detail: display_path(project_root),
    });

    if let Some(binary_path) = &launch.binary_path {
        checks.push(DoctorCheck {
            name: "mcp_binary".to_string(),
            status: "ok".to_string(),
            detail: binary_path.clone(),
        });
    } else {
        checks.push(DoctorCheck {
            name: "mcp_binary".to_string(),
            status: "warning".to_string(),
            detail: "compiled MCP binary not found; using cargo run fallback".to_string(),
        });
    }

    if launch.command == "cargo" {
        checks.push(command_check("cargo", &["--version"]));
    }

    for check in &checks {
        if check.status == "error" {
            errors.push(format!("{}: {}", check.name, check.detail));
        } else if check.status == "warning" {
            warnings.push(format!("{}: {}", check.name, check.detail));
        }
    }

    McpDoctorOutput {
        status: if errors.is_empty() {
            if warnings.is_empty() {
                "ok".to_string()
            } else {
                "warning".to_string()
            }
        } else {
            "error".to_string()
        },
        binary_path: launch.binary_path.clone(),
        server_command: launch.command,
        arguments: launch.args,
        checks,
        errors,
        warnings,
    }
}

fn determine_mcp_launch(project_root: &Path, binary_override: Option<&PathBuf>) -> McpLaunchConfig {
    let candidate = binary_override
        .cloned()
        .or_else(find_sibling_mcp_binary)
        .or_else(|| {
            let mut path = PathBuf::from("target");
            path.push("debug");
            path.push(mcp_binary_name());
            path.exists().then_some(path)
        });

    if let Some(binary_path) = candidate {
        return McpLaunchConfig {
            command: display_path(&binary_path),
            args: Vec::new(),
            cwd: None,
            binary_path: Some(display_path(&binary_path)),
        };
    }

    McpLaunchConfig {
        command: "cargo".to_string(),
        args: vec![
            "run".to_string(),
            "--quiet".to_string(),
            "--bin".to_string(),
            "fast-context-mcp".to_string(),
            "--features".to_string(),
            "mcp".to_string(),
        ],
        cwd: Some(display_path(project_root)),
        binary_path: None,
    }
}

fn build_mcp_config_preview(server_name: &str, launch: &McpLaunchConfig) -> serde_json::Value {
    let mut server = serde_json::json!({
        "command": launch.command,
        "args": launch.args,
    });

    if let Some(cwd) = &launch.cwd {
        server["cwd"] = serde_json::Value::String(cwd.clone());
    }

    serde_json::json!({
        "mcpServers": {
            server_name: server
        }
    })
}

fn default_mcp_config_path() -> Result<PathBuf, String> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| "unable to determine config directory".to_string())?;
    Ok(config_dir.join("Claude").join("claude_desktop_config.json"))
}

fn write_mcp_config(
    config_path: &Path,
    server_name: &str,
    launch: &McpLaunchConfig,
) -> Result<(), String> {
    let mut root = if config_path.exists() {
        let content = fs::read_to_string(config_path).map_err(|err| err.to_string())?;
        serde_json::from_str::<serde_json::Value>(&content).map_err(|err| err.to_string())?
    } else {
        serde_json::json!({})
    };

    if !root.is_object() {
        return Err("existing MCP config must be a JSON object".to_string());
    }

    if root.get("mcpServers").is_none() {
        root["mcpServers"] = serde_json::json!({});
    }

    if !root["mcpServers"].is_object() {
        return Err("existing mcpServers value must be a JSON object".to_string());
    }

    let mut server_entry = serde_json::json!({
        "command": launch.command,
        "args": launch.args,
    });

    if let Some(cwd) = &launch.cwd {
        server_entry["cwd"] = serde_json::Value::String(cwd.clone());
    }

    root["mcpServers"][server_name] = server_entry;

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let rendered = serde_json::to_string_pretty(&root).map_err(|err| err.to_string())?;
    fs::write(config_path, rendered).map_err(|err| err.to_string())
}

fn find_sibling_mcp_binary() -> Option<PathBuf> {
    let mut path = std::env::current_exe().ok()?;
    path.set_file_name(mcp_binary_name());
    path.exists().then_some(path)
}

fn mcp_binary_name() -> &'static str {
    if cfg!(windows) {
        "fast-context-mcp.exe"
    } else {
        "fast-context-mcp"
    }
}

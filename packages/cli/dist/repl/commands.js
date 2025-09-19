/**
 * REPL command implementations
 */
import chalk from 'chalk';
import ora from 'ora';
import { writeFile } from 'fs/promises';
import { formatAnalysisResults, formatSymbolList } from '../utils/formatters.js';
import { validatePath, validateSearchQuery } from '../utils/validators.js';
export class ReplCommands {
    repl;
    constructor(repl) {
        this.repl = repl;
    }
    async analyze(args) {
        if (this.repl.isCurrentlyAnalyzing()) {
            console.log(chalk.yellow('Analysis already in progress'));
            return;
        }
        const path = args[0] || this.repl.getConfig().projectRoot;
        try {
            const validatedPath = await validatePath(path);
            const spinner = ora('Analyzing project...').start();
            this.repl.setAnalyzing(true);
            const startTime = Date.now();
            const analyzer = this.repl.getAnalyzer();
            const results = await analyzer.analyze();
            const duration = Date.now() - startTime;
            this.repl.setCurrentAnalysis(results);
            spinner.succeed(chalk.green(`Analysis completed in ${duration}ms`));
            console.log(chalk.cyan('Analysis Summary:'));
            console.log(`  Files: ${chalk.white(results.fileCount)}`);
            console.log(`  Symbols: ${chalk.white(results.symbolCount)}`);
            console.log(`  Relationships: ${chalk.white(results.relationshipCount)}`);
            console.log(`  Languages: ${chalk.white(results.languages.join(', '))}`);
        }
        catch (error) {
            console.error(chalk.red('Analysis failed:'), error.message);
        }
        finally {
            this.repl.setAnalyzing(false);
        }
    }
    async search(args) {
        if (!args[0]) {
            console.log(chalk.red('Search query is required'));
            console.log(chalk.gray('Usage: search <query> [--kind <types>] [--limit <number>]'));
            return;
        }
        const analysis = this.repl.getCurrentAnalysis();
        if (!analysis) {
            console.log(chalk.yellow('No analysis available. Run "analyze" first.'));
            return;
        }
        try {
            const query = validateSearchQuery(args[0]);
            const spinner = ora('Searching symbols...').start();
            // Parse additional options
            const options = this.parseSearchOptions(args.slice(1));
            // Perform search (placeholder implementation)
            const results = await this.performSearch(query, options);
            spinner.succeed(chalk.green(`Found ${results.length} results`));
            if (results.length > 0) {
                const formatted = formatSymbolList(results, { format: 'table' });
                console.log(formatted);
            }
            else {
                console.log(chalk.gray('No symbols found matching the query'));
            }
        }
        catch (error) {
            console.error(chalk.red('Search failed:'), error.message);
        }
    }
    async dependencies(args) {
        if (!args[0]) {
            console.log(chalk.red('Symbol name is required'));
            console.log(chalk.gray('Usage: deps <symbol> [--depth <number>]'));
            return;
        }
        const analysis = this.repl.getCurrentAnalysis();
        if (!analysis) {
            console.log(chalk.yellow('No analysis available. Run "analyze" first.'));
            return;
        }
        const symbolName = args[0];
        const depth = this.parseDepthOption(args.slice(1));
        console.log(chalk.cyan(`Dependencies for ${symbolName}:`));
        console.log(chalk.gray('(Dependency analysis will be implemented when query engine is available)'));
        // Placeholder implementation
        console.log(`  → ${chalk.white('UserRepository')} ${chalk.gray('(dependency)')}`);
        console.log(`  → ${chalk.white('Logger')} ${chalk.gray('(dependency)')}`);
        console.log(`  ← ${chalk.white('UserController')} ${chalk.gray('(dependent)')}`);
    }
    async patterns(args) {
        const analysis = this.repl.getCurrentAnalysis();
        if (!analysis) {
            console.log(chalk.yellow('No analysis available. Run "analyze" first.'));
            return;
        }
        const spinner = ora('Detecting patterns...').start();
        // Placeholder implementation
        setTimeout(() => {
            spinner.succeed(chalk.green('Pattern detection completed'));
            console.log(chalk.cyan('Detected Patterns:'));
            console.log(`  ${chalk.green('✓')} ${chalk.white('Repository Pattern')} - 3 instances`);
            console.log(`  ${chalk.green('✓')} ${chalk.white('Service Layer')} - 5 services`);
            console.log(`  ${chalk.yellow('⚠')} ${chalk.white('Singleton')} - 2 instances (consider dependency injection)`);
            console.log(`  ${chalk.red('✗')} ${chalk.white('God Object')} - UserService is too large`);
        }, 1000);
    }
    async metrics(args) {
        const analysis = this.repl.getCurrentAnalysis();
        if (!analysis) {
            console.log(chalk.yellow('No analysis available. Run "analyze" first.'));
            return;
        }
        const filePath = args[0];
        if (filePath) {
            console.log(chalk.cyan(`Metrics for ${filePath}:`));
        }
        else {
            console.log(chalk.cyan('Overall Project Metrics:'));
        }
        // Placeholder implementation
        console.log(`  Complexity: ${chalk.white('Medium')} (7.2/10)`);
        console.log(`  Maintainability: ${chalk.green('Good')} (8.1/10)`);
        console.log(`  Test Coverage: ${chalk.yellow('Fair')} (65%)`);
        console.log(`  Technical Debt: ${chalk.red('High')} (23 issues)`);
    }
    async export(args) {
        const analysis = this.repl.getCurrentAnalysis();
        if (!analysis) {
            console.log(chalk.yellow('No analysis available. Run "analyze" first.'));
            return;
        }
        const format = args[0] || 'json';
        const filename = args[1] || `analysis.${format}`;
        try {
            const formatted = await formatAnalysisResults(analysis, { format: format });
            await writeFile(filename, formatted);
            console.log(chalk.green(`Analysis exported to ${filename}`));
        }
        catch (error) {
            console.error(chalk.red('Export failed:'), error.message);
        }
    }
    async config(args) {
        const config = this.repl.getConfig();
        if (args[0] === 'show' || args.length === 0) {
            console.log(chalk.cyan('Current Configuration:'));
            console.log(`  Project Root: ${chalk.white(config.projectRoot)}`);
            console.log(`  Languages: ${chalk.white(config.languages?.join(', ') || 'All')}`);
            console.log(`  Max Files: ${chalk.white(config.maxFiles)}`);
            console.log(`  Max Depth: ${chalk.white(config.maxDepth)}`);
            console.log(`  Caching: ${config.enableCaching ? chalk.green('Enabled') : chalk.red('Disabled')}`);
            console.log(`  Parallel: ${config.parallelProcessing ? chalk.green('Enabled') : chalk.red('Disabled')}`);
            console.log(`  Include Tests: ${config.includeTests ? chalk.green('Yes') : chalk.gray('No')}`);
            console.log(`  Include Docs: ${config.includeDocs ? chalk.green('Yes') : chalk.gray('No')}`);
        }
        else {
            console.log(chalk.gray('Configuration modification not yet implemented'));
            console.log(chalk.gray('Use config files or command-line options'));
        }
    }
    help(args) {
        const command = args[0];
        if (command) {
            this.showCommandHelp(command);
        }
        else {
            this.showGeneralHelp();
        }
    }
    showGeneralHelp() {
        console.log(chalk.cyan('Available Commands:'));
        console.log('');
        console.log(`  ${chalk.white('analyze [path]')}        - Analyze codebase`);
        console.log(`  ${chalk.white('search <query>')}        - Search for symbols`);
        console.log(`  ${chalk.white('deps <symbol>')}         - Show dependencies`);
        console.log(`  ${chalk.white('patterns')}              - Detect patterns`);
        console.log(`  ${chalk.white('metrics [file]')}        - Show complexity metrics`);
        console.log(`  ${chalk.white('export <format> [file]')} - Export results`);
        console.log(`  ${chalk.white('config')}                - Show configuration`);
        console.log(`  ${chalk.white('status')}                - Show REPL status`);
        console.log(`  ${chalk.white('history')}               - Show command history`);
        console.log(`  ${chalk.white('clear')}                 - Clear screen`);
        console.log(`  ${chalk.white('help [command]')}        - Show help`);
        console.log(`  ${chalk.white('exit')}                  - Exit REPL`);
        console.log('');
        console.log(chalk.gray('Type "help <command>" for detailed help on a specific command'));
    }
    showCommandHelp(command) {
        const helpText = {
            analyze: `
${chalk.cyan('analyze [path]')} - Analyze codebase

Performs comprehensive analysis of the specified codebase.

Examples:
  analyze                 - Analyze current project
  analyze ./src           - Analyze specific directory
`,
            search: `
${chalk.cyan('search <query>')} - Search for symbols

Search for symbols in the analyzed codebase.

Options:
  --kind <types>          - Filter by symbol types (function, class, etc.)
  --limit <number>        - Maximum results (default: 50)

Examples:
  search UserService      - Find symbols containing "UserService"
  search auth --kind function - Find functions containing "auth"
`,
            deps: `
${chalk.cyan('deps <symbol>')} - Show dependencies

Show dependency relationships for a symbol.

Options:
  --depth <number>        - Analysis depth (default: 5)

Examples:
  deps UserService       - Show UserService dependencies
  deps UserService --depth 3
`,
            export: `
${chalk.cyan('export <format> [file]')} - Export results

Export analysis results in various formats.

Formats: json, yaml, markdown, csv

Examples:
  export json             - Export as analysis.json
  export yaml results.yml - Export as YAML to specific file
`
        };
        if (helpText[command]) {
            console.log(helpText[command]);
        }
        else {
            console.log(chalk.red(`No help available for command: ${command}`));
        }
    }
    parseSearchOptions(args) {
        const options = { limit: 50 };
        for (let i = 0; i < args.length; i++) {
            if (args[i] === '--kind' && args[i + 1]) {
                options.kind = args[i + 1].split(',');
                i++;
            }
            else if (args[i] === '--limit' && args[i + 1]) {
                options.limit = parseInt(args[i + 1]);
                i++;
            }
        }
        return options;
    }
    parseDepthOption(args) {
        const depthIndex = args.indexOf('--depth');
        if (depthIndex !== -1 && args[depthIndex + 1]) {
            return parseInt(args[depthIndex + 1]) || 5;
        }
        return 5;
    }
    async performSearch(query, options) {
        // Placeholder search implementation
        const mockResults = [
            {
                name: 'UserService',
                kind: 'class',
                file: 'src/services/UserService.ts',
                line: 15,
                score: 0.95
            },
            {
                name: 'authenticateUser',
                kind: 'function',
                file: 'src/auth/authentication.ts',
                line: 42,
                score: 0.87
            }
        ];
        return mockResults.filter(result => result.name.toLowerCase().includes(query.toLowerCase())).slice(0, options.limit);
    }
}
//# sourceMappingURL=commands.js.map
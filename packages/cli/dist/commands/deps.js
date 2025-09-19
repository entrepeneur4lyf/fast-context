/**
 * Dependencies command - Analyze symbol dependencies
 */
import { Command } from 'commander';
import chalk from 'chalk';
import ora from 'ora';
import { EnhancedFastContextAnalyzer } from '@fast-context/core';
import { formatDependencyGraph } from '../utils/formatters.js';
import { loadConfig } from '../config/loader.js';
import { validatePath, validateSymbolName } from '../utils/validators.js';
export const depsCommand = new Command('deps')
    .alias('d')
    .description('Analyze dependency relationships for a symbol')
    .argument('<symbol>', 'symbol name to analyze')
    .argument('[path]', 'path to the project root directory', '.')
    .option('-d, --depth <number>', 'analysis depth', '5')
    .option('--include-external', 'include external dependencies')
    .option('--include-reverse', 'include reverse dependencies (dependents)')
    .option('-f, --format <format>', 'output format (table|json|yaml)', 'table')
    .option('-o, --output <file>', 'output file path')
    .action(async (symbolName, projectPath, options) => {
    const spinner = ora('Initializing dependency analysis...').start();
    try {
        // Validate inputs
        const validatedPath = await validatePath(projectPath);
        const validatedSymbol = validateSymbolName(symbolName);
        // Load configuration
        const config = await loadConfig(options.config, {
            projectRoot: validatedPath
        });
        spinner.text = 'Creating analyzer...';
        // Create analyzer
        const analyzer = new EnhancedFastContextAnalyzer(config);
        spinner.text = 'Analyzing dependencies...';
        // Perform dependency analysis (placeholder)
        const results = await performDependencyAnalysis(analyzer, validatedSymbol, {
            depth: parseInt(options.depth),
            includeExternal: options.includeExternal,
            includeReverse: options.includeReverse
        });
        spinner.succeed(chalk.green('Dependency analysis completed'));
        // Format and display results
        const formattedResults = formatDependencyGraph(results, {
            format: options.format,
            verbose: options.verbose,
            quiet: options.quiet,
            json: options.json
        });
        // Output results
        if (options.output) {
            const fs = await import('fs/promises');
            await fs.writeFile(options.output, formattedResults);
            console.log(chalk.green(`Results saved to ${options.output}`));
        }
        else {
            console.log(formattedResults);
        }
    }
    catch (error) {
        spinner.fail(chalk.red('Dependency analysis failed'));
        console.error(chalk.red('Error:'), error.message);
        process.exit(1);
    }
});
async function performDependencyAnalysis(analyzer, symbol, options) {
    // Placeholder implementation
    return [
        {
            symbol: symbol,
            dependencies: ['UserRepository', 'Logger', 'ValidationService'],
            dependents: ['UserController', 'AdminService'],
            depth: 1
        }
    ];
}
depsCommand.addHelpText('after', `
Examples:
  $ fast-context deps UserService
  $ fast-context deps UserService --depth 3 --include-external
  $ fast-context deps AuthService --format json --output deps.json
`);
//# sourceMappingURL=deps.js.map
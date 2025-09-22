#!/usr/bin/env node
/**
 * Fast-Context CLI - Command-line interface for codebase analysis
 */
import { Command } from 'commander';
import chalk from 'chalk';
import figlet from 'figlet';
import { readFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
// Import commands
import { analyzeCommand } from '../commands/analyze.js';
import { searchCommand } from '../commands/search.js';
import { depsCommand } from '../commands/deps.js';
import { patternsCommand } from '../commands/patterns.js';
import { metricsCommand } from '../commands/metrics.js';
import { replCommand } from '../commands/repl.js';
import { configCommand } from '../commands/config.js';
import { debugCommand } from '../commands/debug.js';
import { exportCommand } from '../commands/export.js';
import { watchCommand } from '../commands/watch.js';
// Get package info
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const packageJson = JSON.parse(readFileSync(join(__dirname, '../../package.json'), 'utf-8'));
// Note: Update checking will be implemented in a future version
// Create main program
const program = new Command();
// Configure program
program
    .name('fast-context')
    .alias('fc')
    .description('Fast-Context CLI - Intelligent codebase analysis engine')
    .version(packageJson.version, '-v, --version', 'display version number')
    .helpOption('-h, --help', 'display help for command')
    .configureHelp({
    sortSubcommands: true,
    subcommandTerm: (cmd) => cmd.name() + ' ' + cmd.usage()
});
// Global options
program
    .option('-c, --config <path>', 'path to configuration file')
    .option('-v, --verbose', 'enable verbose output')
    .option('-q, --quiet', 'suppress non-error output')
    .option('--no-color', 'disable colored output')
    .option('--json', 'output results in JSON format')
    .option('--debug', 'enable debug mode');
// Add commands
program.addCommand(analyzeCommand);
program.addCommand(searchCommand);
program.addCommand(depsCommand);
program.addCommand(patternsCommand);
program.addCommand(metricsCommand);
program.addCommand(replCommand);
program.addCommand(configCommand);
program.addCommand(debugCommand);
program.addCommand(exportCommand);
program.addCommand(watchCommand);
// Custom help
program.on('--help', () => {
    console.log('');
    console.log(chalk.cyan(figlet.textSync('Fast-Context', {
        font: 'Small',
        horizontalLayout: 'fitted'
    })));
    console.log('');
    console.log(chalk.gray('Examples:'));
    console.log('  $ fast-context analyze ./my-project');
    console.log('  $ fast-context search "authentication"');
    console.log('  $ fast-context deps UserService');
    console.log('  $ fast-context repl');
    console.log('  $ fast-context watch ./src');
    console.log('');
    console.log(chalk.gray('For more information, visit:'));
    console.log(chalk.blue('  https://fast-context.dev/docs/cli'));
});
// Error handling
program.exitOverride();
try {
    program.parse();
}
catch (err) {
    if (err.code === 'commander.help') {
        process.exit(0);
    }
    else if (err.code === 'commander.version') {
        process.exit(0);
    }
    else if (err.code === 'commander.helpDisplayed') {
        process.exit(0);
    }
    else {
        console.error(chalk.red('Error:'), err.message);
        process.exit(1);
    }
}
// If no command provided, show help
if (!process.argv.slice(2).length) {
    program.outputHelp();
}
//# sourceMappingURL=cli.js.map
/**
 * REPL command - Interactive codebase exploration
 */
import { Command } from 'commander';
import chalk from 'chalk';
import figlet from 'figlet';
import { FastContextRepl } from '../repl/repl.js';
import { loadConfig } from '../config/loader.js';
import { validatePath } from '../utils/validators.js';
export const replCommand = new Command('repl')
    .alias('r')
    .description('Start interactive REPL for codebase exploration')
    .argument('[path]', 'path to the project root directory', '.')
    .option('-l, --languages <langs...>', 'specific languages to analyze')
    .option('--no-banner', 'disable welcome banner')
    .option('--no-auto-analyze', 'disable automatic analysis on startup')
    .option('--history-file <path>', 'custom history file path')
    .option('--max-history <number>', 'maximum history entries', '1000')
    .action(async (projectPath, options) => {
    try {
        // Validate input path
        const validatedPath = await validatePath(projectPath);
        // Load configuration
        const config = await loadConfig(options.config, {
            projectRoot: validatedPath,
            languages: options.languages
        });
        // Show welcome banner
        if (!options.noBanner) {
            console.log(chalk.cyan(figlet.textSync('Fast-Context REPL', {
                font: 'Small',
                horizontalLayout: 'fitted'
            })));
            console.log(chalk.gray('Interactive codebase exploration and analysis'));
            console.log(chalk.gray(`Project: ${validatedPath}`));
            console.log('');
        }
        // Create and start REPL
        const repl = new FastContextRepl({
            config,
            autoAnalyze: !options.noAutoAnalyze,
            historyFile: options.historyFile,
            maxHistory: parseInt(options.maxHistory),
            showBanner: !options.noBanner
        });
        await repl.start();
    }
    catch (error) {
        console.error(chalk.red('Failed to start REPL:'), error.message);
        if (options.debug) {
            console.error(chalk.red('Debug information:'));
            console.error(error.stack);
        }
        process.exit(1);
    }
});
// Add examples to help
replCommand.addHelpText('after', `
Examples:
  $ fast-context repl
  $ fast-context repl ./my-project
  $ fast-context repl --languages typescript javascript
  $ fast-context repl --no-auto-analyze --no-banner

REPL Commands:
  analyze [path]           - Analyze codebase
  search <query>           - Search for symbols
  deps <symbol>            - Show dependencies
  patterns                 - Detect patterns
  metrics [file]           - Show complexity metrics
  export <format> [file]   - Export results
  config                   - Show configuration
  help                     - Show help
  clear                    - Clear screen
  exit                     - Exit REPL
`);
//# sourceMappingURL=repl.js.map
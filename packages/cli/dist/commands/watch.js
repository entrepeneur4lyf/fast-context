/**
 * Watch command - Watch for file changes and re-analyze
 */
import { Command } from 'commander';
import chalk from 'chalk';
export const watchCommand = new Command('watch')
    .alias('w')
    .description('Watch for file changes and automatically re-analyze')
    .argument('[path]', 'path to watch', '.')
    .option('--debounce <ms>', 'debounce delay in milliseconds', '1000')
    .action(async (path, options) => {
    console.log(chalk.cyan(`Watching ${path} for changes...`));
    console.log(chalk.gray('(Watch mode will be implemented in a future version)'));
});
//# sourceMappingURL=watch.js.map
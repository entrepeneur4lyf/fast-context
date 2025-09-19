/**
 * Config command - Manage configuration
 */

import { Command } from 'commander';
import chalk from 'chalk';

export const configCommand = new Command('config')
  .description('Manage Fast-Context configuration')
  .option('-g, --global', 'use global configuration')
  .action(async (options: any) => {
    console.log(chalk.cyan('Configuration management'));
    console.log(chalk.gray('(Configuration management will be implemented in a future version)'));
  });

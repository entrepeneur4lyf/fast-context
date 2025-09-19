/**
 * Analyze command - Perform comprehensive codebase analysis
 */

import { Command } from 'commander';
import chalk from 'chalk';
import ora from 'ora';
import { EnhancedFastContextAnalyzer } from '@fast-context/core';
import { formatAnalysisResults } from '../utils/formatters.js';
import { loadConfig } from '../config/loader.js';
import { validatePath } from '../utils/validators.js';


export const analyzeCommand = new Command('analyze')
  .alias('a')
  .description('Analyze a codebase for symbols, dependencies, and patterns')
  .argument('<path>', 'path to the project root directory')
  .option('-l, --languages <langs...>', 'specific languages to analyze (e.g., typescript javascript)')
  .option('-i, --ignore <patterns...>', 'patterns to ignore (e.g., node_modules dist)')
  .option('-d, --depth <number>', 'maximum analysis depth', '10')
  .option('-m, --max-files <number>', 'maximum number of files to analyze', '10000')
  .option('-f, --format <format>', 'output format (table|json|yaml|markdown)', 'table')
  .option('-o, --output <file>', 'output file path')
  .option('--no-cache', 'disable caching')
  .option('--no-parallel', 'disable parallel processing')
  .option('--include-tests', 'include test files in analysis')
  .option('--include-docs', 'include documentation files')
  .option('--metrics-only', 'only show metrics, skip detailed analysis')
  .option('--symbols-only', 'only extract symbols, skip relationships')
  .action(async (projectPath: string, options: any) => {
    const spinner = ora('Initializing analysis...').start();
    
    try {
      // Validate input path
      const validatedPath = await validatePath(projectPath);
      
      // Load configuration
      const config = await loadConfig(options.config, {
        projectRoot: validatedPath,
        languages: options.languages,
        ignorePatterns: options.ignore,
        maxDepth: parseInt(options.depth),
        maxFiles: parseInt(options.maxFiles),
        enableCaching: !options.noCache,
        parallelProcessing: !options.noParallel,
        includeTests: options.includeTests,
        includeDocs: options.includeDocs
      });

      spinner.text = 'Creating analyzer...';
      
      // Create analyzer
      const analyzer = new EnhancedFastContextAnalyzer(config);
      
      spinner.text = 'Starting analysis...';

      // Perform analysis
      const startTime = Date.now();
      const results = await analyzer.analyze();
      const duration = Date.now() - startTime;
      
      spinner.succeed(chalk.green(`Analysis completed in ${duration}ms`));
      
      // Format and display results
      const formattedResults = await formatAnalysisResults(results, {
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
      } else {
        console.log(formattedResults);
      }
      
      // Show summary
      if (!options.quiet && !options.json) {
        console.log('');
        console.log(chalk.cyan('Analysis Summary:'));
        console.log(`  Files analyzed: ${chalk.white(results.fileCount)}`);
        console.log(`  Symbols found: ${chalk.white(results.symbolCount)}`);
        console.log(`  Relationships: ${chalk.white(results.relationshipCount)}`);
        console.log(`  Languages: ${chalk.white(results.languages.join(', '))}`);
        console.log(`  Duration: ${chalk.white(duration)}ms`);
        
        if (results.performance) {
          console.log(`  Memory usage: ${chalk.white(results.performance.memoryUsageMb)}MB`);
          console.log(`  CPU usage: ${chalk.white(results.performance.cpuUsagePercent)}%`);
          console.log(`  Throughput: ${chalk.white(results.performance.throughputFilesPerSecond)} files/sec`);
        }
      }
      
    } catch (error: any) {
      spinner.fail(chalk.red('Analysis failed'));
      
      if (options.debug) {
        console.error(chalk.red('Debug information:'));
        console.error(error.stack);
      } else {
        console.error(chalk.red('Error:'), error.message);
        console.error(chalk.gray('Use --debug for more information'));
      }
      
      process.exit(1);
    }
  });

// Add examples to help
analyzeCommand.addHelpText('after', `
Examples:
  $ fast-context analyze ./my-project
  $ fast-context analyze ./src --languages typescript javascript
  $ fast-context analyze . --ignore node_modules dist --format json
  $ fast-context analyze ./app --output analysis.json --no-cache
  $ fast-context analyze . --metrics-only --format table
`);

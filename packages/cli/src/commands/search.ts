/**
 * Search command - Search for symbols in the codebase
 */

import { Command } from 'commander';
import chalk from 'chalk';
import ora from 'ora';
import { EnhancedFastContextAnalyzer } from '@fast-context/core';
import { formatSymbolList } from '../utils/formatters.js';
import { loadConfig } from '../config/loader.js';
import { validatePath, validateSearchQuery } from '../utils/validators.js';

export const searchCommand = new Command('search')
  .alias('s')
  .description('Search for symbols in the codebase')
  .argument('<query>', 'search query (symbol name or pattern)')
  .argument('[path]', 'path to the project root directory', '.')
  .option('-k, --kind <types...>', 'symbol types to search for (function, class, variable, etc.)')
  .option('-l, --languages <langs...>', 'specific languages to search in')
  .option('-f, --files <patterns...>', 'file patterns to search in')
  .option('-e, --exclude <patterns...>', 'file patterns to exclude')
  .option('--limit <number>', 'maximum number of results', '50')
  .option('--min-score <number>', 'minimum relevance score (0-1)', '0.1')
  .option('-f, --format <format>', 'output format (table|json|yaml)', 'table')
  .option('-o, --output <file>', 'output file path')
  .option('--case-sensitive', 'case-sensitive search')
  .option('--regex', 'treat query as regular expression')
  .option('--exact', 'exact match only')
  .option('--include-references', 'include symbol references in results')
  .action(async (query: string, projectPath: string, options: any) => {
    const spinner = ora('Initializing search...').start();
    
    try {
      // Validate inputs
      const validatedPath = await validatePath(projectPath);
      const validatedQuery = validateSearchQuery(query);
      
      // Load configuration
      const config = await loadConfig(options.config, {
        projectRoot: validatedPath,
        languages: options.languages
      });

      spinner.text = 'Creating analyzer...';
      
      // Create analyzer
      const analyzer = new EnhancedFastContextAnalyzer(config);
      
      // Configure search options
      const searchOptions = {
        query: validatedQuery,
        kind: options.kind,
        files: options.files,
        exclude: options.exclude,
        limit: parseInt(options.limit),
        minScore: parseFloat(options.minScore),
        caseSensitive: options.caseSensitive,
        regex: options.regex,
        exact: options.exact,
        includeReferences: options.includeReferences
      };

      spinner.text = 'Searching symbols...';
      
      // Perform search
      const startTime = Date.now();
      
      // For now, we'll use a placeholder search implementation
      // This will be replaced with actual search functionality when the query engine is available
      const results = await performSymbolSearch(analyzer, searchOptions);
      
      const duration = Date.now() - startTime;
      
      spinner.succeed(chalk.green(`Search completed in ${duration}ms`));
      
      // Format and display results
      const formattedResults = formatSymbolList(results, {
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
        console.log(chalk.cyan('Search Summary:'));
        console.log(`  Query: ${chalk.white(validatedQuery)}`);
        console.log(`  Results found: ${chalk.white(results.length)}`);
        console.log(`  Search time: ${chalk.white(duration)}ms`);
        
        if (options.kind) {
          console.log(`  Symbol types: ${chalk.white(options.kind.join(', '))}`);
        }
        
        if (results.length === parseInt(options.limit)) {
          console.log(chalk.yellow(`  Note: Results limited to ${options.limit}. Use --limit to see more.`));
        }
      }
      
    } catch (error: any) {
      spinner.fail(chalk.red('Search failed'));
      
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

/**
 * Placeholder search implementation
 * This will be replaced with actual search functionality when the query engine is available
 */
async function performSymbolSearch(analyzer: any, options: any): Promise<any[]> {
  // Simulate search results for now
  const mockResults = [
    {
      name: 'UserService',
      kind: 'class',
      file: 'src/services/UserService.ts',
      line: 15,
      score: 0.95,
      description: 'Service class for user management operations'
    },
    {
      name: 'authenticateUser',
      kind: 'function',
      file: 'src/auth/authentication.ts',
      line: 42,
      score: 0.87,
      description: 'Function to authenticate user credentials'
    },
    {
      name: 'userRepository',
      kind: 'variable',
      file: 'src/repositories/UserRepository.ts',
      line: 8,
      score: 0.73,
      description: 'Repository instance for user data access'
    }
  ];
  
  // Filter by query (simple string matching for now)
  const filteredResults = mockResults.filter(result => {
    const queryLower = options.query.toLowerCase();
    const nameLower = result.name.toLowerCase();
    
    if (options.exact) {
      return nameLower === queryLower;
    } else if (options.caseSensitive) {
      return result.name.includes(options.query);
    } else {
      return nameLower.includes(queryLower);
    }
  });
  
  // Filter by kind if specified
  if (options.kind && options.kind.length > 0) {
    return filteredResults.filter(result => options.kind.includes(result.kind));
  }
  
  // Apply limit
  return filteredResults.slice(0, options.limit);
}

// Add examples to help
searchCommand.addHelpText('after', `
Examples:
  $ fast-context search "UserService"
  $ fast-context search "auth" --kind function class
  $ fast-context search "user.*Service" --regex
  $ fast-context search "authenticate" --files "src/auth/**"
  $ fast-context search "User" --exact --case-sensitive
  $ fast-context search "service" --format json --output results.json
`);

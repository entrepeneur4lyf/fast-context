/**
 * Interactive REPL for Fast-Context
 */

import { createInterface, Interface } from 'readline';
import { readFile, writeFile, access } from 'fs/promises';
import { homedir } from 'os';
import { join } from 'path';
import chalk from 'chalk';
import ora from 'ora';
import { EnhancedFastContextAnalyzer } from '@fast-context/core';
import { FastContextConfig } from '../config/loader.js';
import { formatAnalysisResults, formatSymbolList } from '../utils/formatters.js';
import { ReplCommands } from './commands.js';

export interface ReplOptions {
  config: FastContextConfig;
  autoAnalyze?: boolean;
  historyFile?: string;
  maxHistory?: number;
  showBanner?: boolean;
}

export class FastContextRepl {
  private rl: Interface;
  private analyzer: EnhancedFastContextAnalyzer;
  private config: FastContextConfig;
  private commands: ReplCommands;
  private historyFile: string;
  private maxHistory: number;
  private history: string[] = [];
  private currentAnalysis: any = null;
  private isAnalyzing = false;

  constructor(options: ReplOptions) {
    this.config = options.config;
    this.analyzer = new EnhancedFastContextAnalyzer(this.config);
    this.historyFile = options.historyFile || join(homedir(), '.fast-context-history');
    this.maxHistory = options.maxHistory || 1000;
    
    // Create readline interface
    this.rl = createInterface({
      input: process.stdin,
      output: process.stdout,
      prompt: chalk.cyan('fast-context> '),
      historySize: this.maxHistory,
      removeHistoryDuplicates: true
    });
    
    // Initialize commands
    this.commands = new ReplCommands(this);
    
    // Set up event handlers
    this.setupEventHandlers();
  }

  async start(): Promise<void> {
    // Load command history
    await this.loadHistory();
    
    // Auto-analyze if enabled
    if (this.config.projectRoot) {
      console.log(chalk.gray(`Analyzing project: ${this.config.projectRoot}`));
      await this.analyzeProject();
    }
    
    // Show initial prompt
    this.showPrompt();
    
    // Start the REPL loop
    return new Promise((resolve) => {
      this.rl.on('close', () => {
        this.saveHistory();
        resolve();
      });
    });
  }

  private setupEventHandlers(): void {
    // Handle line input
    this.rl.on('line', async (input: string) => {
      const trimmed = input.trim();
      
      if (trimmed) {
        this.addToHistory(trimmed);
        await this.processCommand(trimmed);
      }
      
      this.showPrompt();
    });
    
    // Handle Ctrl+C
    this.rl.on('SIGINT', () => {
      if (this.isAnalyzing) {
        console.log(chalk.yellow('\nAnalysis interrupted'));
        this.isAnalyzing = false;
      } else {
        console.log(chalk.gray('\nUse "exit" to quit'));
      }
      this.showPrompt();
    });
    
    // Note: History is automatically managed by readline interface
  }

  private async processCommand(input: string): Promise<void> {
    const parts = input.split(/\s+/);
    const command = parts[0].toLowerCase();
    const args = parts.slice(1);
    
    try {
      switch (command) {
        case 'analyze':
        case 'a':
          await this.commands.analyze(args);
          break;
          
        case 'search':
        case 's':
          await this.commands.search(args);
          break;
          
        case 'deps':
        case 'd':
          await this.commands.dependencies(args);
          break;
          
        case 'patterns':
        case 'p':
          await this.commands.patterns(args);
          break;
          
        case 'metrics':
        case 'm':
          await this.commands.metrics(args);
          break;
          
        case 'export':
        case 'e':
          await this.commands.export(args);
          break;
          
        case 'config':
        case 'c':
          await this.commands.config(args);
          break;
          
        case 'help':
        case 'h':
        case '?':
          this.commands.help(args);
          break;
          
        case 'clear':
        case 'cls':
          console.clear();
          break;
          
        case 'history':
          this.showHistory();
          break;
          
        case 'status':
          this.showStatus();
          break;
          
        case 'exit':
        case 'quit':
        case 'q':
          console.log(chalk.gray('Goodbye!'));
          this.rl.close();
          break;
          
        default:
          if (command) {
            console.log(chalk.red(`Unknown command: ${command}`));
            console.log(chalk.gray('Type "help" for available commands'));
          }
          break;
      }
    } catch (error: any) {
      console.error(chalk.red('Error:'), error.message);
    }
  }

  private showPrompt(): void {
    const status = this.isAnalyzing ? chalk.yellow('analyzing') : 
                  this.currentAnalysis ? chalk.green('ready') : 
                  chalk.gray('no analysis');
    
    this.rl.setPrompt(chalk.cyan(`fast-context(${status})> `));
    this.rl.prompt();
  }

  private async analyzeProject(): Promise<void> {
    if (this.isAnalyzing) {
      console.log(chalk.yellow('Analysis already in progress'));
      return;
    }
    
    const spinner = ora('Analyzing project...').start();
    this.isAnalyzing = true;
    
    try {
      const startTime = Date.now();
      this.currentAnalysis = await this.analyzer.analyze();
      const duration = Date.now() - startTime;
      
      spinner.succeed(chalk.green(`Analysis completed in ${duration}ms`));
      
      console.log(chalk.cyan('Analysis Summary:'));
      console.log(`  Files: ${chalk.white(this.currentAnalysis.fileCount)}`);
      console.log(`  Symbols: ${chalk.white(this.currentAnalysis.symbolCount)}`);
      console.log(`  Languages: ${chalk.white(this.currentAnalysis.languages.join(', '))}`);
      
    } catch (error: any) {
      spinner.fail(chalk.red('Analysis failed'));
      console.error(chalk.red('Error:'), error.message);
    } finally {
      this.isAnalyzing = false;
    }
  }

  private showHistory(): void {
    if (this.history.length === 0) {
      console.log(chalk.gray('No command history'));
      return;
    }
    
    console.log(chalk.cyan('Command History:'));
    this.history.slice(-10).forEach((cmd, index) => {
      const num = this.history.length - 10 + index + 1;
      console.log(`  ${chalk.gray(num.toString().padStart(3))}: ${cmd}`);
    });
  }

  private showStatus(): void {
    console.log(chalk.cyan('REPL Status:'));
    console.log(`  Project: ${chalk.white(this.config.projectRoot)}`);
    console.log(`  Analysis: ${this.currentAnalysis ? chalk.green('Available') : chalk.gray('None')}`);
    console.log(`  Languages: ${chalk.white(this.config.languages?.join(', ') || 'All')}`);
    console.log(`  History: ${chalk.white(this.history.length)} commands`);
    
    if (this.currentAnalysis) {
      console.log(`  Files analyzed: ${chalk.white(this.currentAnalysis.fileCount)}`);
      console.log(`  Symbols found: ${chalk.white(this.currentAnalysis.symbolCount)}`);
    }
  }

  private async loadHistory(): Promise<void> {
    try {
      await access(this.historyFile);
      const content = await readFile(this.historyFile, 'utf-8');
      this.history = content.split('\n').filter(line => line.trim());
      
      // Note: History will be loaded manually when needed
    } catch {
      // History file doesn't exist, start with empty history
    }
  }

  private async saveHistory(): Promise<void> {
    try {
      const historyToSave = this.history.slice(-this.maxHistory);
      await writeFile(this.historyFile, historyToSave.join('\n'), 'utf-8');
    } catch (error: any) {
      console.error(chalk.yellow(`Warning: Failed to save history: ${error.message}`));
    }
  }

  private addToHistory(command: string): void {
    // Don't add duplicate consecutive commands
    if (this.history.length === 0 || this.history[this.history.length - 1] !== command) {
      this.history.push(command);
      
      // Trim history if it gets too long
      if (this.history.length > this.maxHistory) {
        this.history = this.history.slice(-this.maxHistory);
      }
    }
  }

  // Public methods for commands to use
  public getAnalyzer(): EnhancedFastContextAnalyzer {
    return this.analyzer;
  }

  public getCurrentAnalysis(): any {
    return this.currentAnalysis;
  }

  public setCurrentAnalysis(analysis: any): void {
    this.currentAnalysis = analysis;
  }

  public getConfig(): FastContextConfig {
    return this.config;
  }

  public setAnalyzing(analyzing: boolean): void {
    this.isAnalyzing = analyzing;
  }

  public isCurrentlyAnalyzing(): boolean {
    return this.isAnalyzing;
  }
}

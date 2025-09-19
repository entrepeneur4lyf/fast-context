/**
 * Interactive REPL for Fast-Context
 */
import { EnhancedFastContextAnalyzer } from '@fast-context/core';
import { FastContextConfig } from '../config/loader.js';
export interface ReplOptions {
    config: FastContextConfig;
    autoAnalyze?: boolean;
    historyFile?: string;
    maxHistory?: number;
    showBanner?: boolean;
}
export declare class FastContextRepl {
    private rl;
    private analyzer;
    private config;
    private commands;
    private historyFile;
    private maxHistory;
    private history;
    private currentAnalysis;
    private isAnalyzing;
    constructor(options: ReplOptions);
    start(): Promise<void>;
    private setupEventHandlers;
    private processCommand;
    private showPrompt;
    private analyzeProject;
    private showHistory;
    private showStatus;
    private loadHistory;
    private saveHistory;
    private addToHistory;
    getAnalyzer(): EnhancedFastContextAnalyzer;
    getCurrentAnalysis(): any;
    setCurrentAnalysis(analysis: any): void;
    getConfig(): FastContextConfig;
    setAnalyzing(analyzing: boolean): void;
    isCurrentlyAnalyzing(): boolean;
}
//# sourceMappingURL=repl.d.ts.map
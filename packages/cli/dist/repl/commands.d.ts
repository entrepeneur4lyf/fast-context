/**
 * REPL command implementations
 */
import type { FastContextRepl } from './repl.js';
export declare class ReplCommands {
    private repl;
    constructor(repl: FastContextRepl);
    analyze(args: string[]): Promise<void>;
    search(args: string[]): Promise<void>;
    dependencies(args: string[]): Promise<void>;
    patterns(args: string[]): Promise<void>;
    metrics(args: string[]): Promise<void>;
    export(args: string[]): Promise<void>;
    config(args: string[]): Promise<void>;
    help(args: string[]): void;
    private showGeneralHelp;
    private showCommandHelp;
    private parseSearchOptions;
    private parseDepthOption;
    private performSearch;
}
//# sourceMappingURL=commands.d.ts.map
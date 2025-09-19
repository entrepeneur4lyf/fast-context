/**
 * Progress tracking utilities for CLI operations
 */
import { Ora } from 'ora';
export interface ProgressTracker {
    update: (progress: ProgressUpdate) => void;
    setPhase: (phase: string) => void;
    finish: (message?: string) => void;
    fail: (message?: string) => void;
}
export interface ProgressUpdate {
    current: number;
    total: number;
    message?: string;
    phase?: string;
}
/**
 * Create a progress tracker that works with ora spinner
 */
export declare function createProgressTracker(spinner: Ora): ProgressTracker;
/**
 * Create a detailed progress bar for long-running operations
 */
export declare function createDetailedProgressBar(title: string, total: number): {
    update: (current: number, phase?: string) => void;
    setPhase: (phase: string) => void;
    finish: () => void;
};
/**
 * Create a multi-step progress tracker
 */
export declare function createMultiStepProgress(steps: string[]): {
    nextStep: () => void;
    updateCurrentStep: (message: string) => void;
    finish: (message?: string) => void;
    fail: (message?: string) => void;
};
/**
 * Create a progress tracker for file processing
 */
export declare function createFileProgressTracker(totalFiles: number): {
    startFile: (filename: string) => void;
    finishFile: () => void;
    setStatus: (status: string) => void;
    finish: () => void;
    fail: (error: string) => void;
};
/**
 * Create a memory usage tracker
 */
export declare function createMemoryTracker(): {
    update: () => void;
    getPeakUsage: () => number;
    getStartUsage: () => number;
    getCurrentUsage: () => number;
    formatMemory: (bytes: number) => string;
    getReport: () => {
        start: number;
        current: number;
        peak: number;
        increase: number;
        formatted: {
            start: string;
            current: string;
            peak: string;
            increase: string;
        };
    };
};
/**
 * Create a time tracker for performance monitoring
 */
export declare function createTimeTracker(): {
    startPhase: (name: string) => void;
    endPhase: () => void;
    getTotalDuration: () => number;
    getPhases: () => {
        [key: string]: {
            start: number;
            end?: number;
            duration?: number;
        };
    };
    getReport: () => {
        total: number;
        phases: {
            name: string;
            duration: number;
            percentage: number;
        }[];
        formatted: {
            total: string;
            phases: {
                duration: string;
                percentage: string;
                name: string;
            }[];
        };
    };
};
//# sourceMappingURL=progress.d.ts.map
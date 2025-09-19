/**
 * Progress tracking utilities for CLI operations
 */
import ora from 'ora';
import chalk from 'chalk';
import cliProgress from 'cli-progress';
/**
 * Create a progress tracker that works with ora spinner
 */
export function createProgressTracker(spinner) {
    let currentPhase = '';
    let lastUpdate = Date.now();
    return {
        update: (progress) => {
            const now = Date.now();
            // Throttle updates to avoid too frequent spinner changes
            if (now - lastUpdate < 100)
                return;
            lastUpdate = now;
            const percentage = Math.round((progress.current / progress.total) * 100);
            const phaseText = currentPhase ? `${currentPhase}: ` : '';
            const progressText = `${phaseText}${progress.message || 'Processing'} (${percentage}%)`;
            spinner.text = progressText;
        },
        setPhase: (phase) => {
            currentPhase = phase;
            spinner.text = `${phase}...`;
        },
        finish: (message) => {
            spinner.succeed(message || 'Completed');
        },
        fail: (message) => {
            spinner.fail(message || 'Failed');
        }
    };
}
/**
 * Create a detailed progress bar for long-running operations
 */
export function createDetailedProgressBar(title, total) {
    const progressBar = new cliProgress.SingleBar({
        format: `${chalk.cyan(title)} |${chalk.cyan('{bar}')}| {percentage}% | {value}/{total} | {phase} | ETA: {eta}s`,
        barCompleteChar: '█',
        barIncompleteChar: '░',
        hideCursor: true,
        clearOnComplete: false,
        stopOnComplete: true
    });
    progressBar.start(total, 0, { phase: 'Initializing' });
    return {
        update: (current, phase) => {
            progressBar.update(current, { phase: phase || 'Processing' });
        },
        setPhase: (phase) => {
            const current = progressBar.getProgress();
            progressBar.update(current, { phase });
        },
        finish: () => {
            progressBar.stop();
        }
    };
}
/**
 * Create a multi-step progress tracker
 */
export function createMultiStepProgress(steps) {
    let currentStep = 0;
    const spinner = ora(`Step 1/${steps.length}: ${steps[0]}`).start();
    return {
        nextStep: () => {
            if (currentStep < steps.length - 1) {
                currentStep++;
                spinner.text = `Step ${currentStep + 1}/${steps.length}: ${steps[currentStep]}`;
            }
        },
        updateCurrentStep: (message) => {
            spinner.text = `Step ${currentStep + 1}/${steps.length}: ${message}`;
        },
        finish: (message) => {
            spinner.succeed(message || `Completed all ${steps.length} steps`);
        },
        fail: (message) => {
            spinner.fail(message || `Failed at step ${currentStep + 1}: ${steps[currentStep]}`);
        }
    };
}
/**
 * Create a progress tracker for file processing
 */
export function createFileProgressTracker(totalFiles) {
    let processedFiles = 0;
    let currentFile = '';
    const spinner = ora('Preparing to process files...').start();
    return {
        startFile: (filename) => {
            currentFile = filename;
            const percentage = Math.round((processedFiles / totalFiles) * 100);
            spinner.text = `Processing ${filename} (${processedFiles}/${totalFiles} - ${percentage}%)`;
        },
        finishFile: () => {
            processedFiles++;
            const percentage = Math.round((processedFiles / totalFiles) * 100);
            spinner.text = `Processed ${currentFile} (${processedFiles}/${totalFiles} - ${percentage}%)`;
        },
        setStatus: (status) => {
            const percentage = Math.round((processedFiles / totalFiles) * 100);
            spinner.text = `${status} (${processedFiles}/${totalFiles} - ${percentage}%)`;
        },
        finish: () => {
            spinner.succeed(`Processed ${processedFiles} files`);
        },
        fail: (error) => {
            spinner.fail(`Failed processing ${currentFile}: ${error}`);
        }
    };
}
/**
 * Create a memory usage tracker
 */
export function createMemoryTracker() {
    const startMemory = process.memoryUsage();
    let peakMemory = startMemory.heapUsed;
    const tracker = {
        update: () => {
            const current = process.memoryUsage().heapUsed;
            if (current > peakMemory) {
                peakMemory = current;
            }
        },
        getPeakUsage: () => peakMemory,
        getStartUsage: () => startMemory.heapUsed,
        getCurrentUsage: () => process.memoryUsage().heapUsed,
        formatMemory: (bytes) => {
            const mb = bytes / 1024 / 1024;
            return `${mb.toFixed(1)}MB`;
        },
        getReport: () => {
            const current = process.memoryUsage();
            return {
                start: startMemory.heapUsed,
                current: current.heapUsed,
                peak: peakMemory,
                increase: current.heapUsed - startMemory.heapUsed,
                formatted: {
                    start: tracker.formatMemory(startMemory.heapUsed),
                    current: tracker.formatMemory(current.heapUsed),
                    peak: tracker.formatMemory(peakMemory),
                    increase: tracker.formatMemory(current.heapUsed - startMemory.heapUsed)
                }
            };
        }
    };
    return tracker;
}
/**
 * Create a time tracker for performance monitoring
 */
export function createTimeTracker() {
    const startTime = Date.now();
    const phases = {};
    let currentPhase = null;
    return {
        startPhase: (name) => {
            if (currentPhase) {
                phases[currentPhase].end = Date.now();
                phases[currentPhase].duration = phases[currentPhase].end - phases[currentPhase].start;
            }
            currentPhase = name;
            phases[name] = { start: Date.now() };
        },
        endPhase: () => {
            if (currentPhase) {
                phases[currentPhase].end = Date.now();
                phases[currentPhase].duration = phases[currentPhase].end - phases[currentPhase].start;
                currentPhase = null;
            }
        },
        getTotalDuration: () => Date.now() - startTime,
        getPhases: () => phases,
        getReport: () => {
            const total = Date.now() - startTime;
            const phaseReport = Object.entries(phases).map(([name, data]) => ({
                name,
                duration: data.duration || (Date.now() - data.start),
                percentage: ((data.duration || (Date.now() - data.start)) / total) * 100
            }));
            return {
                total,
                phases: phaseReport,
                formatted: {
                    total: `${total}ms`,
                    phases: phaseReport.map(p => ({
                        ...p,
                        duration: `${p.duration}ms`,
                        percentage: `${p.percentage.toFixed(1)}%`
                    }))
                }
            };
        }
    };
}
//# sourceMappingURL=progress.js.map
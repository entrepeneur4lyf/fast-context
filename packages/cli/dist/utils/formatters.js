/**
 * Output formatters for CLI results
 */
import chalk from 'chalk';
import Table from 'cli-table3';
import yaml from 'js-yaml';
import { markdownTable } from 'markdown-table';
export async function formatAnalysisResults(results, options) {
    // If JSON flag is set, always return JSON regardless of format option
    if (options.json) {
        return JSON.stringify(results, null, 2);
    }
    switch (options.format) {
        case 'json':
            return JSON.stringify(results, null, 2);
        case 'yaml':
            return yaml.dump(results, {
                indent: 2,
                lineWidth: 120,
                noRefs: true
            });
        case 'markdown':
            return formatAsMarkdown(results, options);
        case 'table':
        default:
            return formatAsTable(results, options);
    }
}
function formatAsTable(results, options) {
    const output = [];
    // Summary table
    const summaryTable = new Table({
        head: [chalk.cyan('Metric'), chalk.cyan('Value')],
        style: { head: [], border: [] }
    });
    summaryTable.push(['Files Analyzed', chalk.white(results.fileCount.toString())], ['Symbols Found', chalk.white(results.symbolCount.toString())], ['Relationships', chalk.white(results.relationshipCount.toString())], ['Languages', chalk.white(results.languages.join(', '))], ['Duration', chalk.white(`${results.durationMs}ms`)]);
    if (results.performance) {
        summaryTable.push(['Memory Usage', chalk.white(`${results.performance.memoryUsageMb}MB`)], ['CPU Usage', chalk.white(`${results.performance.cpuUsagePercent}%`)], ['Throughput', chalk.white(`${results.performance.throughputFilesPerSecond} files/sec`)]);
    }
    output.push(chalk.cyan('📊 Analysis Summary'));
    output.push(summaryTable.toString());
    // Symbols table (if verbose)
    if (options.verbose && results.symbols && results.symbols.length > 0) {
        output.push('');
        output.push(chalk.cyan('🔍 Top Symbols'));
        const symbolsTable = new Table({
            head: [chalk.cyan('Name'), chalk.cyan('Type'), chalk.cyan('File'), chalk.cyan('Line')],
            style: { head: [], border: [] }
        });
        results.symbols.slice(0, 10).forEach((symbol) => {
            symbolsTable.push([
                chalk.white(symbol.name),
                chalk.yellow(symbol.kind),
                chalk.gray(symbol.file),
                chalk.gray(symbol.line?.toString() || 'N/A')
            ]);
        });
        output.push(symbolsTable.toString());
    }
    // Insights (if available)
    if (results.insights && results.insights.length > 0) {
        output.push('');
        output.push(chalk.cyan('💡 Insights'));
        results.insights.forEach((insight, index) => {
            output.push(`${index + 1}. ${chalk.white(insight.title)}`);
            if (insight.description) {
                output.push(`   ${chalk.gray(insight.description)}`);
            }
        });
    }
    // Recommendations (if available)
    if (results.recommendations && results.recommendations.length > 0) {
        output.push('');
        output.push(chalk.cyan('🚀 Recommendations'));
        results.recommendations.forEach((rec, index) => {
            const priority = rec.priority === 'high' ? chalk.red('HIGH') :
                rec.priority === 'medium' ? chalk.yellow('MEDIUM') :
                    chalk.green('LOW');
            output.push(`${index + 1}. [${priority}] ${chalk.white(rec.title)}`);
            if (rec.description) {
                output.push(`   ${chalk.gray(rec.description)}`);
            }
        });
    }
    return output.join('\n');
}
function formatAsMarkdown(results, options) {
    const output = [];
    output.push('# Analysis Results');
    output.push('');
    // Summary table
    const summaryData = [
        ['Metric', 'Value'],
        ['Files Analyzed', results.fileCount.toString()],
        ['Symbols Found', results.symbolCount.toString()],
        ['Relationships', results.relationshipCount.toString()],
        ['Languages', results.languages.join(', ')],
        ['Duration', `${results.durationMs}ms`]
    ];
    if (results.performance) {
        summaryData.push(['Memory Usage', `${results.performance.memoryUsageMb}MB`], ['CPU Usage', `${results.performance.cpuUsagePercent}%`], ['Throughput', `${results.performance.throughputFilesPerSecond} files/sec`]);
    }
    output.push('## Summary');
    output.push('');
    output.push(markdownTable(summaryData));
    output.push('');
    // Symbols section
    if (options.verbose && results.symbols && results.symbols.length > 0) {
        output.push('## Top Symbols');
        output.push('');
        const symbolsData = [
            ['Name', 'Type', 'File', 'Line'],
            ...results.symbols.slice(0, 10).map((symbol) => [
                symbol.name,
                symbol.kind,
                symbol.file,
                symbol.line?.toString() || 'N/A'
            ])
        ];
        output.push(markdownTable(symbolsData));
        output.push('');
    }
    // Insights section
    if (results.insights && results.insights.length > 0) {
        output.push('## Insights');
        output.push('');
        results.insights.forEach((insight, index) => {
            output.push(`${index + 1}. **${insight.title}**`);
            if (insight.description) {
                output.push(`   ${insight.description}`);
            }
            output.push('');
        });
    }
    // Recommendations section
    if (results.recommendations && results.recommendations.length > 0) {
        output.push('## Recommendations');
        output.push('');
        results.recommendations.forEach((rec, index) => {
            const priority = rec.priority?.toUpperCase() || 'MEDIUM';
            output.push(`${index + 1}. **[${priority}] ${rec.title}**`);
            if (rec.description) {
                output.push(`   ${rec.description}`);
            }
            output.push('');
        });
    }
    return output.join('\n');
}
export function formatSymbolList(symbols, options) {
    if (options.json) {
        return JSON.stringify(symbols, null, 2);
    }
    switch (options.format) {
        case 'json':
            return JSON.stringify(symbols, null, 2);
        case 'yaml':
            return yaml.dump(symbols, { indent: 2 });
        case 'table':
        default:
            const table = new Table({
                head: [chalk.cyan('Name'), chalk.cyan('Type'), chalk.cyan('File'), chalk.cyan('Line')],
                style: { head: [], border: [] }
            });
            symbols.forEach((symbol) => {
                table.push([
                    chalk.white(symbol.name),
                    chalk.yellow(symbol.kind),
                    chalk.gray(symbol.file),
                    chalk.gray(symbol.line?.toString() || 'N/A')
                ]);
            });
            return table.toString();
    }
}
export function formatDependencyGraph(dependencies, options) {
    if (options.json) {
        return JSON.stringify(dependencies, null, 2);
    }
    // For table format, show a simplified dependency list
    const table = new Table({
        head: [chalk.cyan('Symbol'), chalk.cyan('Dependencies'), chalk.cyan('Dependents')],
        style: { head: [], border: [] }
    });
    dependencies.forEach((dep) => {
        table.push([
            chalk.white(dep.symbol),
            chalk.gray(dep.dependencies?.length || 0),
            chalk.gray(dep.dependents?.length || 0)
        ]);
    });
    return table.toString();
}
//# sourceMappingURL=formatters.js.map
"use strict";
/**
 * MCP Prompts Implementation for Fast-Context
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.registerPrompts = registerPrompts;
const core_1 = require("@fast-context/core");
/**
 * Register all Fast-Context prompts with the MCP server
 */
function registerPrompts(server) {
    // Prompt: Code Review
    server.registerPrompt('code-review', {
        title: 'Code Review',
        description: 'Generate comprehensive code review prompts with context and suggestions'
    }, async (args) => {
        const { filePath, focusAreas, severity = 'medium' } = args;
        try {
            const focusText = focusAreas?.length ?
                `Focus particularly on: ${focusAreas.join(', ')}` :
                'Provide a comprehensive review covering all aspects';
            return {
                messages: [{
                        role: 'user',
                        content: {
                            type: 'text',
                            text: `Please review the following code file with ${severity} severity level.

File: ${filePath}
${focusText}

Please provide:
1. Overall code quality assessment
2. Specific issues and improvements
3. Security considerations (if applicable)
4. Performance optimization opportunities
5. Maintainability suggestions
6. Code style and best practices feedback

Format your response with clear sections and actionable recommendations.`
                        }
                    }]
            };
        }
        catch (error) {
            return {
                messages: [{
                        role: 'user',
                        content: {
                            type: 'text',
                            text: `Please review the code file: ${filePath}

Note: Unable to load detailed analysis due to error: ${error instanceof Error ? error.message : String(error)}

Please provide a general code review focusing on:
- Code structure and organization
- Best practices adherence
- Potential improvements
- Security considerations
- Performance optimization opportunities`
                        }
                    }]
            };
        }
    });
    // Prompt: Refactoring Suggestions
    server.registerPrompt('refactoring-suggestions', {
        title: 'Refactoring Suggestions',
        description: 'Generate refactoring suggestions for specific symbols or code sections'
    }, async (args) => {
        const { symbolName, refactoringType, includeExamples = true } = args;
        try {
            const refactoringText = refactoringType ?
                `Focus on ${refactoringType} refactoring techniques` :
                'Consider all applicable refactoring techniques';
            const examplesText = includeExamples ?
                'Please include code examples for your suggestions' :
                'Provide conceptual suggestions without detailed code examples';
            return {
                messages: [{
                        role: 'user',
                        content: {
                            type: 'text',
                            text: `Please provide refactoring suggestions for the following symbol:

Symbol: ${symbolName}

${refactoringText}
${examplesText}

Please analyze and suggest:
1. Code structure improvements
2. Design pattern applications
3. Performance optimizations
4. Maintainability enhancements
5. Testability improvements
6. Dependency reduction opportunities

Consider the symbol's context, usage patterns, and relationships with other code components.`
                        }
                    }]
            };
        }
        catch (error) {
            return {
                messages: [{
                        role: 'user',
                        content: {
                            type: 'text',
                            text: `Please provide refactoring suggestions for the symbol: ${symbolName}

Note: Unable to load detailed analysis due to error: ${error instanceof Error ? error.message : String(error)}

Please provide general refactoring suggestions focusing on:
- Code structure and organization
- Design patterns that might be applicable
- Performance considerations
- Maintainability improvements
- Testing strategies`
                        }
                    }]
            };
        }
    });
    // Prompt: Architecture Analysis
    server.registerPrompt('architecture-analysis', {
        title: 'Architecture Analysis',
        description: 'Generate architectural analysis and recommendations for the codebase'
    }, async (args) => {
        const { projectPath, analysisType = 'overview', includeRecommendations = true } = args;
        try {
            const config = { projectRoot: projectPath };
            const analyzer = new core_1.EnhancedFastContextAnalyzer(config);
            // Get overall analysis
            const result = await analyzer.analyze();
            const analysisTypeText = {
                overview: 'Provide a comprehensive architectural overview',
                patterns: 'Focus on architectural and design patterns',
                dependencies: 'Analyze dependency structure and relationships',
                quality: 'Assess code quality and architectural health'
            }[analysisType];
            const recommendationsText = includeRecommendations ?
                'Include specific recommendations for improvements' :
                'Focus on analysis without detailed recommendations';
            return {
                messages: [{
                        role: 'user',
                        content: {
                            type: 'text',
                            text: `Please analyze the architecture of this codebase:

Project: ${projectPath}
Analysis Type: ${analysisType}

Project Summary:
- File Count: ${result.fileCount}
- Symbol Count: ${result.symbolCount}
- Languages: ${result.languages.join(', ')}
- Duration: ${result.durationMs}ms

${analysisTypeText}
${recommendationsText}

Please provide:
1. Architectural overview and structure
2. Component relationships and dependencies
3. Design patterns and architectural patterns
4. Code organization and modularity assessment
5. Scalability and maintainability analysis
6. Potential architectural improvements
7. Technical debt identification

Focus on providing actionable insights for improving the codebase architecture.`
                        }
                    }]
            };
        }
        catch (error) {
            return {
                messages: [{
                        role: 'user',
                        content: {
                            type: 'text',
                            text: `Please analyze the architecture of the codebase at: ${projectPath}

Note: Unable to load detailed analysis due to error: ${error instanceof Error ? error.message : String(error)}

Please provide a general architectural analysis focusing on:
- Overall code organization
- Component structure and relationships
- Design patterns and architectural patterns
- Scalability considerations
- Maintainability assessment
- Potential improvements`
                        }
                    }]
            };
        }
    });
    // Prompt: Documentation Generation
    server.registerPrompt('documentation-generation', {
        title: 'Documentation Generation',
        description: 'Generate comprehensive documentation for symbols and code components'
    }, async (args) => {
        const { symbolName, format = 'markdown', includeExamples = true } = args;
        try {
            const formatText = {
                jsdoc: 'Generate JSDoc-style documentation',
                markdown: 'Generate Markdown documentation',
                typescript: 'Generate TypeScript interface documentation'
            }[format];
            const examplesText = includeExamples ?
                'Include usage examples and code samples' :
                'Focus on API documentation without examples';
            return {
                messages: [{
                        role: 'user',
                        content: {
                            type: 'text',
                            text: `Please generate documentation for the following symbol:

Symbol: ${symbolName}

${formatText}
${examplesText}

Please include:
1. Clear description of purpose and functionality
2. Parameter documentation (if applicable)
3. Return value documentation (if applicable)
4. Usage guidelines and best practices
5. Related symbols and dependencies
6. Error handling information (if applicable)
7. Performance considerations (if relevant)

Ensure the documentation is comprehensive, accurate, and follows ${format} conventions.`
                        }
                    }]
            };
        }
        catch (error) {
            return {
                messages: [{
                        role: 'user',
                        content: {
                            type: 'text',
                            text: `Please generate ${format} documentation for the symbol: ${symbolName}

Note: Unable to load detailed analysis due to error: ${error instanceof Error ? error.message : String(error)}

Please provide general documentation including:
- Purpose and functionality description
- Usage guidelines
- Parameter and return value information
- Best practices and considerations`
                        }
                    }]
            };
        }
    });
}
//# sourceMappingURL=prompts.js.map
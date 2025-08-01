#!/usr/bin/env node

const { FastContextAnalyzer } = require('./index.js');

async function testCppSupport() {
    console.log('🧪 Testing C++ Support');
    
    try {
        // Create analyzer
        const config = {
            projectRoot: process.cwd(),
            languages: ['cpp'],
            ignorePatterns: ['node_modules/**', 'target/**', '.git/**'],
            enableCaching: true,
            maxFiles: 100,
            parallelProcessing: true
        };
        
        const analyzer = new FastContextAnalyzer(config);
        console.log('✅ Analyzer created');
        
        // Create a test C++ file
        const fs = require('fs');
        const testCppContent = `
#include <iostream>
#include <vector>
#include <string>

namespace math {
    template<typename T>
    class Calculator {
    private:
        T value;
        std::vector<T> history;
        
    public:
        Calculator(T initial = 0) : value(initial) {}
        
        T add(T operand) {
            value += operand;
            history.push_back(value);
            return value;
        }
        
        T subtract(T operand) {
            value -= operand;
            history.push_back(value);
            return value;
        }
        
        T getValue() const {
            return value;
        }
        
        void reset() {
            value = T{};
            history.clear();
        }
    };
    
    union Number {
        int intValue;
        float floatValue;
        double doubleValue;
    };
    
    struct Point {
        double x, y;
        Point(double x = 0, double y = 0) : x(x), y(y) {}
    };
}

int main() {
    math::Calculator<int> calc(10);
    calc.add(5);
    calc.subtract(3);
    
    math::Point origin;
    math::Number num;
    num.intValue = 42;
    
    std::cout << "Result: " << calc.getValue() << std::endl;
    return 0;
}
`;
        
        fs.writeFileSync('test_calculator.cpp', testCppContent);
        console.log('✅ Test C++ file created');
        
        // Analyze project
        await analyzer.analyze();
        console.log('✅ Analysis completed');
        
        // Test symbol extraction
        const symbols = analyzer.findSymbols('Calculator');
        console.log(`\n🔍 Found symbols result:`, typeof symbols, symbols);

        if (!symbols || !Array.isArray(symbols.symbols)) {
            console.log('❌ Symbols result is not an array, got:', symbols);
            return;
        }

        console.log(`\n🔍 Found ${symbols.symbols.length} Calculator symbols:`);

        symbols.symbols.forEach(symbol => {
            console.log(`  📦 ${symbol.name} (${symbol.kind}) in ${symbol.filePath}:${symbol.startLine}`);
            if (symbol.signature) {
                console.log(`     Signature: ${symbol.signature}`);
            }
            if (symbol.modifiers && symbol.modifiers.length > 0) {
                console.log(`     Modifiers: ${symbol.modifiers.join(', ')}`);
            }
        });
        
        // Test namespace symbols
        const namespaceSymbols = analyzer.findSymbols('math');
        console.log(`\n🔍 Found ${namespaceSymbols.symbols.length} math namespace symbols:`);

        // Test method symbols
        const methodSymbols = analyzer.findSymbols('add');
        console.log(`\n🔍 Found ${methodSymbols.symbols.length} add method symbols:`);

        // Test include symbols
        const includeSymbols = analyzer.findSymbols('iostream');
        console.log(`\n🔍 Found ${includeSymbols.symbols.length} iostream include symbols:`);

        // Test union symbols
        const unionSymbols = analyzer.findSymbols('Number');
        console.log(`\n🔍 Found ${unionSymbols.symbols.length} Number union symbols:`);

        // Test struct symbols
        const structSymbols = analyzer.findSymbols('Point');
        console.log(`\n🔍 Found ${structSymbols.symbols.length} Point struct symbols:`);

        // Show some examples
        if (methodSymbols.symbols.length > 0) {
            console.log(`\n📝 Example add method: ${methodSymbols.symbols[0].qualifiedName}`);
        }

        if (structSymbols.symbols.length > 0) {
            console.log(`📝 Example struct: ${structSymbols.symbols[0].qualifiedName}`);
        }
        
        // Clean up
        fs.unlinkSync('test_calculator.cpp');
        console.log('\n✅ Test file cleaned up');
        
        console.log('\n🎉 C++ support test completed successfully!');
        
        // Summary
        const totalSymbols = symbols.symbols.length + namespaceSymbols.symbols.length + methodSymbols.symbols.length +
                           includeSymbols.symbols.length + unionSymbols.symbols.length + structSymbols.symbols.length;
        console.log(`📊 Total C++ symbols found: ${totalSymbols}`);
        
    } catch (error) {
        console.error('❌ C++ support test failed:', error.message);
        process.exit(1);
    }
}

testCppSupport();

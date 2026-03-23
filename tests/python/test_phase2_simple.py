#!/usr/bin/env python3
import tempfile
import os
import sys
sys.path.insert(0, '/home/shawn/workspace/0-projects/rustworkx-nodejs/python')

import fast_context

def test_phase2():
    """Test Phase 2: Core Analysis Engine Integration"""
    
    # Create a test project with some Python files
    with tempfile.TemporaryDirectory() as tmpdir:
        # Create a simple Python file
        test_file = os.path.join(tmpdir, 'test.py')
        with open(test_file, 'w') as f:
            f.write('''
def hello():
    return "Hello World"

class TestClass:
    def method(self):
        pass

def complex_function():
    if True:
        for i in range(10):
            if i % 2 == 0:
                print(i)
            else:
                continue
    return None
''')
        
        # Test the analyzer
        config = fast_context.AnalyzerConfig(tmpdir)
        analyzer = fast_context.FastContextAnalyzer.from_config(config)
        
        print('Testing Phase 2: Core Analysis Engine Integration...')
        
        # Test basic analysis function
        result = fast_context.analyze_project(tmpdir)
        print(f'Analysis completed: {result.file_count} files, {result.symbol_count} symbols')
        
        # Test symbol extraction by kind
        functions = fast_context.find_symbols_by_kind(tmpdir, 'function')
        print(f'Found {len(functions)} functions')
        
        # Test file-based symbol extraction
        symbols = fast_context.find_symbols_in_file(test_file)
        print(f'Found {len(symbols)} symbols in test file')
        
        # Test graph creation
        graph = fast_context.Graph()
        node1 = graph.add_node('Function1')
        node2 = graph.add_node('Function2')
        print(f'Created graph with {graph.node_count()} nodes')
        
        # Test graph algorithms
        if graph.node_count() >= 2:
            graph.add_edge(node1, node2, 1.0)
            path_result = graph.dijkstra(node1, node2)
            print(f'Shortest path distance: {path_result.distance}')
        
        # Test dependencies
        deps = fast_context.find_dependencies(tmpdir, 'hello')
        print(f'Found {len(deps)} dependencies for hello function')
        
        # Test complex symbols
        complex_symbols = fast_context.find_complex_symbols(tmpdir, 5)
        print(f'Found {len(complex_symbols)} complex symbols')
        
        print('Phase 2 Core Analysis Engine Integration: ✓ PASSED')

if __name__ == '__main__':
    test_phase2()

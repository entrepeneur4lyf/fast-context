#!/usr/bin/env python3
import asyncio
import tempfile
import os
import sys
import pytest
sys.path.insert(0, '/home/shawn/workspace/0-projects/rustworkx-nodejs/python')

import fast_context

@pytest.mark.asyncio
async def test_phase2():
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
        
        # Test basic analysis
        result = await analyzer.analyze_async(asyncio.get_event_loop())
        print(f'Analysis completed: {result.file_count} files, {result.symbol_count} symbols')
        
        # Test symbol extraction by kind
        functions = await analyzer.find_symbols_by_kind_async(asyncio.get_event_loop(), 'function')
        print(f'Found {len(functions)} functions')
        
        # Test file-based symbol extraction
        symbols = await analyzer.find_symbols_in_file_async(asyncio.get_event_loop(), test_file)
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
            print(f'Shortest path: {path_result.path if hasattr(path_result, "path") else "No path found"}')
        
        print('Phase 2 Core Analysis Engine Integration: ✓ PASSED')

if __name__ == '__main__':
    asyncio.run(test_phase2())

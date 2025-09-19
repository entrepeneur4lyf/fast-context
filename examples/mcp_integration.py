#!/usr/bin/env python3
"""
MCP Server Integration Example

This example demonstrates how to use Fast-Context with the MCP (Model Context Protocol) server
for AI assistant integration and real-time code analysis.
"""

import asyncio
import json
import sys
import os
from pathlib import Path

# Add the fast_context module to the path
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    import fast_context
    from fast_context.mcp_server import mcp, analyze_codebase, extract_symbols, analyze_graph
    print("✅ Fast-Context MCP server imported successfully")
except ImportError as e:
    print(f"❌ Failed to import MCP server: {e}")
    sys.exit(1)

async def demonstrate_mcp_tools():
    """Demonstrate MCP server tools"""
    print("\n🛠️  MCP Server Tools")
    print("-" * 40)
    
    # Create a temporary project for testing
    import tempfile
    import shutil
    
    with tempfile.TemporaryDirectory() as temp_dir:
        project_dir = Path(temp_dir) / "test_project"
        project_dir.mkdir()
        
        # Create sample files
        (project_dir / "main.py").write_text("""
import asyncio
from typing import List

class DataProcessor:
    def __init__(self, data: List[str]):
        self.data = data
    
    async def process(self) -> List[str]:
        return [item.upper() for item in self.data]
    
    def get_stats(self) -> dict:
        return {
            'count': len(self.data),
            'unique': len(set(self.data))
        }

async def main():
    processor = DataProcessor(['hello', 'world', 'test'])
    result = await processor.process()
    print(result)
    
if __name__ == "__main__":
    asyncio.run(main())
""")
        
        (project_dir / "config.json").write_text("""
{
    "version": "1.0.0",
    "settings": {
        "debug": true,
        "max_workers": 4
    }
}
""")
        
        print(f"📁 Created test project at: {project_dir}")
        
        # Test codebase analysis
        try:
            result = await analyze_codebase(
                str(project_dir),
                languages=["python", "json"],
                max_files=100
            )
            print(f"📊 Codebase analysis completed")
            print(f"   Result type: {type(result)}")
            
        except Exception as e:
            print(f"❌ Codebase analysis failed: {e}")
        
        # Test symbol extraction
        try:
            symbols = await extract_symbols(str(project_dir / "main.py"))
            print(f"🔍 Symbol extraction completed")
            print(f"   Found {len(symbols) if hasattr(symbols, '__len__') else 'multiple'} symbols")
            
        except Exception as e:
            print(f"❌ Symbol extraction failed: {e}")
        
        # Test graph analysis
        try:
            graph_result = await analyze_graph(str(project_dir))
            print(f"📈 Graph analysis completed")
            print(f"   Result type: {type(graph_result)}")
            
        except Exception as e:
            print(f"❌ Graph analysis failed: {e}")

def demonstrate_mcp_resources():
    """Demonstrate MCP server resources"""
    print("\n📚 MCP Server Resources")
    print("-" * 40)
    
    # Note: These would typically be accessed via MCP protocol
    # For this example, we'll just show the structure
    resources = [
        "code://{project_path}/symbols",
        "graph://{nodes}/{edges}/analysis",
        "fast-context://analysis/sessions",
        "fast-context://graphs/registry",
        "fast-context://performance/metrics"
    ]
    
    print("Available MCP resources:")
    for resource in resources:
        print(f"   📄 {resource}")

def demonstrate_mcp_prompts():
    """Demonstrate MCP server prompts"""
    print("\n💬 MCP Server Prompts")
    print("-" * 40)
    
    prompts = [
        "code-review",
        "refactoring-suggestions", 
        "architecture-analysis",
        "performance-optimization"
    ]
    
    print("Available MCP prompts:")
    for prompt in prompts:
        print(f"   💭 {prompt}")

async def demonstrate_async_operations():
    """Demonstrate async MCP operations"""
    print("\n⚡ Async Operations")
    print("-" * 40)
    
    try:
        # Create sample data for async processing
        import tempfile
        from pathlib import Path
        
        with tempfile.TemporaryDirectory() as temp_dir:
            # Create multiple files for batch processing
            for i in range(3):
                file_path = Path(temp_dir) / f"file_{i}.py"
                file_path.write_text(f"""
def function_{i}():
    return "result_{i}"

class Class_{i}:
    def method_{i}(self):
        return f"class_method_{i}"
""")
            
            # Test async symbol extraction
            print("🔄 Processing multiple files asynchronously...")
            
            # Simulate async processing (in real MCP server, this would be actual async)
            import time
            start_time = time.time()
            
            # In real implementation, this would use asyncio.gather
            results = []
            for i in range(3):
                file_path = Path(temp_dir) / f"file_{i}.py"
                # Simulate async work
                await asyncio.sleep(0.1)
                results.append(f"Processed {file_path.name}")
            
            end_time = time.time()
            print(f"✅ Processed {len(results)} files in {end_time - start_time:.2f}s")
            
    except Exception as e:
        print(f"❌ Async operations failed: {e}")

async def main():
    """Main function"""
    print("🚀 Fast-Context MCP Server Integration Example")
    print("=" * 60)
    
    # Demonstrate MCP functionality
    await demonstrate_mcp_tools()
    demonstrate_mcp_resources()
    demonstrate_mcp_prompts()
    await demonstrate_async_operations()
    
    print("\n🎉 MCP Server Example completed successfully!")
    print("\n💡 Next steps:")
    print("   - Start the MCP server: python -m fast_context.mcp_server")
    print("   - Connect via MCP client")
    print("   - Explore real-time code analysis")

if __name__ == "__main__":
    asyncio.run(main())
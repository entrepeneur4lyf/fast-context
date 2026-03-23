#!/usr/bin/env python3
"""
Test script to isolate the bounds checking issue
"""
import asyncio
import tempfile
import os
from pathlib import Path
import sys

# Add the project to Python path
project_root = Path(__file__).parent
sys.path.insert(0, str(project_root / "python"))

try:
    from fast_context import FastContextAnalyzer
    print("✅ FastContextAnalyzer imported successfully")
except Exception as e:
    print(f"❌ Failed to import FastContextAnalyzer: {e}")
    sys.exit(1)

async def test_simple_analysis():
    """Test analysis with a simple Python file"""
    with tempfile.TemporaryDirectory() as temp_dir:
        # Create a simple Python file
        test_file = Path(temp_dir) / "test.py"
        test_file.write_text("""
def hello():
    print("Hello, world!")

if __name__ == "__main__":
    hello()
""")
        
        print(f"📁 Created test file: {test_file}")
        
        try:
            # Create analyzer
            analyzer = FastContextAnalyzer(str(temp_dir))
            print("✅ Analyzer created successfully")
            
            # Run analysis
            print("🔍 Starting analysis...")
            result = await analyzer.analyze_async()
            print("✅ Analysis completed successfully")
            print(f"📊 Result: {result}")
            
        except Exception as e:
            print(f"❌ Analysis failed: {e}")
            import traceback
            traceback.print_exc()

if __name__ == "__main__":
    asyncio.run(test_simple_analysis())

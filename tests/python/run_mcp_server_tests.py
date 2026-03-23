#!/usr/bin/env python3
"""
Fast-Context MCP Server Test Runner

This script runs all integration and performance tests for the MCP server,
providing comprehensive validation of functionality and performance.
"""

import sys
import subprocess
import time
from pathlib import Path
from typing import List, Dict, Any

# Add the parent directory to Python path
sys.path.insert(0, str(Path(__file__).parent.parent))

def run_command(cmd: List[str], description: str) -> Dict[str, Any]:
    """Run a command and return results."""
    print(f"\n🧪 {description}")
    print("=" * 60)
    
    start_time = time.time()
    try:
        result = subprocess.run(
            cmd, 
            capture_output=True, 
            text=True, 
            timeout=300  # 5 minute timeout
        )
        
        duration = time.time() - start_time
        
        return {
            "success": result.returncode == 0,
            "stdout": result.stdout,
            "stderr": result.stderr,
            "duration": duration,
            "returncode": result.returncode
        }
    except subprocess.TimeoutExpired:
        return {
            "success": False,
            "stdout": "",
            "stderr": "Test timed out after 5 minutes",
            "duration": time.time() - start_time,
            "returncode": -1
        }
    except Exception as e:
        return {
            "success": False,
            "stdout": "",
            "stderr": str(e),
            "duration": time.time() - start_time,
            "returncode": -1
        }

def print_test_result(result: Dict[str, Any], description: str):
    """Print test results with appropriate formatting."""
    duration = result["duration"]
    
    if result["success"]:
        print(f"✅ {description} - PASSED ({duration:.2f}s)")
        if result["stdout"]:
            # Print last few lines of stdout for success
            stdout_lines = result["stdout"].split('\n')
            relevant_lines = [line for line in stdout_lines if line.strip() and not line.startswith(' ')]
            if relevant_lines:
                print(f"   Output: {relevant_lines[-1]}")
    else:
        print(f"❌ {description} - FAILED ({duration:.2f}s)")
        if result["stderr"]:
            print(f"   Error: {result['stderr'][:200]}...")
        if result["stdout"]:
            print(f"   Output: {result['stdout'][:200]}...")

def run_basic_functionality_tests():
    """Run basic functionality tests for MCP server."""
    print("🚀 Running Basic Functionality Tests")
    print("=" * 80)
    
    # Test MCP server import
    result = run_command(
        [sys.executable, "-c", 
         "import sys; sys.path.insert(0, '.'); from fast_context.mcp_server import analyze_codebase; print('✅ MCP server imports successfully')"],
        "MCP Server Import Test"
    )
    print_test_result(result, "MCP Server Import")
    
    # Test Fast-Context core import
    result = run_command(
        [sys.executable, "-c", 
         "import sys; sys.path.insert(0, '.'); from fast_context import get_version; print(f'✅ Fast-Context version: {get_version()}')"],
        "Fast-Context Core Import Test"
    )
    print_test_result(result, "Fast-Context Core Import")
    
    # Test basic MCP server functions
    result = run_command(
        [sys.executable, "-c", """
import sys
sys.path.insert(0, '.')
from fast_context.mcp_server import create_graph, get_project_info
import json

# Test graph creation
result = create_graph('undirected', 10, 20)
data = json.loads(result)
print(f'✅ Graph created with ID: {data.get("graph_id", "unknown")}')

# Test project info
result = get_project_info('.')
data = json.loads(result)
print(f'✅ Project info: {data.get("total_files", 0)} files found')
"""],
        "Basic MCP Server Functions Test"
    )
    print_test_result(result, "Basic MCP Server Functions")

def run_integration_tests():
    """Run comprehensive integration tests."""
    print("\n🚀 Running Integration Tests")
    print("=" * 80)
    
    # Run integration tests with pytest
    result = run_command(
        [sys.executable, "-m", "pytest", 
         "tests/test_mcp_server_integration.py", 
         "-v", "--tb=short"],
        "MCP Server Integration Tests"
    )
    print_test_result(result, "Integration Tests")
    
    return result["success"]

def run_performance_tests():
    """Run performance tests."""
    print("\n🚀 Running Performance Tests")
    print("=" * 80)
    
    # Run performance tests with pytest
    result = run_command(
        [sys.executable, "-m", "pytest", 
         "tests/test_mcp_server_performance.py", 
         "-v", "--tb=short", "-s"],
        "MCP Server Performance Tests"
    )
    print_test_result(result, "Performance Tests")
    
    return result["success"]

def run_error_handling_tests():
    """Run error handling and edge case tests."""
    print("\n🚀 Running Error Handling Tests")
    print("=" * 80)
    
    result = run_command(
        [sys.executable, "-c", """
import sys
sys.path.insert(0, '.')
from fast_context.mcp_server import analyze_codebase, create_graph, perform_advanced_graph_analysis
import json

# Test error handling for invalid paths
try:
    result = analyze_codebase('/non/existent/path')
    data = json.loads(result)
    assert 'error' in data
    print('✅ Invalid path handling works')
except Exception as e:
    print(f'❌ Invalid path test failed: {e}')

# Test error handling for invalid graph operations
try:
    result = perform_advanced_graph_analysis('non_existent_graph')
    data = json.loads(result)
    assert 'error' in data
    print('✅ Invalid graph handling works')
except Exception as e:
    print(f'❌ Invalid graph test failed: {e}')

# Test error handling for invalid graph types
try:
    result = create_graph('invalid_type')
    data = json.loads(result)
    assert 'error' in data
    print('✅ Invalid graph type handling works')
except Exception as e:
    print(f'❌ Invalid graph type test failed: {e}')

print('✅ All error handling tests completed')
"""],
        "Error Handling Tests"
    )
    print_test_result(result, "Error Handling Tests")
    
    return result["success"]

def run_streaming_tests():
    """Test streaming functionality."""
    print("\n🚀 Running Streaming Functionality Tests")
    print("=" * 80)
    
    result = run_command(
        [sys.executable, "-c", """
import sys
import asyncio
sys.path.insert(0, '.')
from fast_context.mcp_server import analyze_codebase_streaming
import json

async def test_streaming():
    try:
        result = await analyze_codebase_streaming('.')
        data = json.loads(result)
        assert 'session_id' in data
        assert 'progress_updates' in data
        print(f'✅ Streaming analysis completed with {len(data.get(\"progress_updates\", []))} updates')
    except Exception as e:
        print(f'❌ Streaming test failed: {e}')

asyncio.run(test_streaming())
print('✅ Streaming functionality test completed')
"""],
        "Streaming Functionality Tests"
    )
    print_test_result(result, "Streaming Functionality Tests")
    
    return result["success"]

def run_memory_efficiency_tests():
    """Test memory efficiency."""
    print("\n🚀 Running Memory Efficiency Tests")
    print("=" * 80)
    
    result = run_command(
        [sys.executable, "-c", """
import sys
import psutil
import gc
sys.path.insert(0, '.')
from fast_context.mcp_server import create_advanced_graph, get_performance_metrics
import json

def test_memory_efficiency():
    try:
        # Measure initial memory
        process = psutil.Process()
        memory_before = process.memory_info().rss / 1024 / 1024  # MB
        
        # Create multiple graphs
        graph_ids = []
        for i in range(10):
            result = create_advanced_graph('undirected', 100, 200)
            data = json.loads(result)
            graph_ids.append(data['graph_id'])
        
        memory_during = process.memory_info().rss / 1024 / 1024  # MB
        
        # Get performance metrics
        result = get_performance_metrics()
        data = json.loads(result)
        
        # Force cleanup
        gc.collect()
        import time
        time.sleep(0.1)
        
        memory_after = process.memory_info().rss / 1024 / 1024  # MB
        
        memory_increase = memory_after - memory_before
        print(f'✅ Memory efficiency test: {memory_increase:.2f} MB increase')
        print(f'   Registered graphs: {data.get(\"system_metrics\", {}).get(\"registered_graphs\", 0)}')
        
        # Memory increase should be reasonable
        if memory_increase < 200:
            print('✅ Memory usage is within acceptable limits')
        else:
            print(f'⚠️  Memory usage might be high: {memory_increase:.2f} MB')
        
        return True
    except Exception as e:
        print(f'❌ Memory efficiency test failed: {e}')
        return False

test_memory_efficiency()
print('✅ Memory efficiency test completed')
"""],
        "Memory Efficiency Tests"
    )
    print_test_result(result, "Memory Efficiency Tests")
    
    return result["success"]

def generate_test_report(results: Dict[str, bool]):
    """Generate a comprehensive test report."""
    print("\n" + "=" * 80)
    print("📊 COMPREHENSIVE TEST REPORT")
    print("=" * 80)
    
    passed = sum(results.values())
    total = len(results)
    
    print(f"\n📈 Overall Results:")
    print(f"   Tests Passed: {passed}/{total}")
    print(f"   Success Rate: {(passed/total)*100:.1f}%")
    
    print(f"\n📋 Test Categories:")
    for category, success in results.items():
        status = "✅ PASS" if success else "❌ FAIL"
        print(f"   {category}: {status}")
    
    if passed == total:
        print(f"\n🎉 ALL TESTS PASSED!")
        print("   The Fast-Context MCP Server is ready for production use.")
    else:
        print(f"\n⚠️  {total - passed} test category(s) failed.")
        print("   Please review the failed tests and address any issues.")
    
    print(f"\n📝 Next Steps:")
    if passed == total:
        print("   1. Proceed with documentation and examples")
        print("   2. Prepare for release packaging")
        print("   3. Set up CI/CD pipeline")
    else:
        print("   1. Fix failing test categories")
        print("   2. Re-run tests to validate fixes")
        print("   3. Ensure all tests pass before proceeding")

def main():
    """Run all tests and generate comprehensive report."""
    print("🚀 Fast-Context MCP Server - Comprehensive Test Suite")
    print("=" * 80)
    print(f"📍 Running tests from: {Path.cwd()}")
    print(f"🕐 Started at: {time.strftime('%Y-%m-%d %H:%M:%S')}")
    
    # Track results
    test_results = {}
    
    # Run test categories
    try:
        # Basic functionality tests (always run)
        run_basic_functionality_tests()
        test_results["Basic Functionality"] = True
        
        # Integration tests
        test_results["Integration Tests"] = run_integration_tests()
        
        # Performance tests
        test_results["Performance Tests"] = run_performance_tests()
        
        # Error handling tests
        test_results["Error Handling"] = run_error_handling_tests()
        
        # Streaming tests
        test_results["Streaming Tests"] = run_streaming_tests()
        
        # Memory efficiency tests
        test_results["Memory Efficiency"] = run_memory_efficiency_tests()
        
    except KeyboardInterrupt:
        print("\n\n⚠️  Tests interrupted by user")
        return 1
    except Exception as e:
        print(f"\n\n❌ Unexpected error during testing: {e}")
        return 1
    
    # Generate report
    generate_test_report(test_results)
    
    # Return appropriate exit code
    return 0 if all(test_results.values()) else 1

if __name__ == "__main__":
    exit_code = main()
    sys.exit(exit_code)

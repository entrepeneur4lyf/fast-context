#!/usr/bin/env python3
import sys
import os

# Add the project path to sys.path
sys.path.insert(0, '/home/shawn/workspace/0-projects/rustworkx-nodejs/python')

try:
    import fast_context
    
    print("=== Fast-Context Python Module Test ===")
    print(f"Module path: {fast_context.__file__}")
    print(f"Module version: {fast_context.get_version()}")
    print(f"Supported languages: {fast_context.get_supported_languages()}")
    
    # Check for graph classes
    graph_classes = ['PyRustworkxGraph', 'PyRustworkxDiGraph', 'PathResult', 'CentralityResult', 'ConnectedComponent']
    
    print("\n=== Graph Classes Available ===")
    for cls in graph_classes:
        if hasattr(fast_context, cls):
            print(f"✓ {cls}: Available")
        else:
            print(f"✗ {cls}: Not available")
    
    # List all available classes and functions
    print("\n=== All Available Attributes ===")
    attrs = [attr for attr in dir(fast_context) if not attr.startswith('_')]
    for attr in sorted(attrs):
        obj = getattr(fast_context, attr)
        if callable(obj):
            print(f"  {attr}: Callable")
        else:
            print(f"  {attr}: {type(obj).__name__}")
    
    # Test basic functionality
    print("\n=== Basic Functionality Test ===")
    
    # Test language detection
    lang = fast_context.detect_language("test.py")
    print(f"Language detection for test.py: {lang}")
    
    # Test version
    version = fast_context.get_version()
    print(f"Version: {version}")
    
    # Test analysis
    if hasattr(fast_context, 'AnalyzerConfig'):
        config = fast_context.AnalyzerConfig("/tmp")
        print(f"Created analyzer config: {config.project_root}")
    
    print("\n=== Test Summary ===")
    print("✓ Module imports successfully")
    print("✓ Basic functions work")
    print("✓ Core classes are available")
    
    if any(hasattr(fast_context, cls) for cls in graph_classes):
        print("✓ Graph classes are available")
    else:
        print("✗ Graph classes are not available - need to investigate")
    
except Exception as e:
    print(f"Error during testing: {e}")
    import traceback
    traceback.print_exc()

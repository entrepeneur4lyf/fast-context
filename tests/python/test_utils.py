"""
Basic unit tests for fast_context utility functions.
"""

import pytest
from fast_context import (
    get_version,
    get_supported_languages,
    detect_language
)

def test_get_version():
    """Test version retrieval."""
    version = get_version()
    assert isinstance(version, str)
    assert len(version) > 0
    # Should be in semantic version format
    assert any(char.isdigit() for char in version)

def test_get_supported_languages():
    """Test supported languages retrieval."""
    languages = get_supported_languages()
    assert isinstance(languages, list)
    assert len(languages) > 0
    # Should include common languages
    languages_lower = [lang.lower() for lang in languages]
    assert "rust" in languages_lower
    assert "python" in languages_lower
    assert "javascript" in languages_lower

def test_detect_language():
    """Test language detection."""
    # Test common file extensions (only supported ones)
    test_cases = [
        ("main.rs", "rust"),
        ("app.py", "python"),
        ("script.js", "javascript"),
        ("style.css", "css"),
        ("index.html", "html"),
        ("config.json", "json"),
        ("data.yaml", "yaml"),
        ("data.yml", "yaml"),
        ("main.go", "go"),
        ("main.cpp", "cpp"),
        ("main.java", "java"),
        ("unknown.xyz", None),
        ("", None),
    ]
    
    for filename, expected in test_cases:
        result = detect_language(filename)
        if expected is None:
            assert result is None, f"Expected None for {filename}, got {result}"
        else:
            assert result is not None, f"Expected {expected} for {filename}, got None"
            assert expected.lower() in result.lower(), f"Expected {expected} in {result} for {filename}"

def test_detect_language_case_insensitive():
    """Test language detection is case insensitive."""
    extensions = [".py", ".RS", ".JS", ".CPP", ".HTML"]
    for ext in extensions:
        result = detect_language(f"test{ext}")
        assert result is not None, f"Should detect language for {ext}"

def test_detect_language_with_path():
    """Test language detection with full paths."""
    test_cases = [
        ("/path/to/main.rs", "rust"),
        ("./src/app.py", "python"),
        ("../js/script.js", "javascript"),
        ("C:\\project\\main.go", "go"),
        ("nested/config.json", "json"),
        ("data/settings.yaml", "yaml"),
    ]
    
    for filepath, expected in test_cases:
        result = detect_language(filepath)
        if expected is not None:
            assert result is not None
            assert expected.lower() in result.lower()
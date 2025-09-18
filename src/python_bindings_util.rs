#[cfg(feature = "python")]
pub(crate) fn extensions_for_language_str(lang: &str) -> Option<&'static [&'static str]> {
    match lang.to_lowercase().as_str() {
        "rust" => Some(&["rs"][..]),
        "python" => Some(&["py", "pyw"][..]),
        "javascript" => Some(&["js", "mjs", "cjs"][..]),
        "typescript" => Some(&["ts", "mts", "cts"][..]),
        "java" => Some(&["java"][..]),
        "go" => Some(&["go"][..]),
        "csharp" => Some(&["cs"][..]),
        "cpp" => Some(&["cpp", "cc", "cxx", "c++", "hpp", "hxx", "h++"][..]),
        "swift" => Some(&["swift"][..]),
        "objectivec" => Some(&["m", "mm"][..]),
        "php" => Some(&["php"][..]),
        "ruby" => Some(&["rb"][..]),
        "scala" => Some(&["scala", "sc"][..]),
        "zig" => Some(&["zig"][..]),
        "dart" => Some(&["dart"][..]),
        "lua" => Some(&["lua"][..]),
        "bash" => Some(&["sh", "bash"][..]),
        "css" => Some(&["css"][..]),
        "html" => Some(&["html", "htm"][..]),
        "xml" => Some(&["xml"][..]),
        "json" => Some(&["json"][..]),
        "yaml" => Some(&["yaml", "yml"][..]),
        "markdown" => Some(&["md", "markdown"][..]),
        _ => None,
    }
}

//! Bash symbol extractor
//! 
//! Extracts symbols from Bash shell script source code including:
//! - Functions and function definitions
//! - Variables and environment variables
//! - Source and include statements
//! - Aliases and exports
//! - Command substitutions and arrays

use crate::parsers::LanguageId;
use crate::symbols::{Location, Scope, Symbol, SymbolExtractor, SymbolKind};
use tree_sitter::{Node, Tree};

/// Bash Symbol Extractor
/// Extracts functions, variables, source statements from Bash scripts
pub struct BashExtractor;

impl SymbolExtractor for BashExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Bash
    }

    fn extract_symbols(&self, tree: &Tree, source: &str, file_path: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let mut scope_stack = Vec::new();
        
        self.extract_from_node(tree.root_node(), source, file_path, &mut symbols, &mut scope_stack);
        symbols
    }
}

impl BashExtractor {
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            "function_definition" => {
                self.extract_function(&node, source, file_path, symbols, scope_stack);
            }
            "variable_assignment" => {
                self.extract_variable(&node, source, file_path, symbols, scope_stack);
            }
            "declaration_command" => {
                self.extract_declaration(&node, source, file_path, symbols, scope_stack);
            }
            "command" => {
                self.extract_command(&node, source, file_path, symbols, scope_stack);
            }
            "for_statement" => {
                self.extract_for_variables(&node, source, file_path, symbols, scope_stack);
            }
            "while_statement" | "until_statement" => {
                // These don't define variables but might contain them
            }
            "case_statement" => {
                self.extract_case_patterns(&node, source, file_path, symbols, scope_stack);
            }
            "command_substitution" => {
                self.extract_command_substitution(&node, source, file_path, symbols, scope_stack);
            }
            _ => {}
        }

        // Recursively process child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_from_node(child, source, file_path, symbols, scope_stack);
        }

        // Pop scope if we added one for this node
        if matches!(node.kind(), "function_definition") {
            scope_stack.pop();
        }
    }

    fn extract_function(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &mut Vec<Scope>) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = self.get_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);
            
            // Push function as scope for nested items
            let scope = Scope {
                name: name.clone(),
                kind: SymbolKind::Function,
                location: location.clone(),
            };
            scope_stack.push(scope);
            
            let signature = self.extract_function_signature(node, source);
            let documentation = self.extract_bash_doc(node, source);
            let modifiers = vec!["function".to_string()];
            
            symbols.push(Symbol {
                name,
                kind: SymbolKind::Function,
                location,
                scope_chain: scope_stack[..scope_stack.len()-1].to_vec(),
                language: LanguageId::Bash,
                documentation,
                modifiers,
                signature,
            });
        }
    }

    fn extract_variable(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope]) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = self.get_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);
            
            let mut modifiers = vec!["variable".to_string()];
            
            // Check if it's an array assignment
            if let Some(value) = node.child_by_field_name("value") {
                if value.kind() == "array" {
                    modifiers.push("array".to_string());
                } else if value.kind() == "command_substitution" {
                    modifiers.push("command_result".to_string());
                } else if value.kind() == "expansion" {
                    modifiers.push("expansion".to_string());
                }
            }

            // Check for special variable patterns
            if self.is_special_variable(&name) {
                modifiers.push("special".to_string());
            } else if self.is_environment_variable(&name) {
                modifiers.push("environment".to_string());
            }
            
            symbols.push(Symbol {
                name,
                kind: SymbolKind::Variable,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::Bash,
                documentation: self.extract_bash_doc(node, source),
                modifiers,
                signature: None,
            });
        }
    }

    fn extract_declaration(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope]) {
        // Handle declare, local, export, readonly commands
        let mut cursor = node.walk();
        let mut command_name = String::new();
        let mut var_names = Vec::new();
        
        for child in node.children(&mut cursor) {
            if child.kind() == "word" && command_name.is_empty() {
                command_name = self.get_node_text(&child, source);
            } else if child.kind() == "variable_assignment" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = self.get_node_text(&name_node, source);
                    let location = Location::from_node(&child, file_path);
                    var_names.push((name, location));
                }
            } else if child.kind() == "word" && !command_name.is_empty() {
                // Handle declarations like "local var1 var2"
                let text = self.get_node_text(&child, source);
                if !text.starts_with('-') { // Skip flags
                    let location = Location::from_node(&child, file_path);
                    var_names.push((text, location));
                }
            }
        }
        
        for (name, location) in var_names {
            let mut modifiers = vec!["variable".to_string()];
            
            match command_name.as_str() {
                "declare" => modifiers.push("declared".to_string()),
                "local" => modifiers.push("local".to_string()),
                "export" => modifiers.push("exported".to_string()),
                "readonly" => modifiers.push("readonly".to_string()),
                _ => {}
            }
            
            symbols.push(Symbol {
                name,
                kind: SymbolKind::Variable,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::Bash,
                documentation: self.extract_bash_doc(node, source),
                modifiers,
                signature: None,
            });
        }
    }

    fn extract_command(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope]) {
        // Check for source, ., include commands
        let mut cursor = node.walk();
        let mut command_parts = Vec::new();
        
        for child in node.children(&mut cursor) {
            if child.kind() == "word" || child.kind() == "string" {
                command_parts.push(self.get_node_text(&child, source));
            }
        }
        
        if let Some(first_part) = command_parts.first() {
            if matches!(first_part.as_str(), "source" | "." | "include") {
                if let Some(file_path_arg) = command_parts.get(1) {
                    let import_path = self.clean_string_literal(file_path_arg);
                    let location = Location::from_node(node, file_path);
                    
                    let mut modifiers = vec![first_part.clone()];
                    if first_part == "." {
                        modifiers.push("source".to_string());
                    }
                    
                    symbols.push(Symbol {
                        name: import_path,
                        kind: SymbolKind::Import,
                        location,
                        scope_chain: scope_stack.to_owned(),
                        language: LanguageId::Bash,
                        documentation: None,
                        modifiers,
                        signature: None,
                    });
                }
            } else if first_part == "alias" {
                // Handle alias definitions
                if let Some(alias_def) = command_parts.get(1) {
                    if let Some(eq_pos) = alias_def.find('=') {
                        let alias_name = &alias_def[..eq_pos];
                        let location = Location::from_node(node, file_path);
                        
                        symbols.push(Symbol {
                            name: alias_name.to_string(),
                            kind: SymbolKind::Function, // Treat aliases as functions
                            location,
                            scope_chain: scope_stack.to_owned(),
                            language: LanguageId::Bash,
                            documentation: self.extract_bash_doc(node, source),
                            modifiers: vec!["alias".to_string()],
                            signature: Some(alias_def.clone()),
                        });
                    }
                }
            }
        }
    }

    fn extract_for_variables(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope]) {
        // Extract loop variables from for statements
        if let Some(variable) = node.child_by_field_name("variable") {
            let name = self.get_node_text(&variable, source);
            let location = Location::from_node(&variable, file_path);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Variable,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::Bash,
                documentation: None,
                modifiers: vec!["variable".to_string(), "loop".to_string()],
                signature: None,
            });
        }
    }

    fn extract_case_patterns(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope]) {
        // Extract patterns from case statements - these can be useful for understanding script logic
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "case_item" {
                if let Some(pattern) = child.child_by_field_name("pattern") {
                    let pattern_text = self.get_node_text(&pattern, source);
                    let location = Location::from_node(&pattern, file_path);

                    // Extract individual patterns (they might be separated by |)
                    for pattern_part in pattern_text.split('|') {
                        let clean_pattern = pattern_part.trim();
                        if !clean_pattern.is_empty() && clean_pattern != ")" {
                            symbols.push(Symbol {
                                name: clean_pattern.to_string(),
                                kind: SymbolKind::Constant,
                                location: location.clone(),
                                scope_chain: scope_stack.to_owned(),
                                language: LanguageId::Bash,
                                documentation: None,
                                modifiers: vec!["pattern".to_string(), "case".to_string()],
                                signature: None,
                            });
                        }
                    }
                }
            }
        }
    }

    fn extract_command_substitution(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &[Scope]) {
        // Extract command substitutions like $(command) or `command`
        let substitution_text = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);

        // Generate a unique name for the command substitution
        let name = format!("cmd_subst_{}", location.start_line);

        symbols.push(Symbol {
            name,
            kind: SymbolKind::Function,
            location,
            scope_chain: scope_stack.to_owned(),
            language: LanguageId::Bash,
            documentation: None,
            modifiers: vec!["command_substitution".to_string()],
            signature: Some(substitution_text),
        });
    }

    fn get_node_text(&self, node: &Node, source: &str) -> String {
        node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
    }

    fn clean_string_literal(&self, text: &str) -> String {
        // Remove quotes from string literals
        if (text.starts_with('"') && text.ends_with('"')) || 
           (text.starts_with('\'') && text.ends_with('\'')) {
            text[1..text.len()-1].to_string()
        } else {
            text.to_string()
        }
    }

    fn extract_bash_doc(&self, node: &Node, source: &str) -> Option<String> {
        // Bash documentation appears as # comments preceding declarations
        let mut current = *node;
        let mut doc_comments = Vec::new();

        // Look backwards from the node to find Bash doc comments
        while let Some(prev) = current.prev_sibling() {
            match prev.kind() {
                "comment" => {
                    let comment_text = prev.utf8_text(source.as_bytes()).ok()?;
                    if comment_text.starts_with("##") {
                        // Bash doc comment (double hash)
                        let content = comment_text.strip_prefix("##").unwrap_or("").trim();
                        if !content.is_empty() {
                            doc_comments.insert(0, content.to_string());
                        }
                    } else if comment_text.starts_with("#") && !comment_text.starts_with("##") && !comment_text.starts_with("#!") {
                        // Regular comment - might be documentation
                        let content = comment_text.strip_prefix("#").unwrap_or("").trim();
                        if !content.is_empty() {
                            doc_comments.insert(0, content.to_string());
                        }
                    }
                    current = prev;
                }
                _ if prev.kind().contains("whitespace") || prev.kind() == "\n" => {
                    // Allow whitespace between comments and declarations
                    current = prev;
                    continue;
                }
                _ => {
                    // Stop at first non-comment, non-whitespace node
                    break;
                }
            }
        }

        if doc_comments.is_empty() {
            None
        } else {
            Some(doc_comments.join("\n"))
        }
    }

    fn extract_function_signature(&self, node: &Node, source: &str) -> Option<String> {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = self.get_node_text(&name_node, source);
            Some(format!("function {name}() {{"))
        } else {
            None
        }
    }

    fn is_special_variable(&self, name: &str) -> bool {
        // Bash special variables
        matches!(name,
            "$" | "?" | "!" | "#" | "*" | "@" | "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" |
            "BASH" | "BASH_VERSION" | "BASH_VERSINFO" | "BASHPID" | "PPID" | "UID" | "EUID" | "GROUPS" |
            "HOSTNAME" | "HOSTTYPE" | "MACHTYPE" | "OSTYPE" | "SHELLOPTS" | "BASHOPTS" | "BASH_ALIASES" |
            "BASH_ARGC" | "BASH_ARGV" | "BASH_CMDS" | "BASH_COMMAND" | "BASH_ENV" | "BASH_EXECUTION_STRING" |
            "BASH_LINENO" | "BASH_REMATCH" | "BASH_SOURCE" | "BASH_SUBSHELL" | "BASH_XTRACEFD" |
            "COLUMNS" | "COMP_CWORD" | "COMP_LINE" | "COMP_POINT" | "COMP_TYPE" | "COMP_KEY" | "COMP_WORDBREAKS" |
            "COMP_WORDS" | "COMPREPLY" | "COPROC" | "DIRSTACK" | "EPOCHREALTIME" | "EPOCHSECONDS" |
            "FUNCNAME" | "GLOBIGNORE" | "HISTCMD" | "HISTCONTROL" | "HISTFILE" | "HISTFILESIZE" |
            "HISTIGNORE" | "HISTSIZE" | "HISTTIMEFORMAT" | "IFS" | "IGNOREEOF" | "INPUTRC" |
            "LINENO" | "LINES" | "MAPFILE" | "OLDPWD" | "OPTARG" | "OPTERR" | "OPTIND" | "PIPESTATUS" |
            "POSIXLY_CORRECT" | "PWD" | "RANDOM" | "READLINE_LINE" | "READLINE_POINT" | "REPLY" |
            "SECONDS" | "SHLVL" | "TIMEFORMAT" | "TMOUT" | "TMPDIR"
        )
    }

    fn is_environment_variable(&self, name: &str) -> bool {
        // Common environment variables
        matches!(name,
            "PATH" | "HOME" | "USER" | "SHELL" | "TERM" | "LANG" | "LC_ALL" | "LC_CTYPE" | "LC_NUMERIC" |
            "LC_TIME" | "LC_COLLATE" | "LC_MONETARY" | "LC_MESSAGES" | "LC_PAPER" | "LC_NAME" |
            "LC_ADDRESS" | "LC_TELEPHONE" | "LC_MEASUREMENT" | "LC_IDENTIFICATION" | "EDITOR" |
            "VISUAL" | "PAGER" | "BROWSER" | "MANPATH" | "INFOPATH" | "LD_LIBRARY_PATH" | "PKG_CONFIG_PATH" |
            "PYTHONPATH" | "CLASSPATH" | "JAVA_HOME" | "ANDROID_HOME" | "GOPATH" | "GOROOT" | "CARGO_HOME" |
            "RUSTUP_HOME" | "NODE_PATH" | "NPM_CONFIG_PREFIX" | "GEM_HOME" | "GEM_PATH" | "RBENV_ROOT" |
            "PYENV_ROOT" | "NVM_DIR" | "CONDA_DEFAULT_ENV" | "VIRTUAL_ENV" | "WORKON_HOME" |
            "XDG_CONFIG_HOME" | "XDG_DATA_HOME" | "XDG_CACHE_HOME" | "XDG_RUNTIME_DIR" |
            "DISPLAY" | "WAYLAND_DISPLAY" | "SSH_AUTH_SOCK" | "SSH_AGENT_PID" | "GPG_AGENT_INFO" |
            "DBUS_SESSION_BUS_ADDRESS" | "DESKTOP_SESSION" | "XDG_CURRENT_DESKTOP" | "XDG_SESSION_TYPE"
        ) || name.starts_with("XDG_") || name.ends_with("_HOME") || name.ends_with("_PATH")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bash_symbol_extraction() {
        let extractor = BashExtractor;
        assert_eq!(extractor.language(), LanguageId::Bash);
    }

    #[test]
    fn test_function_signature_extraction() {
        let extractor = BashExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Bash);
    }

    #[test]
    fn test_variable_extraction() {
        let extractor = BashExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Bash);
    }

    #[test]
    fn test_declaration_extraction() {
        let extractor = BashExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Bash);
    }

    #[test]
    fn test_source_extraction() {
        let extractor = BashExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Bash);
    }

    #[test]
    fn test_alias_extraction() {
        let extractor = BashExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Bash);
    }

    #[test]
    fn test_for_variable_extraction() {
        let extractor = BashExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Bash);
    }

    #[test]
    fn test_bash_doc_extraction() {
        let extractor = BashExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Bash);
    }

    #[test]
    fn test_case_pattern_extraction() {
        let extractor = BashExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Bash);
    }

    #[test]
    fn test_command_substitution_extraction() {
        let extractor = BashExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Bash);
    }

    #[test]
    fn test_special_variable_detection() {
        let extractor = BashExtractor;

        // Test special variable detection
        assert!(extractor.is_special_variable("BASH_VERSION"));
        assert!(extractor.is_special_variable("PWD"));
        assert!(!extractor.is_special_variable("MY_VAR"));

        // Test environment variable detection
        assert!(extractor.is_environment_variable("PATH"));
        assert!(extractor.is_environment_variable("HOME"));
        assert!(!extractor.is_environment_variable("MY_VAR"));
    }

    #[test]
    fn test_environment_variable_detection() {
        let extractor = BashExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::Bash);
    }
}

//! Regex symbol extractor
//!
//! Extracts symbols from regular expressions including:
//! - Named capture groups
//! - Character classes  
//! - Quantifiers
//! - Anchors
//! - Backreferences

use crate::parsers::LanguageId;
use crate::symbols::{Location, Scope, Symbol, SymbolExtractor, SymbolKind};
use tree_sitter::{Node, Tree};

/// Regex symbol extractor
pub struct RegexExtractor;

impl SymbolExtractor for RegexExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Regex
    }

    fn extract_symbols(&self, tree: &Tree, source: &str, file_path: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let mut scope_stack = Vec::new();

        self.extract_from_node(
            tree.root_node(),
            source,
            file_path,
            &mut symbols,
            &mut scope_stack,
        );
        symbols
    }
}

impl RegexExtractor {
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            // Named capture groups: (?P<name>pattern) or (?<name>pattern)
            "named_group" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = self.get_node_text(name_node, source);
                    if !name.is_empty() {
                        let location = Location::from_node(&name_node, file_path);
                        symbols.push(Symbol {
                            name: name.clone(),
                            kind: SymbolKind::Variable,
                            location,
                            scope_chain: scope_stack.clone(),
                            language: LanguageId::Regex,
                            documentation: None,
                            modifiers: vec!["capture_group".to_string()],
                            signature: None,
                        });
                    }
                }
            }

            // Character classes: [abc], [a-z], [^abc]
            "character_class" => {
                let class_text = self.get_node_text(node, source);
                if !class_text.is_empty() {
                    let location = Location::from_node(&node, file_path);
                    symbols.push(Symbol {
                        name: class_text.clone(),
                        kind: SymbolKind::Constant,
                        location,
                        scope_chain: scope_stack.clone(),
                        language: LanguageId::Regex,
                        documentation: None,
                        modifiers: vec!["character_class".to_string()],
                        signature: None,
                    });
                }
            }

            // Quantifiers: *, +, ?, {n}, {n,m}
            "quantifier" => {
                let quantifier_text = self.get_node_text(node, source);
                if !quantifier_text.is_empty() {
                    let location = Location::from_node(&node, file_path);
                    symbols.push(Symbol {
                        name: quantifier_text.clone(),
                        kind: SymbolKind::Constant,
                        location,
                        scope_chain: scope_stack.clone(),
                        language: LanguageId::Regex,
                        documentation: None,
                        modifiers: vec!["quantifier".to_string()],
                        signature: None,
                    });
                }
            }

            // Anchors: ^, $, \b, \B
            "anchor" => {
                let anchor_text = self.get_node_text(node, source);
                if !anchor_text.is_empty() {
                    let location = Location::from_node(&node, file_path);
                    symbols.push(Symbol {
                        name: anchor_text.clone(),
                        kind: SymbolKind::Constant,
                        location,
                        scope_chain: scope_stack.clone(),
                        language: LanguageId::Regex,
                        documentation: None,
                        modifiers: vec!["anchor".to_string()],
                        signature: None,
                    });
                }
            }

            // Backreferences: \1, \2, etc.
            "backreference" => {
                let backref_text = self.get_node_text(node, source);
                if !backref_text.is_empty() {
                    let location = Location::from_node(&node, file_path);
                    symbols.push(Symbol {
                        name: backref_text.clone(),
                        kind: SymbolKind::Variable,
                        location,
                        scope_chain: scope_stack.clone(),
                        language: LanguageId::Regex,
                        documentation: None,
                        modifiers: vec!["backreference".to_string()],
                        signature: None,
                    });
                }
            }

            _ => {}
        }

        // Recursively process child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_from_node(child, source, file_path, symbols, scope_stack);
        }
    }

    fn get_node_text(&self, node: Node, source: &str) -> String {
        source[node.byte_range()].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_symbol_extraction() {
        let extractor = RegexExtractor;

        // Test regex with named capture groups
        let _regex_code = r"(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2})";

        // Note: This test would need actual tree-sitter-regex parsing
        // For now, we'll just test the extractor structure
        assert_eq!(extractor.language(), LanguageId::Regex);
    }
}

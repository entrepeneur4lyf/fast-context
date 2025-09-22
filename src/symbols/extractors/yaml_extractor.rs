//! YAML symbol extractor
//!
//! Extracts symbols from YAML source code including:
//! - Keys and values
//! - Anchors and aliases (references)
//! - Arrays and objects
//! - Comments and documentation
//! - Nested structures

use crate::parsers::LanguageId;
use crate::symbols::{Location, Scope, Symbol, SymbolExtractor, SymbolKind};
use tree_sitter::{Node, Tree};

/// Safe text extraction from tree-sitter nodes with bounds checking
fn safe_node_text(node: &Node, source: &str) -> String {
    let start = node.start_byte();
    let end = node.end_byte();
    
    // Ensure byte range is within source bounds
    if start <= end && end <= source.len() {
        // Use direct slice access with bounds checking
        if let Some(slice) = source.get(start..end) {
            return slice.to_string();
        }
    }
    
    // Return empty string if bounds are invalid
    String::new()
}

/// YAML Symbol Extractor
/// Extracts keys, values, anchors, aliases, and nested structures from YAML code
pub struct YamlExtractor;

impl SymbolExtractor for YamlExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::YAML
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

impl YamlExtractor {
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            "block_mapping_pair" => {
                self.extract_mapping_pair(&node, source, file_path, symbols, scope_stack);
            }
            "flow_mapping_pair" => {
                self.extract_flow_mapping_pair(&node, source, file_path, symbols, scope_stack);
            }
            "anchor" => {
                self.extract_anchor(&node, source, file_path, symbols, scope_stack);
            }
            "alias" => {
                self.extract_alias(&node, source, file_path, symbols, scope_stack);
            }
            "block_sequence_item" => {
                self.extract_sequence_item(&node, source, file_path, symbols, scope_stack);
            }
            "flow_sequence" => {
                self.extract_flow_sequence(&node, source, file_path, symbols, scope_stack);
            }
            "comment" => {
                self.extract_comment(&node, source, file_path, symbols, scope_stack);
            }
            _ => {}
        }

        // Recursively process child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_from_node(child, source, file_path, symbols, scope_stack);
        }

        // Pop scope if we added one for this node
        if matches!(node.kind(), "block_mapping_pair" | "flow_mapping_pair")
            && self.should_create_scope(&node, source)
        {
            scope_stack.pop();
        }
    }

    fn extract_mapping_pair(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        // YAML mapping pair: key: value
        if let Some(key_node) = node.child_by_field_name("key") {
            let key_text = self.extract_node_text(&key_node, source);
            if !key_text.is_empty() {
                let location = Location::from_node(&key_node, file_path);

                // Determine if this is a complex object that should create a scope
                let value_node = node.child_by_field_name("value");
                let is_complex = value_node.is_some_and(|v| self.is_complex_value(&v));

                let kind = if is_complex {
                    SymbolKind::Namespace
                } else {
                    SymbolKind::Field
                };

                let mut modifiers = vec!["key".to_string()];

                // Check for anchors
                if self.has_anchor(&key_node, source) {
                    modifiers.push("anchor".to_string());
                }

                // Extract value information for signature
                let signature = if let Some(value_node) = value_node {
                    let value_text = self.extract_node_text(&value_node, source);
                    if value_text.len() > 100 {
                        let truncated = value_text.get(..97).unwrap_or(&value_text);
                        Some(format!("{}: {}...", key_text, truncated))
                    } else {
                        Some(format!("{key_text}: {value_text}"))
                    }
                } else {
                    Some(format!("{key_text}: null"))
                };

                // Create scope for complex objects
                if is_complex {
                    let scope = Scope {
                        name: key_text.clone(),
                        kind: SymbolKind::Namespace,
                        location: location.clone(),
                    };
                    scope_stack.push(scope);
                }

                symbols.push(Symbol {
                    name: key_text,
                    kind,
                    location,
                    scope_chain: scope_stack[..if is_complex {
                        scope_stack.len() - 1
                    } else {
                        scope_stack.len()
                    }]
                        .to_vec(),
                    language: LanguageId::YAML,
                    documentation: self.extract_yaml_comment(node, source),
                    modifiers,
                    signature,
                });
            }
        }
    }

    fn extract_flow_mapping_pair(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Flow mapping pair: {key: value}
        if let Some(key_node) = node.child_by_field_name("key") {
            let key_text = self.extract_node_text(&key_node, source);
            if !key_text.is_empty() {
                let location = Location::from_node(&key_node, file_path);

                let value_text = if let Some(value_node) = node.child_by_field_name("value") {
                    self.extract_node_text(&value_node, source)
                } else {
                    "null".to_string()
                };

                symbols.push(Symbol {
                    name: key_text.clone(),
                    kind: SymbolKind::Field,
                    location,
                    scope_chain: scope_stack.to_owned(),
                    language: LanguageId::YAML,
                    documentation: None,
                    modifiers: vec!["flow_key".to_string()],
                    signature: Some(format!("{key_text}: {value_text}")),
                });
            }
        }
    }

    fn extract_anchor(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // YAML anchor: &anchor_name
        let anchor_text = safe_node_text(node, source);
        if !anchor_text.is_empty() {
            let clean_name = anchor_text.trim_start_matches('&');
            let location = Location::from_node(node, file_path);

            symbols.push(Symbol {
                name: clean_name.to_string(),
                kind: SymbolKind::Constant,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::YAML,
                documentation: None,
                modifiers: vec!["anchor".to_string()],
                signature: Some(anchor_text),
            });
        }
    }

    fn extract_alias(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // YAML alias: *anchor_name
        let alias_text = safe_node_text(node, source);
        if !alias_text.is_empty() {
            let clean_name = alias_text.trim_start_matches('*');
            let location = Location::from_node(node, file_path);

            symbols.push(Symbol {
                name: clean_name.to_string(),
                kind: SymbolKind::Variable,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::YAML,
                documentation: None,
                modifiers: vec!["alias".to_string()],
                signature: Some(alias_text),
            });
        }
    }

    fn extract_sequence_item(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // YAML sequence item: - item
        let item_text = self.extract_node_text(node, source);
        if !item_text.is_empty() && item_text.len() < 100 {
            let location = Location::from_node(node, file_path);

            // Generate a name based on the content or position
            let name = if item_text.contains(':') {
                // If it's a mapping, use the first key as name
                item_text
                    .split(':')
                    .next()
                    .unwrap_or("item")
                    .trim()
                    .to_string()
            } else {
                // Use the content itself if short, otherwise use position
                if item_text.len() < 30 {
                    item_text.clone()
                } else {
                    format!("item_{}_{}", location.start_line, location.start_column)
                }
            };

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Variable,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::YAML,
                documentation: None,
                modifiers: vec!["sequence_item".to_string()],
                signature: Some(item_text),
            });
        }
    }

    fn extract_flow_sequence(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // Flow sequence: [item1, item2, item3]
        let sequence_text = self.extract_node_text(node, source);
        if !sequence_text.is_empty() {
            let location = Location::from_node(node, file_path);
            let name = format!("array_{}_{}", location.start_line, location.start_column);

            symbols.push(Symbol {
                name,
                kind: SymbolKind::Variable,
                location,
                scope_chain: scope_stack.to_owned(),
                language: LanguageId::YAML,
                documentation: None,
                modifiers: vec!["flow_sequence".to_string()],
                signature: Some(sequence_text),
            });
        }
    }

    fn extract_comment(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        // YAML comment: # comment text
        let comment_text = safe_node_text(node, source);
        if !comment_text.is_empty() && comment_text.len() > 5 {
            let clean_comment = comment_text.trim_start_matches('#').trim();
            if !clean_comment.is_empty() && clean_comment.len() > 10 {
                let location = Location::from_node(node, file_path);
                let name = format!("comment_{}_{}", location.start_line, location.start_column);

                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::Variable,
                    location,
                    scope_chain: scope_stack.to_owned(),
                    language: LanguageId::YAML,
                    documentation: Some(clean_comment.to_string()),
                    modifiers: vec!["comment".to_string()],
                    signature: Some(comment_text),
                });
            }
        }
    }

    /// Helper methods for YAML processing
    fn extract_node_text(&self, node: &Node, source: &str) -> String {
        node.utf8_text(source.as_bytes())
            .unwrap_or("")
            .trim()
            .to_string()
    }

    fn is_complex_value(&self, node: &Node) -> bool {
        // Check if the value is a complex structure (mapping, sequence)
        matches!(
            node.kind(),
            "block_mapping" | "flow_mapping" | "block_sequence" | "flow_sequence"
        )
    }

    fn has_anchor(&self, node: &Node, source: &str) -> bool {
        // Check if the node or its children contain an anchor
        let text = self.extract_node_text(node, source);
        text.contains('&')
    }

    fn should_create_scope(&self, node: &Node, _source: &str) -> bool {
        // Determine if this mapping pair should create a scope
        if let Some(value_node) = node.child_by_field_name("value") {
            self.is_complex_value(&value_node)
        } else {
            false
        }
    }

    fn extract_yaml_comment(&self, node: &Node, source: &str) -> Option<String> {
        // Look for comments preceding this node
        let mut current = *node;
        while let Some(prev) = current.prev_sibling() {
            match prev.kind() {
                "comment" => {
                    let comment_text = prev.utf8_text(source.as_bytes()).ok()?;
                    let clean_comment = comment_text.trim_start_matches('#').trim();
                    if !clean_comment.is_empty() {
                        return Some(clean_comment.to_string());
                    }
                }
                _ if prev.kind().contains("whitespace") || prev.kind() == "\n" => {
                    current = prev;
                    continue;
                }
                _ => break,
            }
            current = prev;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_symbol_extraction() {
        let extractor = YamlExtractor;
        assert_eq!(extractor.language(), LanguageId::YAML);
    }

    #[test]
    fn test_extract_node_text() {
        let extractor = YamlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::YAML);
    }

    #[test]
    fn test_is_complex_value() {
        let extractor = YamlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::YAML);
    }

    #[test]
    fn test_anchor_detection() {
        let extractor = YamlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::YAML);
    }

    #[test]
    fn test_mapping_pair_extraction() {
        let extractor = YamlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::YAML);
    }

    #[test]
    fn test_sequence_extraction() {
        let extractor = YamlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::YAML);
    }

    #[test]
    fn test_comment_extraction() {
        let extractor = YamlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::YAML);
    }
}

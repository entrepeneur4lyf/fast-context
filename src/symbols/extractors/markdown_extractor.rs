//! Markdown symbol extractor
//!
//! Extracts symbols from Markdown documents including:
//! - Headers (h1-h6)
//! - Links and references
//! - Code blocks and inline code
//! - Tables and table headers
//! - Images and media references

use crate::parsers::LanguageId;
use crate::symbols::{Location, Scope, Symbol, SymbolExtractor, SymbolKind};
use tree_sitter::{Node, Tree};

/// Markdown symbol extractor
pub struct MarkdownExtractor;

impl SymbolExtractor for MarkdownExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Markdown
    }

    fn extract_symbols(&self, tree: &Tree, source: &str, file_path: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let mut scope_stack = Vec::new();

        self.extract_from_node(tree.root_node(), source, file_path, &mut symbols, &mut scope_stack);
        symbols
    }
}

impl MarkdownExtractor {
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            // Headers: # Header, ## Header, etc.
            "atx_heading" | "setext_heading" => {
                if let Some(heading_content) = node.child_by_field_name("heading_content") {
                    let title = self.get_node_text(heading_content, source).trim().to_string();
                    if !title.is_empty() {
                        let location = Location::from_node(&node, file_path);
                        let level = self.get_heading_level(&node, source);

                        // Create scope for this heading
                        let scope = Scope {
                            name: title.clone(),
                            kind: SymbolKind::Module, // Use Module for sections
                            location: location.clone(),
                        };

                        // Adjust scope stack based on heading level
                        self.adjust_scope_stack_for_heading(scope_stack, level, scope.clone());

                        symbols.push(Symbol {
                            name: title.clone(),
                            kind: SymbolKind::Module,
                            location,
                            scope_chain: scope_stack.clone(),
                            language: LanguageId::Markdown,
                            documentation: None,
                            modifiers: vec![format!("h{}", level)],
                            signature: None,
                        });
                    }
                }
            }

            // Links: [text](url) or [text][ref]
            "link" => {
                if let Some(link_text) = node.child_by_field_name("link_text") {
                    let text = self.get_node_text(link_text, source).trim().to_string();
                    if !text.is_empty() {
                        let location = Location::from_node(&node, file_path);
                        let url = self.extract_link_url(&node, source);

                        symbols.push(Symbol {
                            name: text.clone(),
                            kind: SymbolKind::Import, // Links are like imports/references
                            location,
                            scope_chain: scope_stack.clone(),
                            language: LanguageId::Markdown,
                            documentation: url.clone(),
                            modifiers: vec!["link".to_string()],
                            signature: url,
                        });
                    }
                }
            }

            // Reference definitions: [ref]: url "title"
            "link_reference_definition" => {
                if let Some(label) = node.child_by_field_name("label") {
                    let ref_name = self.get_node_text(label, source).trim().to_string();
                    if !ref_name.is_empty() {
                        let location = Location::from_node(&node, file_path);
                        let url = self.extract_reference_url(&node, source);

                        symbols.push(Symbol {
                            name: ref_name.clone(),
                            kind: SymbolKind::Constant, // Reference definitions are constants
                            location,
                            scope_chain: scope_stack.clone(),
                            language: LanguageId::Markdown,
                            documentation: None,
                            modifiers: vec!["reference".to_string()],
                            signature: url,
                        });
                    }
                }
            }

            // Code blocks: ```language or ~~~language
            "fenced_code_block" => {
                let location = Location::from_node(&node, file_path);
                let language = self.extract_code_block_language(&node, source);
                let code_content = self.extract_code_block_content(&node, source);

                let name = if let Some(lang) = &language {
                    format!("code_block_{lang}")
                } else {
                    "code_block".to_string()
                };

                symbols.push(Symbol {
                    name,
                    kind: SymbolKind::Function, // Code blocks are like functions
                    location,
                    scope_chain: scope_stack.clone(),
                    language: LanguageId::Markdown,
                    documentation: code_content,
                    modifiers: vec!["code_block".to_string()],
                    signature: language,
                });
            }

            // Tables
            "table" => {
                let location = Location::from_node(&node, file_path);
                let headers = self.extract_table_headers(&node, source);

                symbols.push(Symbol {
                    name: "table".to_string(),
                    kind: SymbolKind::Struct, // Tables are like structs
                    location,
                    scope_chain: scope_stack.clone(),
                    language: LanguageId::Markdown,
                    documentation: None,
                    modifiers: vec!["table".to_string()],
                    signature: headers,
                });
            }

            // Images: ![alt](src)
            "image" => {
                if let Some(alt_text) = node.child_by_field_name("link_text") {
                    let alt = self.get_node_text(alt_text, source).trim().to_string();
                    if !alt.is_empty() {
                        let location = Location::from_node(&node, file_path);
                        let src = self.extract_image_src(&node, source);

                        symbols.push(Symbol {
                            name: alt.clone(),
                            kind: SymbolKind::Constant, // Images are constants/resources
                            location,
                            scope_chain: scope_stack.clone(),
                            language: LanguageId::Markdown,
                            documentation: None,
                            modifiers: vec!["image".to_string()],
                            signature: src,
                        });
                    }
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

    fn get_heading_level(&self, node: &Node, source: &str) -> usize {
        // For ATX headings, count the # characters
        if node.kind() == "atx_heading" {
            let text = self.get_node_text(*node, source);
            text.chars().take_while(|&c| c == '#').count()
        } else {
            // For setext headings, = is h1, - is h2
            let text = self.get_node_text(*node, source);
            if text.contains('=') { 1 } else { 2 }
        }
    }

    fn adjust_scope_stack_for_heading(&self, scope_stack: &mut Vec<Scope>, level: usize, new_scope: Scope) {
        // Remove scopes that are at the same level or deeper
        scope_stack.retain(|scope| {
            if let Some(modifier) = scope.name.chars().take_while(|&c| c == '#').count().checked_sub(1) {
                modifier < level - 1
            } else {
                true
            }
        });

        scope_stack.push(new_scope);
    }

    fn extract_link_url(&self, node: &Node, source: &str) -> Option<String> {
        if let Some(url_node) = node.child_by_field_name("link_destination") {
            let url = self.get_node_text(url_node, source).trim().to_string();
            if !url.is_empty() {
                return Some(url);
            }
        }
        None
    }

    fn extract_reference_url(&self, node: &Node, source: &str) -> Option<String> {
        if let Some(url_node) = node.child_by_field_name("link_destination") {
            let url = self.get_node_text(url_node, source).trim().to_string();
            if !url.is_empty() {
                return Some(url);
            }
        }
        None
    }

    fn extract_code_block_language(&self, node: &Node, source: &str) -> Option<String> {
        if let Some(info_node) = node.child_by_field_name("info_string") {
            let info = self.get_node_text(info_node, source).trim().to_string();
            if !info.is_empty() {
                return Some(info);
            }
        }
        None
    }

    fn extract_code_block_content(&self, node: &Node, source: &str) -> Option<String> {
        if let Some(content_node) = node.child_by_field_name("code_fence_content") {
            let content = self.get_node_text(content_node, source);
            if !content.trim().is_empty() {
                return Some(content);
            }
        }
        None
    }

    fn extract_table_headers(&self, node: &Node, source: &str) -> Option<String> {
        // Look for table header row
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "table_header_row" {
                let headers: Vec<String> = child.children(&mut child.walk())
                    .filter(|cell| cell.kind() == "table_cell")
                    .map(|cell| self.get_node_text(cell, source).trim().to_string())
                    .collect();

                if !headers.is_empty() {
                    return Some(headers.join(", "));
                }
            }
        }
        None
    }

    fn extract_image_src(&self, node: &Node, source: &str) -> Option<String> {
        if let Some(src_node) = node.child_by_field_name("link_destination") {
            let src = self.get_node_text(src_node, source).trim().to_string();
            if !src.is_empty() {
                return Some(src);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_symbol_extraction() {
        let extractor = MarkdownExtractor;
        assert_eq!(extractor.language(), LanguageId::Markdown);
    }
}

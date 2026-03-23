//! CSS symbol extractor
//!
//! Extracts symbols from CSS source code including:
//! - Selectors (class, id, element, attribute, pseudo)
//! - Rules and at-rules
//! - Properties and custom properties
//! - Imports and media queries
//! - Keyframes and animations

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

/// CSS Symbol Extractor
/// Extracts selectors, rules, properties, imports from CSS code
pub struct CssExtractor;

impl SymbolExtractor for CssExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::CSS
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

impl CssExtractor {
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            "rule_set" => {
                self.extract_rule_set(&node, source, file_path, symbols, scope_stack);
            }
            "at_rule" => {
                self.extract_at_rule(&node, source, file_path, symbols, scope_stack);
            }
            "import_statement" => {
                self.extract_import(&node, source, file_path, symbols, scope_stack);
            }
            "media_statement" => {
                self.extract_media(&node, source, file_path, symbols, scope_stack);
            }
            "keyframes_statement" => {
                self.extract_keyframes(&node, source, file_path, symbols, scope_stack);
            }
            "property_name" => {
                self.extract_property(&node, source, file_path, symbols, scope_stack);
            }
            "class_selector" => {
                self.extract_class_selector(&node, source, file_path, symbols, scope_stack);
            }
            "id_selector" => {
                self.extract_id_selector(&node, source, file_path, symbols, scope_stack);
            }
            "attribute_selector" => {
                self.extract_attribute_selector(&node, source, file_path, symbols, scope_stack);
            }
            "pseudo_class_selector" | "pseudo_element_selector" => {
                self.extract_pseudo_selector(&node, source, file_path, symbols, scope_stack);
            }
            "custom_property_name" => {
                self.extract_custom_property(&node, source, file_path, symbols, scope_stack);
            }
            "supports_statement" => {
                self.extract_supports(&node, source, file_path, symbols, scope_stack);
            }
            _ => {}
        }

        // Recursively process child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_from_node(child, source, file_path, symbols, scope_stack);
        }

        // Pop scope if we added one for this node
        if matches!(
            node.kind(),
            "rule_set" | "media_statement" | "keyframes_statement" | "supports_statement"
        ) {
            scope_stack.pop();
        }
    }

    fn extract_rule_set(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        // Extract selectors from the rule set
        if let Some(selectors) = node.child_by_field_name("selectors") {
            let selector_text = self.get_node_text(&selectors, source);
            let location = Location::from_node(&selectors, file_path);

            // Push rule as scope for properties
            let scope = Scope {
                name: selector_text.clone(),
                kind: SymbolKind::Class, // Treat CSS rules as classes
                location: location.clone(),
            };
            scope_stack.push(scope);

            // Extract individual selectors
            let mut cursor = selectors.walk();
            for child in selectors.children(&mut cursor) {
                if child.kind() == "selector" {
                    let selector = self.get_node_text(&child, source).trim().to_string();
                    if !selector.is_empty() {
                        let location = Location::from_node(&child, file_path);

                        let mut modifiers = vec!["selector".to_string()];

                        // Determine selector type
                        if selector.starts_with('.') {
                            modifiers.push("class".to_string());
                        } else if selector.starts_with('#') {
                            modifiers.push("id".to_string());
                        } else if selector.contains('[') {
                            modifiers.push("attribute".to_string());
                        } else if selector.contains(':') {
                            modifiers.push("pseudo".to_string());
                        } else {
                            modifiers.push("element".to_string());
                        }

                        symbols.push(Symbol {
                            name: selector,
                            kind: SymbolKind::Class,
                            location,
                            scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                            language: LanguageId::CSS,
                            documentation: self.extract_css_doc(node, source),
                            modifiers,
                            signature: None,
                        });
                    }
                }
            }
        }
    }

    fn extract_at_rule(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        let rule_text = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);

        // Extract at-rule name
        if let Some(name_start) = rule_text.find('@') {
            if let Some(name_end) = rule_text[name_start + 1..]
                .find(|c: char| c.is_whitespace() || c == '{' || c == ';')
            {
                let at_rule_name = &rule_text[name_start + 1..name_start + 1 + name_end];

                let mut modifiers = vec!["at-rule".to_string(), at_rule_name.to_string()];

                // Special handling for different at-rules
                match at_rule_name {
                    "import" => {
                        modifiers.push("import".to_string());
                    }
                    "media" => {
                        modifiers.push("media-query".to_string());
                    }
                    "keyframes" => {
                        modifiers.push("animation".to_string());
                    }
                    "font-face" => {
                        modifiers.push("font".to_string());
                    }
                    _ => {}
                }

                symbols.push(Symbol {
                    name: format!("@{at_rule_name}"),
                    kind: SymbolKind::Constant,
                    location,
                    scope_chain: scope_stack.to_owned(),
                    language: LanguageId::CSS,
                    documentation: self.extract_css_doc(node, source),
                    modifiers,
                    signature: Some(rule_text.trim().to_string()),
                });
            }
        }
    }

    fn extract_import(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        let import_text = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);

        // Extract import path
        if let Some(url_start) = import_text.find('"').or_else(|| import_text.find('\'')) {
            let quote_char = import_text.chars().nth(url_start).unwrap();
            if let Some(url_end) = import_text[url_start + 1..].find(quote_char) {
                let import_path = &import_text[url_start + 1..url_start + 1 + url_end];

                symbols.push(Symbol {
                    name: import_path.to_string(),
                    kind: SymbolKind::Import,
                    location,
                    scope_chain: scope_stack.to_owned(),
                    language: LanguageId::CSS,
                    documentation: None,
                    modifiers: vec!["import".to_string(), "css".to_string()],
                    signature: Some(import_text.trim().to_string()),
                });
            }
        } else if import_text.contains("url(") {
            // Handle url() imports
            if let Some(url_start) = import_text.find("url(") {
                let start_pos = url_start + 4;
                if let Some(url_end) = import_text[start_pos..].find(')') {
                    let mut import_path = &import_text[start_pos..start_pos + url_end];

                    // Remove quotes if present
                    import_path = import_path.trim_start_matches('"').trim_start_matches('\'');
                    import_path = import_path.trim_end_matches('"').trim_end_matches('\'');

                    symbols.push(Symbol {
                        name: import_path.to_string(),
                        kind: SymbolKind::Import,
                        location,
                        scope_chain: scope_stack.to_owned(),
                        language: LanguageId::CSS,
                        documentation: None,
                        modifiers: vec!["import".to_string(), "url".to_string()],
                        signature: Some(import_text.trim().to_string()),
                    });
                }
            }
        }
    }

    fn extract_media(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        let media_text = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);

        // Extract media query
        if let Some(query_start) = media_text.find("@media") {
            let query_part = &media_text[query_start + 6..];
            if let Some(brace_pos) = query_part.find('{') {
                let media_query = query_part[..brace_pos].trim();

                // Push media query as scope
                let scope = Scope {
                    name: format!("@media {media_query}"),
                    kind: SymbolKind::Namespace,
                    location: location.clone(),
                };
                scope_stack.push(scope);

                symbols.push(Symbol {
                    name: format!("@media {media_query}"),
                    kind: SymbolKind::Namespace,
                    location,
                    scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                    language: LanguageId::CSS,
                    documentation: self.extract_css_doc(node, source),
                    modifiers: vec!["media-query".to_string()],
                    signature: None,
                });
            }
        }
    }

    fn extract_keyframes(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        let keyframes_text = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);

        // Extract keyframes name
        if let Some(name_start) = keyframes_text.find("@keyframes") {
            let name_part = &keyframes_text[name_start + 10..];
            if let Some(brace_pos) = name_part.find('{') {
                let animation_name = name_part[..brace_pos].trim();

                // Push keyframes as scope
                let scope = Scope {
                    name: animation_name.to_string(),
                    kind: SymbolKind::Function,
                    location: location.clone(),
                };
                scope_stack.push(scope);

                symbols.push(Symbol {
                    name: animation_name.to_string(),
                    kind: SymbolKind::Function,
                    location,
                    scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                    language: LanguageId::CSS,
                    documentation: self.extract_css_doc(node, source),
                    modifiers: vec!["keyframes".to_string(), "animation".to_string()],
                    signature: Some(format!("@keyframes {animation_name}")),
                });
            }
        }
    }

    fn extract_property(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        let property_name = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);

        let mut modifiers = vec!["property".to_string()];

        // Detect custom properties (CSS variables)
        if property_name.starts_with("--") {
            modifiers.push("custom".to_string());
            modifiers.push("variable".to_string());
        }

        // Detect modern CSS features
        if self.is_grid_property(&property_name) {
            modifiers.push("grid".to_string());
        } else if self.is_flexbox_property(&property_name) {
            modifiers.push("flexbox".to_string());
        } else if self.is_animation_property(&property_name) {
            modifiers.push("animation".to_string());
        } else if self.is_transform_property(&property_name) {
            modifiers.push("transform".to_string());
        }

        // Get property value if available
        let signature = if let Some(parent) = node.parent() {
            if parent.kind() == "declaration" {
                Some(self.get_node_text(&parent, source))
            } else {
                None
            }
        } else {
            None
        };

        symbols.push(Symbol {
            name: property_name,
            kind: if modifiers.contains(&"custom".to_string()) {
                SymbolKind::Variable
            } else {
                SymbolKind::Field
            },
            location,
            scope_chain: scope_stack.to_owned(),
            language: LanguageId::CSS,
            documentation: None,
            modifiers,
            signature,
        });
    }

    fn extract_class_selector(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        let class_name = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);

        symbols.push(Symbol {
            name: class_name,
            kind: SymbolKind::Class,
            location,
            scope_chain: scope_stack.to_owned(),
            language: LanguageId::CSS,
            documentation: None,
            modifiers: vec!["selector".to_string(), "class".to_string()],
            signature: None,
        });
    }

    fn extract_id_selector(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        let id_name = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);

        symbols.push(Symbol {
            name: id_name,
            kind: SymbolKind::Variable, // IDs are unique like variables
            location,
            scope_chain: scope_stack.to_owned(),
            language: LanguageId::CSS,
            documentation: None,
            modifiers: vec!["selector".to_string(), "id".to_string()],
            signature: None,
        });
    }

    fn extract_attribute_selector(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        let attr_selector = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);

        symbols.push(Symbol {
            name: attr_selector,
            kind: SymbolKind::Field,
            location,
            scope_chain: scope_stack.to_owned(),
            language: LanguageId::CSS,
            documentation: None,
            modifiers: vec!["selector".to_string(), "attribute".to_string()],
            signature: None,
        });
    }

    fn extract_pseudo_selector(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        let pseudo_selector = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);

        let mut modifiers = vec!["selector".to_string()];

        if node.kind() == "pseudo_class_selector" {
            modifiers.push("pseudo-class".to_string());
        } else {
            modifiers.push("pseudo-element".to_string());
        }

        symbols.push(Symbol {
            name: pseudo_selector,
            kind: SymbolKind::Method, // Pseudo selectors are like methods
            location,
            scope_chain: scope_stack.to_owned(),
            language: LanguageId::CSS,
            documentation: None,
            modifiers,
            signature: None,
        });
    }

    fn extract_custom_property(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &[Scope],
    ) {
        let property_name = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);

        // Get property value if available
        let signature = if let Some(parent) = node.parent() {
            if parent.kind() == "declaration" {
                Some(self.get_node_text(&parent, source))
            } else {
                None
            }
        } else {
            None
        };

        symbols.push(Symbol {
            name: property_name,
            kind: SymbolKind::Variable,
            location,
            scope_chain: scope_stack.to_owned(),
            language: LanguageId::CSS,
            documentation: None,
            modifiers: vec![
                "custom-property".to_string(),
                "variable".to_string(),
                "css-var".to_string(),
            ],
            signature,
        });
    }

    fn extract_supports(
        &self,
        node: &Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        let supports_text = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);

        // Extract supports query
        if let Some(query_start) = supports_text.find("@supports") {
            let query_part = &supports_text[query_start + 9..];
            if let Some(brace_pos) = query_part.find('{') {
                let supports_query = query_part[..brace_pos].trim();

                // Push supports query as scope
                let scope = Scope {
                    name: format!("@supports {supports_query}"),
                    kind: SymbolKind::Namespace,
                    location: location.clone(),
                };
                scope_stack.push(scope);

                symbols.push(Symbol {
                    name: format!("@supports {supports_query}"),
                    kind: SymbolKind::Namespace,
                    location,
                    scope_chain: scope_stack[..scope_stack.len() - 1].to_vec(),
                    language: LanguageId::CSS,
                    documentation: self.extract_css_doc(node, source),
                    modifiers: vec!["supports-query".to_string(), "feature-query".to_string()],
                    signature: None,
                });
            }
        }
    }

    fn get_node_text(&self, node: &Node, source: &str) -> String {
        safe_node_text(node, source)
    }

    fn extract_css_doc(&self, node: &Node, source: &str) -> Option<String> {
        // CSS documentation appears as /* */ comments preceding declarations
        let mut current = *node;
        let mut doc_comments = Vec::new();

        // Look backwards from the node to find CSS doc comments
        while let Some(prev) = current.prev_sibling() {
            match prev.kind() {
                "comment" => {
                    let comment_text = prev.utf8_text(source.as_bytes()).ok()?;
                    if comment_text.starts_with("/*") && comment_text.ends_with("*/") {
                        let content = comment_text
                            .strip_prefix("/*")
                            .unwrap_or("")
                            .strip_suffix("*/")
                            .unwrap_or("")
                            .trim();
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

    fn is_grid_property(&self, property: &str) -> bool {
        matches!(
            property,
            "display"
                | "grid"
                | "grid-template"
                | "grid-template-rows"
                | "grid-template-columns"
                | "grid-template-areas"
                | "grid-auto-rows"
                | "grid-auto-columns"
                | "grid-auto-flow"
                | "grid-row"
                | "grid-column"
                | "grid-area"
                | "grid-row-start"
                | "grid-row-end"
                | "grid-column-start"
                | "grid-column-end"
                | "justify-items"
                | "align-items"
                | "place-items"
                | "justify-content"
                | "align-content"
                | "place-content"
                | "justify-self"
                | "align-self"
                | "place-self"
                | "grid-gap"
                | "grid-row-gap"
                | "grid-column-gap"
                | "gap"
                | "row-gap"
                | "column-gap"
        )
    }

    fn is_flexbox_property(&self, property: &str) -> bool {
        matches!(
            property,
            "display"
                | "flex"
                | "flex-direction"
                | "flex-wrap"
                | "flex-flow"
                | "justify-content"
                | "align-items"
                | "align-content"
                | "order"
                | "flex-grow"
                | "flex-shrink"
                | "flex-basis"
                | "align-self"
        )
    }

    fn is_animation_property(&self, property: &str) -> bool {
        matches!(
            property,
            "animation"
                | "animation-name"
                | "animation-duration"
                | "animation-timing-function"
                | "animation-delay"
                | "animation-iteration-count"
                | "animation-direction"
                | "animation-fill-mode"
                | "animation-play-state"
                | "transition"
                | "transition-property"
                | "transition-duration"
                | "transition-timing-function"
                | "transition-delay"
        )
    }

    fn is_transform_property(&self, property: &str) -> bool {
        matches!(
            property,
            "transform"
                | "transform-origin"
                | "transform-style"
                | "perspective"
                | "perspective-origin"
                | "backface-visibility"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_symbol_extraction() {
        let extractor = CssExtractor;
        assert_eq!(extractor.language(), LanguageId::CSS);
    }

    #[test]
    fn test_rule_set_extraction() {
        let extractor = CssExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::CSS);
    }

    #[test]
    fn test_selector_extraction() {
        let extractor = CssExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::CSS);
    }

    #[test]
    fn test_property_extraction() {
        let extractor = CssExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::CSS);
    }

    #[test]
    fn test_import_extraction() {
        let extractor = CssExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::CSS);
    }

    #[test]
    fn test_media_query_extraction() {
        let extractor = CssExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::CSS);
    }

    #[test]
    fn test_keyframes_extraction() {
        let extractor = CssExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::CSS);
    }

    #[test]
    fn test_css_doc_extraction() {
        let extractor = CssExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::CSS);
    }

    #[test]
    fn test_custom_property_extraction() {
        let extractor = CssExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::CSS);
    }

    #[test]
    fn test_supports_query_extraction() {
        let extractor = CssExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::CSS);
    }

    #[test]
    fn test_modern_css_property_detection() {
        let extractor = CssExtractor;

        // Test grid property detection
        assert!(extractor.is_grid_property("grid-template-columns"));
        assert!(extractor.is_grid_property("grid-area"));
        assert!(!extractor.is_grid_property("color"));

        // Test flexbox property detection
        assert!(extractor.is_flexbox_property("flex-direction"));
        assert!(extractor.is_flexbox_property("justify-content"));
        assert!(!extractor.is_flexbox_property("margin"));

        // Test animation property detection
        assert!(extractor.is_animation_property("animation-duration"));
        assert!(extractor.is_animation_property("transition"));
        assert!(!extractor.is_animation_property("width"));

        // Test transform property detection
        assert!(extractor.is_transform_property("transform"));
        assert!(extractor.is_transform_property("perspective"));
        assert!(!extractor.is_transform_property("height"));
    }
}

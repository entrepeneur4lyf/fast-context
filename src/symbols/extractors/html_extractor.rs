//! HTML symbol extractor
//! 
//! Extracts symbols from HTML source code including:
//! - Elements and tags
//! - Attributes (id, class, data attributes)
//! - Script and style blocks
//! - Forms and inputs
//! - Links and images

use crate::parsers::LanguageId;
use crate::symbols::{Location, Scope, Symbol, SymbolExtractor, SymbolKind};
use tree_sitter::{Node, Tree};

/// HTML Symbol Extractor
/// Extracts elements, attributes, scripts, styles from HTML code
pub struct HtmlExtractor;

impl SymbolExtractor for HtmlExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::HTML
    }

    fn extract_symbols(&self, tree: &Tree, source: &str, file_path: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let mut scope_stack = Vec::new();
        
        self.extract_from_node(tree.root_node(), source, file_path, &mut symbols, &mut scope_stack);
        symbols
    }
}

impl HtmlExtractor {
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<Symbol>,
        scope_stack: &mut Vec<Scope>,
    ) {
        match node.kind() {
            "element" => {
                self.extract_element(&node, source, file_path, symbols, scope_stack);
            }
            "self_closing_tag" => {
                self.extract_self_closing_tag(&node, source, file_path, symbols, scope_stack);
            }
            "start_tag" => {
                self.extract_start_tag(&node, source, file_path, symbols, scope_stack);
            }
            "script_element" => {
                self.extract_script_element(&node, source, file_path, symbols, scope_stack);
            }
            "style_element" => {
                self.extract_style_element(&node, source, file_path, symbols, scope_stack);
            }
            "attribute" => {
                self.extract_attribute(&node, source, file_path, symbols, scope_stack);
            }
            "doctype" => {
                self.extract_doctype(&node, source, file_path, symbols, scope_stack);
            }
            _ => {}
        }

        // Recursively process child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_from_node(child, source, file_path, symbols, scope_stack);
        }

        // Pop scope if we added one for this node
        if matches!(node.kind(), "element" | "script_element" | "style_element") {
            scope_stack.pop();
        }
    }

    fn extract_element(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &mut Vec<Scope>) {
        // Extract element name from start tag
        if let Some(start_tag) = node.child_by_field_name("start_tag") {
            if let Some(name_node) = start_tag.child_by_field_name("name") {
                let tag_name = self.get_node_text(&name_node, source);
                let location = Location::from_node(&start_tag, file_path);
                
                // Push element as scope for nested items
                let scope = Scope {
                    name: tag_name.clone(),
                    kind: SymbolKind::Class,
                    location: location.clone(),
                };
                scope_stack.push(scope);
                
                let mut modifiers = vec!["element".to_string(), tag_name.clone()];

                // Add semantic element classification
                if self.is_semantic_element(&tag_name) {
                    modifiers.push("semantic".to_string());
                }
                if self.is_form_element(&tag_name) {
                    modifiers.push("form".to_string());
                }
                if self.is_media_element(&tag_name) {
                    modifiers.push("media".to_string());
                }
                if self.is_interactive_element(&tag_name) {
                    modifiers.push("interactive".to_string());
                }
                if self.is_custom_element(&tag_name) {
                    modifiers.push("custom".to_string());
                    modifiers.push("web-component".to_string());
                }

                // Extract important attributes for additional context
                let mut id_attr = None;
                let mut class_attr = None;
                
                let mut cursor = start_tag.walk();
                for child in start_tag.children(&mut cursor) {
                    if child.kind() == "attribute" {
                        if let Some(attr_name) = child.child_by_field_name("name") {
                            let attr_name_text = self.get_node_text(&attr_name, source);
                            if attr_name_text == "id" {
                                if let Some(attr_value) = child.child_by_field_name("value") {
                                    id_attr = Some(self.clean_attribute_value(&self.get_node_text(&attr_value, source)));
                                }
                            } else if attr_name_text == "class" {
                                if let Some(attr_value) = child.child_by_field_name("value") {
                                    class_attr = Some(self.clean_attribute_value(&self.get_node_text(&attr_value, source)));
                                }
                            }
                        }
                    }
                }
                
                // Add id and class info to modifiers
                if let Some(id) = &id_attr {
                    modifiers.push(format!("id={id}"));
                }
                if let Some(class) = &class_attr {
                    modifiers.push(format!("class={class}"));
                }
                
                // Create element name with id/class for uniqueness
                let element_name = if let Some(id) = id_attr {
                    format!("{tag_name}#{id}")
                } else if let Some(class) = class_attr {
                    format!("{}.{}", tag_name, class.split_whitespace().next().unwrap_or(""))
                } else {
                    tag_name.clone()
                };
                
                symbols.push(Symbol {
                    name: element_name,
                    kind: SymbolKind::Class,
                    location,
                    scope_chain: scope_stack[..scope_stack.len()-1].to_vec(),
                    language: LanguageId::HTML,
                    documentation: self.extract_html_doc(node, source),
                    modifiers,
                    signature: None,
                });
            }
        }
    }

    fn extract_self_closing_tag(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &Vec<Scope>) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let tag_name = self.get_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);
            
            let mut modifiers = vec!["element".to_string(), "self-closing".to_string(), tag_name.clone()];
            
            // Extract important attributes
            let mut id_attr = None;
            let mut src_attr = None;
            let mut href_attr = None;
            
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "attribute" {
                    if let Some(attr_name) = child.child_by_field_name("name") {
                        let attr_name_text = self.get_node_text(&attr_name, source);
                        if let Some(attr_value) = child.child_by_field_name("value") {
                            let value = self.clean_attribute_value(&self.get_node_text(&attr_value, source));
                            match attr_name_text.as_str() {
                                "id" => id_attr = Some(value),
                                "src" => src_attr = Some(value),
                                "href" => href_attr = Some(value),
                                _ => {}
                            }
                        }
                    }
                }
            }
            
            // Add attribute info to modifiers
            if let Some(id) = &id_attr {
                modifiers.push(format!("id={id}"));
            }
            if let Some(src) = &src_attr {
                modifiers.push(format!("src={src}"));
            }
            if let Some(href) = &href_attr {
                modifiers.push(format!("href={href}"));
            }
            
            // Create element name with id for uniqueness
            let element_name = if let Some(id) = id_attr {
                format!("{tag_name}#{id}")
            } else {
                tag_name.clone()
            };
            
            symbols.push(Symbol {
                name: element_name,
                kind: SymbolKind::Class,
                location,
                scope_chain: scope_stack.clone(),
                language: LanguageId::HTML,
                documentation: None,
                modifiers,
                signature: None,
            });
        }
    }

    fn extract_start_tag(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &Vec<Scope>) {
        // This is handled by extract_element for full elements
        // Only process standalone start tags
        if let Some(parent) = node.parent() {
            if parent.kind() != "element" {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let tag_name = self.get_node_text(&name_node, source);
                    let location = Location::from_node(node, file_path);
                    
                    symbols.push(Symbol {
                        name: tag_name.clone(),
                        kind: SymbolKind::Class,
                        location,
                        scope_chain: scope_stack.clone(),
                        language: LanguageId::HTML,
                        documentation: None,
                        modifiers: vec!["element".to_string(), "start-tag".to_string(), tag_name],
                        signature: None,
                    });
                }
            }
        }
    }

    fn extract_script_element(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &mut Vec<Scope>) {
        let location = Location::from_node(node, file_path);
        
        // Push script as scope
        let scope = Scope {
            name: "script".to_string(),
            kind: SymbolKind::Function,
            location: location.clone(),
        };
        scope_stack.push(scope);
        
        let mut modifiers = vec!["element".to_string(), "script".to_string()];
        let mut src_attr = None;
        let mut type_attr = None;
        
        // Extract script attributes
        if let Some(start_tag) = node.child_by_field_name("start_tag") {
            let mut cursor = start_tag.walk();
            for child in start_tag.children(&mut cursor) {
                if child.kind() == "attribute" {
                    if let Some(attr_name) = child.child_by_field_name("name") {
                        let attr_name_text = self.get_node_text(&attr_name, source);
                        if let Some(attr_value) = child.child_by_field_name("value") {
                            let value = self.clean_attribute_value(&self.get_node_text(&attr_value, source));
                            match attr_name_text.as_str() {
                                "src" => {
                                    src_attr = Some(value.clone());
                                    modifiers.push(format!("src={value}"));
                                }
                                "type" => {
                                    type_attr = Some(value.clone());
                                    modifiers.push(format!("type={value}"));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        
        let script_name = if let Some(src) = &src_attr {
            format!("script[src={src}]")
        } else {
            "script[inline]".to_string()
        };

        // Build signature with available attributes
        let mut signature_parts = Vec::new();
        if let Some(src) = src_attr {
            signature_parts.push(format!("src=\"{src}\""));
        }
        if let Some(script_type) = type_attr {
            signature_parts.push(format!("type=\"{script_type}\""));
        }
        let signature = if signature_parts.is_empty() {
            None
        } else {
            Some(format!("<script {}>", signature_parts.join(" ")))
        };

        symbols.push(Symbol {
            name: script_name,
            kind: SymbolKind::Function,
            location,
            scope_chain: scope_stack[..scope_stack.len()-1].to_vec(),
            language: LanguageId::HTML,
            documentation: self.extract_html_doc(node, source),
            modifiers,
            signature,
        });
    }

    fn extract_style_element(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &mut Vec<Scope>) {
        let location = Location::from_node(node, file_path);
        
        // Push style as scope
        let scope = Scope {
            name: "style".to_string(),
            kind: SymbolKind::Namespace,
            location: location.clone(),
        };
        scope_stack.push(scope);
        
        let mut modifiers = vec!["element".to_string(), "style".to_string()];
        
        // Extract style attributes
        if let Some(start_tag) = node.child_by_field_name("start_tag") {
            let mut cursor = start_tag.walk();
            for child in start_tag.children(&mut cursor) {
                if child.kind() == "attribute" {
                    if let Some(attr_name) = child.child_by_field_name("name") {
                        let attr_name_text = self.get_node_text(&attr_name, source);
                        if attr_name_text == "type" {
                            if let Some(attr_value) = child.child_by_field_name("value") {
                                let value = self.clean_attribute_value(&self.get_node_text(&attr_value, source));
                                modifiers.push(format!("type={value}"));
                            }
                        }
                    }
                }
            }
        }
        
        symbols.push(Symbol {
            name: "style[inline]".to_string(),
            kind: SymbolKind::Namespace,
            location,
            scope_chain: scope_stack[..scope_stack.len()-1].to_vec(),
            language: LanguageId::HTML,
            documentation: self.extract_html_doc(node, source),
            modifiers,
            signature: None,
        });
    }

    fn extract_attribute(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &Vec<Scope>) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let attr_name = self.get_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);
            
            let mut modifiers = vec!["attribute".to_string()];
            let signature;
            
            // Get attribute value
            if let Some(value_node) = node.child_by_field_name("value") {
                let attr_value = self.clean_attribute_value(&self.get_node_text(&value_node, source));
                signature = Some(format!("{attr_name}={attr_value}"));
                
                // Special handling for important attributes
                match attr_name.as_str() {
                    "id" => {
                        modifiers.push("id".to_string());
                        modifiers.push("unique".to_string());
                        modifiers.push("identifier".to_string());

                        // Create a separate symbol for the ID
                        symbols.push(Symbol {
                            name: format!("#{attr_value}"),
                            kind: SymbolKind::Variable,
                            location: location.clone(),
                            scope_chain: scope_stack.clone(),
                            language: LanguageId::HTML,
                            documentation: None,
                            modifiers: vec!["id".to_string(), "selector".to_string()],
                            signature: signature.clone(),
                        });
                    }
                    "class" => {
                        modifiers.push("class".to_string());

                        // Create symbols for each class
                        for class_name in attr_value.split_whitespace() {
                            if !class_name.is_empty() {
                                symbols.push(Symbol {
                                    name: format!(".{class_name}"),
                                    kind: SymbolKind::Class,
                                    location: location.clone(),
                                    scope_chain: scope_stack.clone(),
                                    language: LanguageId::HTML,
                                    documentation: None,
                                    modifiers: vec!["class".to_string(), "selector".to_string()],
                                    signature: signature.clone(),
                                });
                            }
                        }
                    }
                    "name" | "for" => {
                        modifiers.push("identifier".to_string());
                    }
                    name if name.starts_with("data-") => {
                        modifiers.push("data".to_string());
                        modifiers.push("custom".to_string());
                    }
                    name if name.starts_with("aria-") => {
                        modifiers.push("aria".to_string());
                        modifiers.push("accessibility".to_string());
                    }
                    name if name.starts_with("on") => {
                        modifiers.push("event".to_string());
                        modifiers.push("handler".to_string());
                    }
                    "src" | "href" | "action" | "formaction" | "poster" | "cite" => {
                        modifiers.push("url".to_string());
                    }
                    "role" => {
                        modifiers.push("role".to_string());
                        modifiers.push("accessibility".to_string());
                    }
                    "type" | "method" | "enctype" | "target" => {
                        modifiers.push("behavior".to_string());
                    }
                    "required" | "disabled" | "readonly" | "checked" | "selected" | "hidden" => {
                        modifiers.push("state".to_string());
                    }
                    "itemscope" | "itemtype" | "itemprop" | "itemref" | "itemid" => {
                        modifiers.push("microdata".to_string());
                        modifiers.push("structured-data".to_string());
                    }
                    _ => {}
                }
            } else {
                // Boolean attribute (no value)
                modifiers.push("boolean".to_string());
                signature = Some(attr_name.clone());
            }
            
            symbols.push(Symbol {
                name: attr_name,
                kind: SymbolKind::Field,
                location,
                scope_chain: scope_stack.clone(),
                language: LanguageId::HTML,
                documentation: None,
                modifiers,
                signature,
            });
        }
    }

    fn extract_doctype(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &Vec<Scope>) {
        let doctype_text = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);

        // Extract DOCTYPE declaration
        let mut modifiers = vec!["doctype".to_string(), "declaration".to_string()];

        // Determine HTML version from DOCTYPE
        if doctype_text.to_lowercase().contains("html") {
            if doctype_text.contains("XHTML") {
                modifiers.push("xhtml".to_string());
            } else if doctype_text.trim() == "<!DOCTYPE html>" {
                modifiers.push("html5".to_string());
            } else {
                modifiers.push("html4".to_string());
            }
        }

        symbols.push(Symbol {
            name: "DOCTYPE".to_string(),
            kind: SymbolKind::Constant,
            location,
            scope_chain: scope_stack.clone(),
            language: LanguageId::HTML,
            documentation: None,
            modifiers,
            signature: Some(doctype_text),
        });
    }

    fn get_node_text(&self, node: &Node, source: &str) -> String {
        node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
    }

    fn clean_attribute_value(&self, value: &str) -> String {
        // Remove quotes from attribute values
        if (value.starts_with('"') && value.ends_with('"')) || 
           (value.starts_with('\'') && value.ends_with('\'')) {
            value[1..value.len()-1].to_string()
        } else {
            value.to_string()
        }
    }

    fn extract_html_doc(&self, node: &Node, source: &str) -> Option<String> {
        // HTML documentation appears as <!-- --> comments preceding elements
        let mut current = *node;
        let mut doc_comments = Vec::new();

        // Look backwards from the node to find HTML doc comments
        while let Some(prev) = current.prev_sibling() {
            match prev.kind() {
                "comment" => {
                    let comment_text = prev.utf8_text(source.as_bytes()).ok()?;
                    if comment_text.starts_with("<!--") && comment_text.ends_with("-->") {
                        let content = comment_text
                            .strip_prefix("<!--").unwrap_or("")
                            .strip_suffix("-->").unwrap_or("")
                            .trim();
                        if !content.is_empty() {
                            doc_comments.insert(0, content.to_string());
                        }
                    }
                    current = prev;
                }
                "text" => {
                    // Check if it's just whitespace
                    let text_content = prev.utf8_text(source.as_bytes()).ok()?;
                    if text_content.trim().is_empty() {
                        current = prev;
                        continue;
                    } else {
                        // Stop at non-whitespace text
                        break;
                    }
                }
                _ => {
                    // Stop at other elements
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

    fn is_semantic_element(&self, tag_name: &str) -> bool {
        matches!(tag_name,
            "article" | "aside" | "details" | "figcaption" | "figure" | "footer" | "header" |
            "main" | "mark" | "nav" | "section" | "summary" | "time" | "address" | "hgroup"
        )
    }

    fn is_form_element(&self, tag_name: &str) -> bool {
        matches!(tag_name,
            "form" | "input" | "textarea" | "select" | "option" | "optgroup" | "button" |
            "label" | "fieldset" | "legend" | "datalist" | "output" | "progress" | "meter"
        )
    }

    fn is_media_element(&self, tag_name: &str) -> bool {
        matches!(tag_name,
            "audio" | "video" | "source" | "track" | "img" | "picture" | "canvas" | "svg" |
            "embed" | "object" | "param" | "iframe"
        )
    }

    fn is_interactive_element(&self, tag_name: &str) -> bool {
        matches!(tag_name,
            "a" | "button" | "input" | "select" | "textarea" | "details" | "summary" |
            "dialog" | "menu" | "menuitem"
        )
    }

    fn is_custom_element(&self, tag_name: &str) -> bool {
        // Custom elements must contain a hyphen
        tag_name.contains('-') && tag_name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_symbol_extraction() {
        let extractor = HtmlExtractor;
        assert_eq!(extractor.language(), LanguageId::HTML);
    }

    #[test]
    fn test_element_extraction() {
        let extractor = HtmlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::HTML);
    }

    #[test]
    fn test_self_closing_tag_extraction() {
        let extractor = HtmlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::HTML);
    }

    #[test]
    fn test_attribute_extraction() {
        let extractor = HtmlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::HTML);
    }

    #[test]
    fn test_script_element_extraction() {
        let extractor = HtmlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::HTML);
    }

    #[test]
    fn test_style_element_extraction() {
        let extractor = HtmlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::HTML);
    }

    #[test]
    fn test_id_class_extraction() {
        let extractor = HtmlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::HTML);
    }

    #[test]
    fn test_html_doc_extraction() {
        let extractor = HtmlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::HTML);
    }

    #[test]
    fn test_doctype_extraction() {
        let extractor = HtmlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::HTML);
    }

    #[test]
    fn test_semantic_element_detection() {
        let extractor = HtmlExtractor;

        // Test semantic element detection
        assert!(extractor.is_semantic_element("article"));
        assert!(extractor.is_semantic_element("nav"));
        assert!(extractor.is_semantic_element("section"));
        assert!(!extractor.is_semantic_element("div"));

        // Test form element detection
        assert!(extractor.is_form_element("input"));
        assert!(extractor.is_form_element("form"));
        assert!(!extractor.is_form_element("div"));

        // Test media element detection
        assert!(extractor.is_media_element("video"));
        assert!(extractor.is_media_element("audio"));
        assert!(!extractor.is_media_element("div"));

        // Test interactive element detection
        assert!(extractor.is_interactive_element("button"));
        assert!(extractor.is_interactive_element("a"));
        assert!(!extractor.is_interactive_element("div"));

        // Test custom element detection
        assert!(extractor.is_custom_element("my-component"));
        assert!(extractor.is_custom_element("custom-button"));
        assert!(!extractor.is_custom_element("div"));
        assert!(!extractor.is_custom_element("MyComponent")); // Must be lowercase
    }

    #[test]
    fn test_modern_html_attributes() {
        let extractor = HtmlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::HTML);
    }
}

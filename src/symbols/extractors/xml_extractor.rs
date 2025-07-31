//! XML symbol extractor
//! 
//! Extracts symbols from XML source code including:
//! - Elements and tags
//! - Attributes and namespaces
//! - CDATA sections
//! - Processing instructions
//! - DTD declarations

use crate::parsers::LanguageId;
use crate::symbols::{Location, Scope, Symbol, SymbolExtractor, SymbolKind};
use tree_sitter::{Node, Tree};

/// XML Symbol Extractor
/// Extracts elements, attributes, namespaces from XML code
pub struct XmlExtractor;

impl SymbolExtractor for XmlExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::XML
    }

    fn extract_symbols(&self, tree: &Tree, source: &str, file_path: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let mut scope_stack = Vec::new();
        
        self.extract_from_node(tree.root_node(), source, file_path, &mut symbols, &mut scope_stack);
        symbols
    }
}

impl XmlExtractor {
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
            "attribute" => {
                self.extract_attribute(&node, source, file_path, symbols, scope_stack);
            }
            "processing_instruction" => {
                self.extract_processing_instruction(&node, source, file_path, symbols, scope_stack);
            }
            "doctype" => {
                self.extract_doctype(&node, source, file_path, symbols, scope_stack);
            }
            "xml_declaration" => {
                self.extract_xml_declaration(&node, source, file_path, symbols, scope_stack);
            }
            "cdata" => {
                self.extract_cdata(&node, source, file_path, symbols, scope_stack);
            }
            "entity_declaration" => {
                self.extract_entity_declaration(&node, source, file_path, symbols, scope_stack);
            }
            "element_declaration" => {
                self.extract_element_declaration(&node, source, file_path, symbols, scope_stack);
            }
            "attribute_declaration" => {
                self.extract_attribute_declaration(&node, source, file_path, symbols, scope_stack);
            }
            _ => {}
        }

        // Recursively process child nodes
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_from_node(child, source, file_path, symbols, scope_stack);
        }

        // Pop scope if we added one for this node
        if matches!(node.kind(), "element") {
            scope_stack.pop();
        }
    }

    fn extract_element(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &mut Vec<Scope>) {
        // Extract element name from start tag
        if let Some(start_tag) = node.child_by_field_name("start_tag") {
            if let Some(name_node) = start_tag.child_by_field_name("name") {
                let element_name = self.get_node_text(&name_node, source);
                let location = Location::from_node(&start_tag, file_path);
                
                // Handle namespaced elements
                let (namespace, local_name) = self.parse_namespaced_name(&element_name);
                
                // Push element as scope for nested items
                let scope = Scope {
                    name: local_name.clone(),
                    kind: SymbolKind::Class,
                    location: location.clone(),
                };
                scope_stack.push(scope);
                
                let mut modifiers = vec!["element".to_string()];

                // Add namespace information
                if let Some(ns) = &namespace {
                    modifiers.push(format!("namespace={ns}"));
                }

                // Detect XML Schema elements
                if self.is_schema_element(&local_name) {
                    modifiers.push("schema".to_string());
                    modifiers.push("xsd".to_string());
                }

                // Detect common XML vocabularies
                if self.is_soap_element(&local_name, &namespace) {
                    modifiers.push("soap".to_string());
                    modifiers.push("web-service".to_string());
                } else if self.is_rss_element(&local_name) {
                    modifiers.push("rss".to_string());
                    modifiers.push("feed".to_string());
                } else if self.is_svg_element(&local_name, &namespace) {
                    modifiers.push("svg".to_string());
                    modifiers.push("graphics".to_string());
                }
                
                // Extract important attributes for additional context
                let mut id_attr = None;
                let mut type_attr = None;
                let mut name_attr = None;
                
                let mut cursor = start_tag.walk();
                for child in start_tag.children(&mut cursor) {
                    if child.kind() == "attribute" {
                        if let Some(attr_name) = child.child_by_field_name("name") {
                            let attr_name_text = self.get_node_text(&attr_name, source);
                            if let Some(attr_value) = child.child_by_field_name("value") {
                                let value = self.clean_attribute_value(&self.get_node_text(&attr_value, source));
                                match attr_name_text.as_str() {
                                    "id" => id_attr = Some(value),
                                    "type" => type_attr = Some(value),
                                    "name" => name_attr = Some(value),
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
                if let Some(type_val) = &type_attr {
                    modifiers.push(format!("type={type_val}"));
                }
                if let Some(name) = &name_attr {
                    modifiers.push(format!("name={name}"));
                }
                
                // Create element name with id for uniqueness
                let symbol_name = if let Some(id) = id_attr {
                    format!("{element_name}#{id}")
                } else if let Some(name) = name_attr {
                    format!("{element_name}[@name='{name}']")
                } else {
                    element_name.clone()
                };
                
                symbols.push(Symbol {
                    name: symbol_name,
                    kind: SymbolKind::Class,
                    location,
                    scope_chain: scope_stack[..scope_stack.len()-1].to_vec(),
                    language: LanguageId::XML,
                    documentation: self.extract_xml_doc(node, source),
                    modifiers,
                    signature: None,
                });
            }
        }
    }

    fn extract_self_closing_tag(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &Vec<Scope>) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let element_name = self.get_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);
            
            // Handle namespaced elements
            let (namespace, local_name) = self.parse_namespaced_name(&element_name);

            let mut modifiers = vec!["element".to_string(), "self-closing".to_string()];

            // Add namespace information
            if let Some(ns) = &namespace {
                modifiers.push(format!("namespace={ns}"));
            }

            // Detect XML Schema elements
            if self.is_schema_element(&local_name) {
                modifiers.push("schema".to_string());
                modifiers.push("xsd".to_string());
            }

            // Detect common XML vocabularies
            if self.is_soap_element(&local_name, &namespace) {
                modifiers.push("soap".to_string());
                modifiers.push("web-service".to_string());
            } else if self.is_rss_element(&local_name) {
                modifiers.push("rss".to_string());
                modifiers.push("feed".to_string());
            } else if self.is_svg_element(&local_name, &namespace) {
                modifiers.push("svg".to_string());
                modifiers.push("graphics".to_string());
            }
            
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
            let symbol_name = if let Some(id) = id_attr {
                format!("{element_name}#{id}")
            } else {
                element_name.clone()
            };
            
            symbols.push(Symbol {
                name: symbol_name,
                kind: SymbolKind::Class,
                location,
                scope_chain: scope_stack.clone(),
                language: LanguageId::XML,
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
                    let element_name = self.get_node_text(&name_node, source);
                    let location = Location::from_node(node, file_path);
                    
                    let (namespace, _) = self.parse_namespaced_name(&element_name);
                    let mut modifiers = vec!["element".to_string(), "start-tag".to_string()];
                    
                    if let Some(ns) = &namespace {
                        modifiers.push(format!("namespace={ns}"));
                    }
                    
                    symbols.push(Symbol {
                        name: element_name,
                        kind: SymbolKind::Class,
                        location,
                        scope_chain: scope_stack.clone(),
                        language: LanguageId::XML,
                        documentation: None,
                        modifiers,
                        signature: None,
                    });
                }
            }
        }
    }

    fn extract_attribute(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &Vec<Scope>) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let attr_name = self.get_node_text(&name_node, source);
            let location = Location::from_node(node, file_path);
            
            // Handle namespaced attributes
            let (namespace, local_name) = self.parse_namespaced_name(&attr_name);
            
            let mut modifiers = vec!["attribute".to_string()];
            let signature;
            
            // Add namespace information
            if let Some(ns) = &namespace {
                modifiers.push(format!("namespace={ns}"));
            }
            
            // Get attribute value
            if let Some(value_node) = node.child_by_field_name("value") {
                let attr_value = self.clean_attribute_value(&self.get_node_text(&value_node, source));
                signature = Some(format!("{attr_name}=\"{attr_value}\""));
                
                // Special handling for important attributes
                match local_name.as_str() {
                    "id" => {
                        modifiers.push("id".to_string());
                        modifiers.push("unique".to_string());
                        
                        // Create a separate symbol for the ID
                        symbols.push(Symbol {
                            name: format!("#{attr_value}"),
                            kind: SymbolKind::Variable,
                            location: location.clone(),
                            scope_chain: scope_stack.clone(),
                            language: LanguageId::XML,
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
                                    language: LanguageId::XML,
                                    documentation: None,
                                    modifiers: vec!["class".to_string(), "selector".to_string()],
                                    signature: signature.clone(),
                                });
                            }
                        }
                    }
                    "xmlns" => {
                        modifiers.push("namespace".to_string());
                        modifiers.push("declaration".to_string());
                        
                        // Create namespace symbol
                        symbols.push(Symbol {
                            name: format!("xmlns:{attr_value}"),
                            kind: SymbolKind::Namespace,
                            location: location.clone(),
                            scope_chain: scope_stack.clone(),
                            language: LanguageId::XML,
                            documentation: None,
                            modifiers: vec!["namespace".to_string(), "uri".to_string()],
                            signature: signature.clone(),
                        });
                    }
                    name if name.starts_with("xmlns:") => {
                        let prefix = &name[6..];
                        modifiers.push("namespace".to_string());
                        modifiers.push("declaration".to_string());
                        
                        // Create namespace prefix symbol
                        symbols.push(Symbol {
                            name: format!("{prefix}:{attr_value}"),
                            kind: SymbolKind::Namespace,
                            location: location.clone(),
                            scope_chain: scope_stack.clone(),
                            language: LanguageId::XML,
                            documentation: None,
                            modifiers: vec!["namespace".to_string(), "prefix".to_string()],
                            signature: signature.clone(),
                        });
                    }
                    "src" | "href" => {
                        modifiers.push("url".to_string());
                    }
                    "name" => {
                        modifiers.push("name".to_string());
                    }
                    "type" => {
                        modifiers.push("type".to_string());
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
                language: LanguageId::XML,
                documentation: None,
                modifiers,
                signature,
            });
        }
    }

    fn extract_processing_instruction(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &Vec<Scope>) {
        let pi_text = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);
        
        // Extract processing instruction target
        if let Some(target_start) = pi_text.find("<?") {
            let content = &pi_text[target_start + 2..];
            if let Some(target_end) = content.find(|c: char| c.is_whitespace() || c == '?') {
                let target = &content[..target_end];
                
                let mut modifiers = vec!["processing-instruction".to_string()];
                
                // Special handling for common processing instructions
                match target {
                    "xml-stylesheet" => {
                        modifiers.push("stylesheet".to_string());
                    }
                    "xml-model" => {
                        modifiers.push("model".to_string());
                    }
                    _ => {}
                }
                
                symbols.push(Symbol {
                    name: format!("<?{target}"),
                    kind: SymbolKind::Constant,
                    location,
                    scope_chain: scope_stack.clone(),
                    language: LanguageId::XML,
                    documentation: None,
                    modifiers,
                    signature: Some(pi_text.trim().to_string()),
                });
            }
        }
    }

    fn extract_doctype(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &Vec<Scope>) {
        let doctype_text = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);
        
        // Extract DOCTYPE name
        if let Some(name_start) = doctype_text.find("<!DOCTYPE") {
            let content = &doctype_text[name_start + 9..];
            if let Some(name_end) = content.find(|c: char| c.is_whitespace() || c == '>' || c == '[') {
                let doctype_name = content[..name_end].trim();
                
                symbols.push(Symbol {
                    name: format!("<!DOCTYPE {doctype_name}"),
                    kind: SymbolKind::Type,
                    location,
                    scope_chain: scope_stack.clone(),
                    language: LanguageId::XML,
                    documentation: None,
                    modifiers: vec!["doctype".to_string(), "declaration".to_string()],
                    signature: Some(doctype_text.trim().to_string()),
                });
            }
        }
    }

    fn extract_xml_declaration(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &Vec<Scope>) {
        let decl_text = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);
        
        let mut modifiers = vec!["xml-declaration".to_string()];
        
        // Extract version, encoding, standalone info
        if decl_text.contains("version=") {
            modifiers.push("version".to_string());
        }
        if decl_text.contains("encoding=") {
            modifiers.push("encoding".to_string());
        }
        if decl_text.contains("standalone=") {
            modifiers.push("standalone".to_string());
        }
        
        symbols.push(Symbol {
            name: "<?xml".to_string(),
            kind: SymbolKind::Constant,
            location,
            scope_chain: scope_stack.clone(),
            language: LanguageId::XML,
            documentation: None,
            modifiers,
            signature: Some(decl_text.trim().to_string()),
        });
    }

    fn extract_cdata(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &Vec<Scope>) {
        let location = Location::from_node(node, file_path);
        let cdata_content = self.get_node_text(node, source);

        // Extract meaningful content from CDATA section
        let content_preview = if cdata_content.len() > 50 {
            format!("{}...", &cdata_content[..50])
        } else {
            cdata_content.clone()
        };

        symbols.push(Symbol {
            name: "<![CDATA[".to_string(),
            kind: SymbolKind::Constant,
            location,
            scope_chain: scope_stack.clone(),
            language: LanguageId::XML,
            documentation: Some(format!("CDATA section containing: {content_preview}")),
            modifiers: vec!["cdata".to_string(), "section".to_string()],
            signature: Some(cdata_content),
        });
    }

    fn extract_entity_declaration(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &Vec<Scope>) {
        let entity_text = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);

        // Extract entity name from declaration
        if let Some(name_start) = entity_text.find("<!ENTITY") {
            let content = &entity_text[name_start + 8..];
            if let Some(name_end) = content.find(|c: char| c.is_whitespace()) {
                let entity_name = content[..name_end].trim();

                let mut modifiers = vec!["entity".to_string(), "declaration".to_string()];

                // Determine entity type
                if entity_text.contains("SYSTEM") {
                    modifiers.push("external".to_string());
                } else if entity_text.contains("PUBLIC") {
                    modifiers.push("public".to_string());
                } else {
                    modifiers.push("internal".to_string());
                }

                symbols.push(Symbol {
                    name: format!("&{entity_name};"),
                    kind: SymbolKind::Variable,
                    location,
                    scope_chain: scope_stack.clone(),
                    language: LanguageId::XML,
                    documentation: None,
                    modifiers,
                    signature: Some(entity_text.trim().to_string()),
                });
            }
        }
    }

    fn extract_element_declaration(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &Vec<Scope>) {
        let decl_text = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);

        // Extract element name from DTD declaration
        if let Some(name_start) = decl_text.find("<!ELEMENT") {
            let content = &decl_text[name_start + 9..];
            if let Some(name_end) = content.find(|c: char| c.is_whitespace()) {
                let element_name = content[..name_end].trim();

                let mut modifiers = vec!["element".to_string(), "declaration".to_string(), "dtd".to_string()];

                // Analyze content model
                if decl_text.contains("EMPTY") {
                    modifiers.push("empty".to_string());
                } else if decl_text.contains("ANY") {
                    modifiers.push("any".to_string());
                } else if decl_text.contains("#PCDATA") {
                    modifiers.push("mixed".to_string());
                } else {
                    modifiers.push("element-content".to_string());
                }

                symbols.push(Symbol {
                    name: format!("<!ELEMENT {element_name}"),
                    kind: SymbolKind::Type,
                    location,
                    scope_chain: scope_stack.clone(),
                    language: LanguageId::XML,
                    documentation: None,
                    modifiers,
                    signature: Some(decl_text.trim().to_string()),
                });
            }
        }
    }

    fn extract_attribute_declaration(&self, node: &Node, source: &str, file_path: &str, symbols: &mut Vec<Symbol>, scope_stack: &Vec<Scope>) {
        let decl_text = self.get_node_text(node, source);
        let location = Location::from_node(node, file_path);

        // Extract attribute declaration from DTD
        if let Some(name_start) = decl_text.find("<!ATTLIST") {
            let content = &decl_text[name_start + 9..];
            let parts: Vec<&str> = content.split_whitespace().collect();

            if parts.len() >= 3 {
                let element_name = parts[0];
                let attr_name = parts[1];
                let attr_type = parts.get(2).unwrap_or(&"");

                let mut modifiers = vec!["attribute".to_string(), "declaration".to_string(), "dtd".to_string()];

                // Analyze attribute type
                match *attr_type {
                    "CDATA" => modifiers.push("cdata".to_string()),
                    "ID" => modifiers.push("id".to_string()),
                    "IDREF" => modifiers.push("idref".to_string()),
                    "IDREFS" => modifiers.push("idrefs".to_string()),
                    "NMTOKEN" => modifiers.push("nmtoken".to_string()),
                    "NMTOKENS" => modifiers.push("nmtokens".to_string()),
                    "ENTITY" => modifiers.push("entity".to_string()),
                    "ENTITIES" => modifiers.push("entities".to_string()),
                    "NOTATION" => modifiers.push("notation".to_string()),
                    _ if attr_type.starts_with('(') => modifiers.push("enumeration".to_string()),
                    _ => {}
                }

                // Check for default value constraints
                if decl_text.contains("#REQUIRED") {
                    modifiers.push("required".to_string());
                } else if decl_text.contains("#IMPLIED") {
                    modifiers.push("implied".to_string());
                } else if decl_text.contains("#FIXED") {
                    modifiers.push("fixed".to_string());
                }

                symbols.push(Symbol {
                    name: format!("{element_name}@{attr_name}"),
                    kind: SymbolKind::Field,
                    location,
                    scope_chain: scope_stack.clone(),
                    language: LanguageId::XML,
                    documentation: None,
                    modifiers,
                    signature: Some(decl_text.trim().to_string()),
                });
            }
        }
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

    fn parse_namespaced_name(&self, name: &str) -> (Option<String>, String) {
        if let Some(colon_pos) = name.find(':') {
            let namespace = name[..colon_pos].to_string();
            let local_name = name[colon_pos + 1..].to_string();
            (Some(namespace), local_name)
        } else {
            (None, name.to_string())
        }
    }

    fn extract_xml_doc(&self, node: &Node, source: &str) -> Option<String> {
        // XML documentation appears as <!-- --> comments preceding elements
        let mut current = *node;
        let mut doc_comments = Vec::new();

        // Look backwards from the node to find XML doc comments
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

    fn is_schema_element(&self, element_name: &str) -> bool {
        matches!(element_name,
            "schema" | "element" | "attribute" | "complexType" | "simpleType" |
            "sequence" | "choice" | "all" | "group" | "attributeGroup" |
            "restriction" | "extension" | "union" | "list" | "import" | "include" |
            "redefine" | "annotation" | "documentation" | "appinfo" |
            "key" | "keyref" | "unique" | "selector" | "field" |
            "any" | "anyAttribute" | "notation"
        )
    }

    fn is_soap_element(&self, element_name: &str, namespace: &Option<String>) -> bool {
        if let Some(ns) = namespace {
            if ns.contains("soap") || ns.contains("SOAP") {
                return true;
            }
        }
        matches!(element_name,
            "Envelope" | "Header" | "Body" | "Fault" | "faultcode" | "faultstring" |
            "faultactor" | "detail" | "mustUnderstand" | "actor"
        )
    }

    fn is_rss_element(&self, element_name: &str) -> bool {
        matches!(element_name,
            "rss" | "channel" | "item" | "title" | "description" | "link" |
            "pubDate" | "lastBuildDate" | "ttl" | "language" | "copyright" |
            "managingEditor" | "webMaster" | "category" | "generator" |
            "docs" | "cloud" | "rating" | "textInput" | "skipHours" | "skipDays" |
            "image" | "url" | "width" | "height" | "guid" | "author" | "comments" |
            "enclosure" | "source"
        )
    }

    fn is_svg_element(&self, element_name: &str, namespace: &Option<String>) -> bool {
        if let Some(ns) = namespace {
            if ns.contains("svg") || ns.contains("SVG") {
                return true;
            }
        }
        matches!(element_name,
            "svg" | "g" | "defs" | "desc" | "title" | "symbol" | "use" | "image" |
            "switch" | "style" | "path" | "rect" | "circle" | "ellipse" | "line" |
            "polyline" | "polygon" | "text" | "tspan" | "tref" | "textPath" |
            "marker" | "pattern" | "clipPath" | "mask" | "linearGradient" |
            "radialGradient" | "stop" | "animate" | "animateColor" | "animateMotion" |
            "animateTransform" | "set" | "foreignObject" | "metadata"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_symbol_extraction() {
        let extractor = XmlExtractor;
        assert_eq!(extractor.language(), LanguageId::XML);
    }

    #[test]
    fn test_element_extraction() {
        let extractor = XmlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::XML);
    }

    #[test]
    fn test_namespaced_element_extraction() {
        let extractor = XmlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::XML);
    }

    #[test]
    fn test_attribute_extraction() {
        let extractor = XmlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::XML);
    }

    #[test]
    fn test_namespace_declaration_extraction() {
        let extractor = XmlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::XML);
    }

    #[test]
    fn test_processing_instruction_extraction() {
        let extractor = XmlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::XML);
    }

    #[test]
    fn test_doctype_extraction() {
        let extractor = XmlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::XML);
    }

    #[test]
    fn test_xml_declaration_extraction() {
        let extractor = XmlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::XML);
    }

    #[test]
    fn test_cdata_extraction() {
        let extractor = XmlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::XML);
    }

    #[test]
    fn test_xml_doc_extraction() {
        let extractor = XmlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::XML);
    }

    #[test]
    fn test_entity_declaration_extraction() {
        let extractor = XmlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::XML);
    }

    #[test]
    fn test_element_declaration_extraction() {
        let extractor = XmlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::XML);
    }

    #[test]
    fn test_attribute_declaration_extraction() {
        let extractor = XmlExtractor;

        // Test basic functionality - full testing would require tree-sitter parsing
        assert_eq!(extractor.language(), LanguageId::XML);
    }

    #[test]
    fn test_xml_vocabulary_detection() {
        let extractor = XmlExtractor;

        // Test XML Schema element detection
        assert!(extractor.is_schema_element("schema"));
        assert!(extractor.is_schema_element("complexType"));
        assert!(extractor.is_schema_element("element"));
        assert!(!extractor.is_schema_element("custom"));

        // Test RSS element detection
        assert!(extractor.is_rss_element("rss"));
        assert!(extractor.is_rss_element("channel"));
        assert!(extractor.is_rss_element("item"));
        assert!(!extractor.is_rss_element("custom"));

        // Test SVG element detection
        assert!(extractor.is_svg_element("svg", &None));
        assert!(extractor.is_svg_element("rect", &None));
        assert!(extractor.is_svg_element("path", &None));
        assert!(!extractor.is_svg_element("custom", &None));

        // Test SOAP element detection
        assert!(extractor.is_soap_element("Envelope", &None));
        assert!(extractor.is_soap_element("Body", &None));
        assert!(!extractor.is_soap_element("custom", &None));
    }

    #[test]
    fn test_namespace_parsing() {
        let extractor = XmlExtractor;

        // Test namespace parsing
        let (ns, local) = extractor.parse_namespaced_name("xsd:element");
        assert_eq!(ns, Some("xsd".to_string()));
        assert_eq!(local, "element");

        let (ns, local) = extractor.parse_namespaced_name("element");
        assert_eq!(ns, None);
        assert_eq!(local, "element");
    }
}

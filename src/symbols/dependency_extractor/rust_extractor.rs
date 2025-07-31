//! Rust-specific dependency extraction
//! 
//! Extracts dependency relationships from Rust source code, including:
//! - Function calls and method invocations
//! - Variable references and assignments  
//! - Module imports and use statements
//! - Trait implementations and inheritance
//! - Macro invocations

use crate::parsers::LanguageId;
use crate::symbols::{Dependency, DependencyType, Location};
use super::{DependencyExtractor, ExtractionContext, BaseDependencyExtractor};
use tree_sitter::Node;

/// Rust-specific dependency extractor
pub struct RustDependencyExtractor;

impl DependencyExtractor for RustDependencyExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Rust
    }
    
    fn extract_dependencies(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
        context: &mut ExtractionContext,
    ) -> Vec<Dependency> {
        let mut dependencies = Vec::new();
        BaseDependencyExtractor::traverse_node(
            self, tree.root_node(), source, context, &mut dependencies
        );
        dependencies
    }
    
    fn extract_from_node(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        match node.kind() {
            "call_expression" => {
                self.extract_function_calls(node, source, context, dependencies);
            }
            "field_expression" => {
                self.extract_method_calls(node, source, context, dependencies);
            }
            "identifier" => {
                self.extract_variable_references(node, source, context, dependencies);
            }
            "use_declaration" => {
                self.extract_imports(node, source, context, dependencies);
            }
            "impl_item" => {
                self.extract_trait_implementations(node, source, context, dependencies);
            }
            "let_declaration" | "const_item" | "static_item" => {
                self.extract_assignments(node, source, context, dependencies);
            }
            "macro_invocation" => {
                self.extract_macro_invocations(node, source, context, dependencies);
            }
            // Control flow nodes
            "if_expression" | "if_let_expression" => {
                self.extract_conditional_dependencies(node, source, context, dependencies);
            }
            "while_expression" | "for_expression" | "loop_expression" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "match_expression" => {
                self.extract_match_dependencies(node, source, context, dependencies);
            }
            "return_expression" => {
                self.extract_return_dependencies(node, source, context, dependencies);
            }
            "break_expression" | "continue_expression" => {
                self.extract_break_continue_dependencies(node, source, context, dependencies);
            }
            "mod_item" => {
                // mod submodule; or mod submodule { ... }
                if let Some(name_node) = node.child_by_field_name("name") {
                    let module_name = self.get_node_text(&name_node, source);
                    let current_scope = context.current_scope();
                    
                    if !module_name.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope,
                            module_name,
                            DependencyType::ModuleDependency,
                            &node,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }
            }
            "extern_crate_declaration" => {
                // extern crate serde;
                if let Some(name_node) = node.child_by_field_name("name") {
                    let crate_name = self.get_node_text(&name_node, source);
                    let current_scope = context.current_scope();
                    
                    if !crate_name.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope,
                            format!("crate::{crate_name}"),
                            DependencyType::ModuleDependency,
                            &node,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }
            }
            _ => {}
        }
    }
    
    fn is_function_call(&self, node: &Node) -> bool {
        matches!(node.kind(), "call_expression")
    }
    
    fn is_variable_reference(&self, node: &Node) -> bool {
        matches!(node.kind(), "identifier" | "field_expression")
    }
    
    fn is_import_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "use_declaration")
    }
    
    fn is_inheritance(&self, node: &Node) -> bool {
        matches!(node.kind(), "impl_item")
    }
    
    fn is_assignment(&self, node: &Node) -> bool {
        matches!(node.kind(), "let_declaration" | "const_item" | "static_item")
    }
    
    fn extract_function_calls(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        // Extract function name from call_expression
        if let Some(function_node) = node.child_by_field_name("function") {
            let function_name = self.get_node_text(&function_node, source);
            let current_scope = context.current_scope();
            
            // Skip empty or invalid function names
            if function_name.trim().is_empty() || function_name.contains('\n') {
                return;
            }
            
            // Handle different call patterns
            let (caller, callee) = if function_name.contains("::") {
                // Static method call: Struct::method()
                (current_scope, function_name)
            } else if function_name.contains('.') {
                // Method call: obj.method() - extract object and method
                let parts: Vec<&str> = function_name.split('.').collect();
                if parts.len() >= 2 {
                    let object = parts[0].to_string();
                    let method = parts[1..].join(".");
                    (current_scope, format!("{object}.{method}"))
                } else {
                    (current_scope, function_name)
                }
            } else {
                // Simple function call
                (current_scope, function_name)
            };
            
            // Try to resolve the function in known symbols
            let resolved_functions = context.find_symbols(&callee);
            let target_function = if !resolved_functions.is_empty() {
                resolved_functions[0].qualified_name()
            } else {
                callee
            };
            
            let dependency = self.create_dependency(
                caller,
                target_function,
                DependencyType::Calls,
                &node,
                context,
            );
            
            dependencies.push(dependency);
            
            // Also extract arguments for variable references
            if let Some(args_node) = node.child_by_field_name("arguments") {
                self.extract_argument_references(args_node, source, context, dependencies);
            }
        }
    }
    
    fn extract_variable_references(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        // Only extract identifiers that aren't part of declarations or function definitions
        if self.is_reference_context(&node) {
            let var_name = self.get_node_text(&node, source);
            let current_scope = context.current_scope();
            
            // Skip keywords, built-in types, and empty names
            if self.is_rust_keyword(&var_name) || var_name.trim().is_empty() {
                return;
            }
            
            // Try to resolve variable in known symbols
            let resolved_vars = context.find_symbols(&var_name);
            let target_var = if !resolved_vars.is_empty() {
                resolved_vars[0].qualified_name()
            } else {
                var_name
            };
            
            let dependency = self.create_dependency(
                current_scope,
                target_var,
                DependencyType::References,
                &node,
                context,
            );
            
            dependencies.push(dependency);
        }
    }
    
    fn extract_imports(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        // Extract use statements: use std::collections::HashMap;
        if let Some(use_tree) = node.child_by_field_name("argument") {
            self.extract_use_tree(use_tree, source, context, dependencies);
        }
    }
    
    fn extract_inheritance(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        self.extract_trait_implementations(node, source, context, dependencies);
    }
    
    fn extract_assignments(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        match node.kind() {
            "let_declaration" => {
                self.extract_let_assignment(node, source, context, dependencies);
            }
            "const_item" | "static_item" => {
                self.extract_const_assignment(node, source, context, dependencies);
            }
            _ => {}
        }
    }

    fn extract_control_flow(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        match node.kind() {
            "if_expression" | "if_let_expression" => {
                self.extract_conditional_dependencies(node, source, context, dependencies);
            }
            "while_expression" | "for_expression" | "loop_expression" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "match_expression" => {
                self.extract_match_dependencies(node, source, context, dependencies);
            }
            "return_expression" => {
                self.extract_return_dependencies(node, source, context, dependencies);
            }
            "break_expression" | "continue_expression" => {
                self.extract_break_continue_dependencies(node, source, context, dependencies);
            }
            _ => {}
        }
    }

    fn is_conditional_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "if_expression" | "if_let_expression")
    }

    fn is_loop_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "while_expression" | "for_expression" | "loop_expression")
    }

    fn is_exception_handling(&self, node: &Node) -> bool {
        // Rust doesn't have traditional try/catch, but has Result handling
        matches!(node.kind(), "try_expression")
    }

    fn is_switch_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "match_expression")
    }

    fn is_return_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "return_expression")
    }

    fn is_break_continue(&self, node: &Node) -> bool {
        matches!(node.kind(), "break_expression" | "continue_expression")
    }
}

impl RustDependencyExtractor {
    /// Extract method calls from field expressions (obj.method())
    fn extract_method_calls(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        // Check if this field expression is part of a call
        if let Some(parent) = node.parent() {
            if parent.kind() == "call_expression" {
                // This is handled by extract_function_calls
                return;
            }
        }
        
        // This is a field access, not a method call
        if let Some(object_node) = node.child_by_field_name("object") {
            let object_name = self.get_node_text(&object_node, source);
            let current_scope = context.current_scope();
            
            if !object_name.trim().is_empty() && !self.is_rust_keyword(&object_name) {
                let resolved_objects = context.find_symbols(&object_name);
                let target_object = if !resolved_objects.is_empty() {
                    resolved_objects[0].qualified_name()
                } else {
                    object_name
                };
                
                let dependency = self.create_dependency(
                    current_scope,
                    target_object,
                    DependencyType::References,
                    &node,
                    context,
                );
                
                dependencies.push(dependency);
            }
        }
    }
    
    /// Extract trait implementations from impl blocks
    fn extract_trait_implementations(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        // impl TraitName for StructName
        if let Some(trait_node) = node.child_by_field_name("trait") {
            if let Some(type_node) = node.child_by_field_name("type") {
                let trait_name = self.get_node_text(&trait_node, source);
                let type_name = self.get_node_text(&type_node, source);
                
                if !trait_name.trim().is_empty() && !type_name.trim().is_empty() {
                    let dependency = self.create_dependency(
                        type_name,
                        trait_name,
                        DependencyType::Implements,
                        &node,
                        context,
                    );
                    
                    dependencies.push(dependency);
                }
            }
        }
    }
    
    /// Extract macro invocations
    fn extract_macro_invocations(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(macro_node) = node.child_by_field_name("macro") {
            let macro_name = self.get_node_text(&macro_node, source);
            let current_scope = context.current_scope();
            
            if !macro_name.trim().is_empty() {
                let dependency = self.create_dependency(
                    current_scope,
                    macro_name,
                    DependencyType::MacroInvocation,
                    &node,
                    context,
                );
                
                dependencies.push(dependency);
            }
        }
    }
    
    /// Extract use tree recursively
    fn extract_use_tree(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        match node.kind() {
            "scoped_identifier" => {
                let import_path = self.get_node_text(&node, source);
                let current_scope = context.current_scope();
                
                let dependency = self.create_dependency(
                    current_scope,
                    import_path,
                    DependencyType::Imports,
                    &node,
                    context,
                );
                
                dependencies.push(dependency);
            }
            "use_list" => {
                // Handle use std::{HashMap, HashSet};
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() != "," && child.kind() != "{" && child.kind() != "}" {
                        self.extract_use_tree(child, source, context, dependencies);
                    }
                }
            }
            "use_as_clause" => {
                // Handle use std::collections::HashMap as Map;
                if let Some(path_node) = node.child_by_field_name("path") {
                    self.extract_use_tree(path_node, source, context, dependencies);
                }
            }
            "identifier" => {
                // Simple identifier in use statement
                let import_name = self.get_node_text(&node, source);
                let current_scope = context.current_scope();
                
                if !self.is_rust_keyword(&import_name) {
                    let dependency = self.create_dependency(
                        current_scope,
                        import_name,
                        DependencyType::Imports,
                        &node,
                        context,
                    );
                    
                    dependencies.push(dependency);
                }
            }
            _ => {
                // Recursively handle other node types
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.extract_use_tree(child, source, context, dependencies);
                }
            }
        }
    }
    
    /// Extract let assignment dependencies
    fn extract_let_assignment(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(pattern_node) = node.child_by_field_name("pattern") {
            if let Some(value_node) = node.child_by_field_name("value") {
                let var_name = self.get_node_text(&pattern_node, source);
                let current_scope = context.current_scope();

                // Create variable dependency with scope information
                dependencies.push(Dependency {
                    from_symbol: current_scope.clone(),
                    to_symbol: var_name.clone(),
                    relationship_type: DependencyType::Declares,
                    location: Location::from_node(&pattern_node, &context.file_path),
                    file_path: context.file_path.clone(),
                    language: LanguageId::Rust,
                    context: Some(format!("let {var_name} = ...")),
                    strength: 1.0,
                    is_conditional: false,
                });

                // Extract dependencies from the value expression
                self.extract_expression_dependencies(
                    value_node, source, context, dependencies, &var_name
                );
            }
        }
    }
    
    /// Extract const/static assignment dependencies
    fn extract_const_assignment(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            if let Some(value_node) = node.child_by_field_name("value") {
                let const_name = self.get_node_text(&name_node, source);
                
                // Extract dependencies from the value expression
                self.extract_expression_dependencies(
                    value_node, source, context, dependencies, &const_name
                );
            }
        }
    }
    
    /// Extract dependencies from expressions in assignments
    fn extract_expression_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
        assigner: &str,
    ) {
        match node.kind() {
            "identifier" => {
                let referenced_var = self.get_node_text(&node, source);
                if !self.is_rust_keyword(&referenced_var) && !referenced_var.trim().is_empty() {
                    let dependency = self.create_dependency(
                        assigner.to_string(),
                        referenced_var,
                        DependencyType::DataFlow,
                        &node,
                        context,
                    );
                    dependencies.push(dependency);
                }
            }
            "call_expression" => {
                // Function call in assignment creates both call and data flow dependencies
                self.extract_function_calls(node, source, context, dependencies);
            }
            _ => {
                // Recursively extract from child expressions
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.extract_expression_dependencies(child, source, context, dependencies, assigner);
                }
            }
        }
    }
    
    /// Extract argument references from function calls
    fn extract_argument_references(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                let arg_name = self.get_node_text(&child, source);
                let current_scope = context.current_scope();
                
                if !self.is_rust_keyword(&arg_name) && !arg_name.trim().is_empty() {
                    let resolved_args = context.find_symbols(&arg_name);
                    let target_arg = if !resolved_args.is_empty() {
                        resolved_args[0].qualified_name()
                    } else {
                        arg_name
                    };
                    
                    let dependency = self.create_dependency(
                        current_scope,
                        target_arg,
                        DependencyType::References,
                        &child,
                        context,
                    );
                    
                    dependencies.push(dependency);
                }
            } else {
                // Recursively extract from complex argument expressions
                self.extract_argument_references(child, source, context, dependencies);
            }
        }
    }
    
    /// Check if a node is in a reference context (not declaration)
    fn is_reference_context(&self, node: &Node) -> bool {
        let mut current = *node;
        
        // Walk up the tree to check context
        while let Some(parent) = current.parent() {
            match parent.kind() {
                // Skip identifiers in these declaration contexts
                "function_item" | "struct_item" | "enum_item" | "trait_item" |
                "impl_item" | "let_declaration" | "const_item" | "static_item" |
                "parameter" | "field_declaration" | "variant" => {
                    // Check if this identifier is the name being declared
                    if let Some(name_field) = parent.child_by_field_name("name") {
                        if name_field.id() == node.id() {
                            return false; // This is a declaration
                        }
                    }
                }
                "use_declaration" => {
                    return false; // Import statements aren't references
                }
                _ => {}
            }
            current = parent;
        }
        
        true
    }
    
    /// Check if a string is a Rust keyword
    fn is_rust_keyword(&self, name: &str) -> bool {
        matches!(name,
            "as" | "break" | "const" | "continue" | "crate" | "else" | "enum" |
            "extern" | "false" | "fn" | "for" | "if" | "impl" | "in" | "let" |
            "loop" | "match" | "mod" | "move" | "mut" | "pub" | "ref" | "return" |
            "self" | "Self" | "static" | "struct" | "super" | "trait" | "true" |
            "type" | "unsafe" | "use" | "where" | "while" | "async" | "await" |
            "dyn" | "abstract" | "become" | "box" | "do" | "final" | "macro" |
            "override" | "priv" | "typeof" | "unsized" | "virtual" | "yield" |
            // Built-in types
            "bool" | "char" | "str" | "u8" | "u16" | "u32" | "u64" | "u128" |
            "i8" | "i16" | "i32" | "i64" | "i128" | "f32" | "f64" | "usize" | "isize"
        )
    }

    /// Extract conditional dependencies (if/else)
    fn extract_conditional_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        // Extract condition dependencies
        if let Some(condition_node) = node.child_by_field_name("condition") {
            self.extract_condition_variables(condition_node, source, context, dependencies, &current_scope);
        }

        // Extract dependencies from then/else blocks
        if let Some(consequence_node) = node.child_by_field_name("consequence") {
            // Mark as conditional execution
            let dependency = self.create_dependency(
                current_scope.clone(),
                "conditional_block".to_string(),
                DependencyType::ConditionalExecution,
                &consequence_node,
                context,
            );
            dependencies.push(dependency);
        }

        if let Some(alternative_node) = node.child_by_field_name("alternative") {
            let dependency = self.create_dependency(
                current_scope,
                "else_block".to_string(),
                DependencyType::ConditionalExecution,
                &alternative_node,
                context,
            );
            dependencies.push(dependency);
        }
    }

    /// Extract loop dependencies
    fn extract_loop_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        match node.kind() {
            "for_expression" => {
                // for pattern in iterator
                if let Some(pattern_node) = node.child_by_field_name("pattern") {
                    let pattern_name = self.get_node_text(&pattern_node, source);
                    if !pattern_name.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            pattern_name,
                            DependencyType::LoopIteration,
                            &pattern_node,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }

                if let Some(value_node) = node.child_by_field_name("value") {
                    self.extract_condition_variables(value_node, source, context, dependencies, &current_scope);
                }
            }
            "while_expression" => {
                // while condition
                if let Some(condition_node) = node.child_by_field_name("condition") {
                    self.extract_condition_variables(condition_node, source, context, dependencies, &current_scope);
                }
            }
            "loop_expression" => {
                // Infinite loop - just mark as loop iteration
                let dependency = self.create_dependency(
                    current_scope,
                    "infinite_loop".to_string(),
                    DependencyType::LoopIteration,
                    &node,
                    context,
                );
                dependencies.push(dependency);
            }
            _ => {}
        }
    }

    /// Extract match expression dependencies
    fn extract_match_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        // Extract value being matched
        if let Some(value_node) = node.child_by_field_name("value") {
            self.extract_condition_variables(value_node, source, context, dependencies, &current_scope);
        }

        // Extract match arms
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "match_arm" {
                let dependency = self.create_dependency(
                    current_scope.clone(),
                    "match_arm".to_string(),
                    DependencyType::SwitchCase,
                    &child,
                    context,
                );
                dependencies.push(dependency);

                // Extract pattern variables
                if let Some(pattern_node) = child.child_by_field_name("pattern") {
                    self.extract_pattern_variables(pattern_node, source, context, dependencies, &current_scope);
                }
            }
        }
    }

    /// Extract return dependencies
    fn extract_return_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        // Extract returned value dependencies
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "return" {
                self.extract_expression_dependencies(child, source, context, dependencies, &current_scope);
            }
        }

        let dependency = self.create_dependency(
            current_scope,
            "return_value".to_string(),
            DependencyType::ReturnFlow,
            &node,
            context,
        );
        dependencies.push(dependency);
    }

    /// Extract break/continue dependencies
    fn extract_break_continue_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();
        let flow_type = if node.kind() == "break_expression" { "break" } else { "continue" };

        // Extract label if present
        if let Some(label_node) = node.child_by_field_name("label") {
            let label_name = self.get_node_text(&label_node, source);
            let dependency = self.create_dependency(
                current_scope.clone(),
                label_name,
                DependencyType::BreakContinue,
                &label_node,
                context,
            );
            dependencies.push(dependency);
        }

        let dependency = self.create_dependency(
            current_scope,
            flow_type.to_string(),
            DependencyType::BreakContinue,
            &node,
            context,
        );
        dependencies.push(dependency);
    }

    /// Extract variables from condition expressions
    fn extract_condition_variables(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
        scope: &str,
    ) {
        match node.kind() {
            "identifier" => {
                let var_name = self.get_node_text(&node, source);
                if !self.is_rust_keyword(&var_name) && !var_name.trim().is_empty() {
                    let dependency = self.create_dependency(
                        scope.to_string(),
                        var_name,
                        DependencyType::ControlFlow,
                        &node,
                        context,
                    );
                    dependencies.push(dependency);
                }
            }
            _ => {
                // Recursively extract from child expressions
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.extract_condition_variables(child, source, context, dependencies, scope);
                }
            }
        }
    }

    /// Extract variables from pattern matching
    fn extract_pattern_variables(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
        scope: &str,
    ) {
        match node.kind() {
            "identifier" => {
                let var_name = self.get_node_text(&node, source);
                if !self.is_rust_keyword(&var_name) && !var_name.trim().is_empty() {
                    let dependency = self.create_dependency(
                        scope.to_string(),
                        var_name,
                        DependencyType::SwitchCase,
                        &node,
                        context,
                    );
                    dependencies.push(dependency);
                }
            }
            _ => {
                // Recursively extract from child patterns
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.extract_pattern_variables(child, source, context, dependencies, scope);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::ParserFactory;
    use crate::symbols::SymbolExtractorFactory;

    #[test]
    #[ignore] // Skip due to tree-sitter initialization issues in some environments
    fn test_rust_function_call_extraction() {
        let source = r#"
fn main() {
    let result = calculate(10, 20);
    println!("Result: {}", result);
}

fn calculate(a: i32, b: i32) -> i32 {
    a + b
}
"#;

        let mut parser_factory = ParserFactory::new();
        let parse_result = parser_factory.parse(source, LanguageId::Rust).unwrap();
        
        let symbol_factory = SymbolExtractorFactory::new();
        let symbols = symbol_factory.extract_symbols(
            &parse_result.tree,
            source,
            "test.rs",
            LanguageId::Rust,
        );
        
        let extractor = RustDependencyExtractor;
        let deps = extractor.extract_dependencies(
            &parse_result.tree,
            source,
            &mut ExtractionContext::new("test.rs".to_string(), LanguageId::Rust, symbols),
        );
        
        // Should find call to calculate and println! macro
        assert!(deps.len() >= 2);
        
        let calculate_call = deps.iter().find(|d| 
            d.to_symbol.contains("calculate") && d.relationship_type == DependencyType::Calls
        );
        assert!(calculate_call.is_some());
        
        let println_call = deps.iter().find(|d| 
            d.to_symbol.contains("println") && d.relationship_type == DependencyType::MacroInvocation
        );
        assert!(println_call.is_some());
    }

    #[test]
    #[ignore] // Skip due to tree-sitter initialization issues in some environments
    fn test_rust_use_statement_extraction() {
        let source = r#"
use std::collections::HashMap;
use std::io::{Read, Write};

fn main() {
    let mut map = HashMap::new();
}
"#;

        let mut parser_factory = ParserFactory::new();
        let parse_result = parser_factory.parse(source, LanguageId::Rust).unwrap();
        
        let symbol_factory = SymbolExtractorFactory::new();
        let symbols = symbol_factory.extract_symbols(
            &parse_result.tree,
            source,
            "test.rs",
            LanguageId::Rust,
        );
        
        let extractor = RustDependencyExtractor;
        let deps = extractor.extract_dependencies(
            &parse_result.tree,
            source,
            &mut ExtractionContext::new("test.rs".to_string(), LanguageId::Rust, symbols),
        );
        
        // Should find imports for HashMap, Read, Write
        let import_deps: Vec<_> = deps.iter().filter(|d| 
            d.relationship_type == DependencyType::Imports
        ).collect();
        
        assert!(import_deps.len() >= 3);
    }

    #[test]
    #[ignore] // Skip due to tree-sitter initialization issues in some environments
    fn test_rust_module_dependency_extraction() {
        let source = r#"
extern crate serde;
extern crate tokio;

mod utils;
mod config {
    pub fn load() {}
}

fn main() {
    utils::helper();
    config::load();
}
"#;

        let mut parser_factory = ParserFactory::new();
        let parse_result = parser_factory.parse(source, LanguageId::Rust).unwrap();
        
        let symbol_factory = SymbolExtractorFactory::new();
        let symbols = symbol_factory.extract_symbols(
            &parse_result.tree,
            source,
            "test.rs",
            LanguageId::Rust,
        );
        
        let extractor = RustDependencyExtractor;
        let deps = extractor.extract_dependencies(
            &parse_result.tree,
            source,
            &mut ExtractionContext::new("test.rs".to_string(), LanguageId::Rust, symbols),
        );
        
        // Should find module dependencies for extern crates and mod declarations
        let module_deps: Vec<_> = deps.iter().filter(|d| 
            d.relationship_type == DependencyType::ModuleDependency
        ).collect();
        
        assert!(module_deps.len() >= 4); // serde, tokio, utils, config
        
        // Check for extern crate dependencies
        let serde_dep = deps.iter().find(|d| 
            d.to_symbol.contains("serde") && d.relationship_type == DependencyType::ModuleDependency
        );
        assert!(serde_dep.is_some());
        
        let tokio_dep = deps.iter().find(|d| 
            d.to_symbol.contains("tokio") && d.relationship_type == DependencyType::ModuleDependency
        );
        assert!(tokio_dep.is_some());
        
        // Check for mod dependencies
        let utils_dep = deps.iter().find(|d| 
            d.to_symbol == "utils" && d.relationship_type == DependencyType::ModuleDependency
        );
        assert!(utils_dep.is_some());
        
        let config_dep = deps.iter().find(|d| 
            d.to_symbol == "config" && d.relationship_type == DependencyType::ModuleDependency
        );
        assert!(config_dep.is_some());
    }
}
//! C#-specific dependency extraction
//! 
//! Extracts dependency relationships from C# source code, including:
//! - Method calls and constructor invocations
//! - Property access and field references
//! - Class inheritance and interface implementation
//! - Using statements and namespace dependencies
//! - Attribute usage
//! - Exception handling (try/catch/finally)
//! - Control flow (if/else, loops, switch)
//! - LINQ expressions and lambda functions

use crate::parsers::LanguageId;
use crate::symbols::{Dependency, DependencyType};
use super::{DependencyExtractor, ExtractionContext, BaseDependencyExtractor};
use tree_sitter::Node;

/// C#-specific dependency extractor
pub struct CSharpDependencyExtractor;

impl DependencyExtractor for CSharpDependencyExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::CSharp
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
            "invocation_expression" => {
                self.extract_function_calls(node, source, context, dependencies);
            }
            "object_creation_expression" => {
                self.extract_constructor_calls(node, source, context, dependencies);
            }
            "member_access_expression" => {
                self.extract_member_access(node, source, context, dependencies);
            }
            "identifier_name" => {
                self.extract_variable_references(node, source, context, dependencies);
            }
            "using_directive" => {
                self.extract_imports(node, source, context, dependencies);
            }
            "class_declaration" | "interface_declaration" | "struct_declaration" => {
                self.extract_class_inheritance(node, source, context, dependencies);
            }
            "assignment_expression" | "variable_declarator" => {
                self.extract_assignments(node, source, context, dependencies);
            }
            "attribute" => {
                self.extract_attribute_dependencies(node, source, context, dependencies);
            }
            // Control flow
            "if_statement" => {
                self.extract_conditional_dependencies(node, source, context, dependencies);
            }
            "for_statement" | "foreach_statement" | "while_statement" | "do_statement" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "try_statement" | "catch_clause" | "finally_clause" => {
                self.extract_exception_dependencies(node, source, context, dependencies);
            }
            "switch_statement" | "switch_expression" => {
                self.extract_switch_dependencies(node, source, context, dependencies);
            }
            "return_statement" => {
                self.extract_return_dependencies(node, source, context, dependencies);
            }
            "break_statement" | "continue_statement" => {
                self.extract_break_continue_dependencies(node, source, context, dependencies);
            }
            "lambda_expression" | "anonymous_method_expression" => {
                self.extract_lambda_dependencies(node, source, context, dependencies);
            }
            _ => {}
        }
    }
    
    fn is_function_call(&self, node: &Node) -> bool {
        matches!(node.kind(), "invocation_expression" | "object_creation_expression")
    }
    
    fn is_variable_reference(&self, node: &Node) -> bool {
        matches!(node.kind(), "identifier_name" | "member_access_expression")
    }
    
    fn is_import_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "using_directive")
    }
    
    fn is_inheritance(&self, node: &Node) -> bool {
        (node.kind() == "class_declaration" && node.child_by_field_name("base_list").is_some()) ||
        (node.kind() == "interface_declaration" && node.child_by_field_name("base_list").is_some()) ||
        (node.kind() == "struct_declaration" && node.child_by_field_name("base_list").is_some())
    }
    
    fn is_assignment(&self, node: &Node) -> bool {
        matches!(node.kind(), "assignment_expression" | "variable_declarator")
    }
    
    fn extract_function_calls(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(expression_node) = node.child_by_field_name("function") {
            let method_name = self.get_node_text(&expression_node, source);
            let current_scope = context.current_scope();
            
            if !method_name.trim().is_empty() {
                // Try to resolve the method in known symbols
                let resolved_methods = context.find_symbols_global(&method_name);
                let target_method = if !resolved_methods.is_empty() {
                    resolved_methods[0].qualified_name()
                } else {
                    method_name
                };
                
                let dependency = self.create_dependency(
                    current_scope,
                    target_method,
                    DependencyType::Calls,
                    &node,
                    context,
                );
                
                dependencies.push(dependency);
                
                // Extract arguments for variable references
                if let Some(args_node) = node.child_by_field_name("arguments") {
                    self.extract_argument_references(args_node, source, context, dependencies);
                }
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
        // Only extract identifiers that aren't part of declarations
        if self.is_reference_context(&node) {
            let var_name = self.get_node_text(&node, source);
            let current_scope = context.current_scope();
            
            // Skip keywords and built-ins
            if self.is_csharp_keyword(&var_name) || var_name.trim().is_empty() {
                return;
            }
            
            // Try to resolve variable in known symbols
            let resolved_vars = context.find_symbols_global(&var_name);
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
        let current_scope = context.current_scope();
        
        // using System.Collections.Generic; or using static System.Math;
        if let Some(name_node) = node.child_by_field_name("name") {
            let namespace_name = self.get_node_text(&name_node, source);
            if !namespace_name.trim().is_empty() {
                let dependency_type = if self.get_node_text(&node, source).contains("static") {
                    DependencyType::Uses
                } else {
                    DependencyType::NamespaceUsage
                };
                
                let dependency = self.create_dependency(
                    current_scope,
                    namespace_name,
                    dependency_type,
                    &name_node,
                    context,
                );
                dependencies.push(dependency);
            }
        }
    }
    
    fn extract_inheritance(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        self.extract_class_inheritance(node, source, context, dependencies);
    }
    
    fn extract_assignments(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        match node.kind() {
            "assignment_expression" => {
                self.extract_assignment_dependencies(node, source, context, dependencies);
            }
            "variable_declarator" => {
                self.extract_variable_declaration_dependencies(node, source, context, dependencies);
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
            "if_statement" => {
                self.extract_conditional_dependencies(node, source, context, dependencies);
            }
            "for_statement" | "foreach_statement" | "while_statement" | "do_statement" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "try_statement" | "catch_clause" | "finally_clause" => {
                self.extract_exception_dependencies(node, source, context, dependencies);
            }
            "switch_statement" | "switch_expression" => {
                self.extract_switch_dependencies(node, source, context, dependencies);
            }
            "return_statement" => {
                self.extract_return_dependencies(node, source, context, dependencies);
            }
            "break_statement" | "continue_statement" => {
                self.extract_break_continue_dependencies(node, source, context, dependencies);
            }
            _ => {}
        }
    }
    
    fn is_conditional_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "if_statement")
    }
    
    fn is_loop_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "for_statement" | "foreach_statement" | "while_statement" | "do_statement")
    }
    
    fn is_exception_handling(&self, node: &Node) -> bool {
        matches!(node.kind(), "try_statement" | "catch_clause" | "finally_clause")
    }
    
    fn is_switch_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "switch_statement" | "switch_expression")
    }
    
    fn is_return_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "return_statement")
    }
    
    fn is_break_continue(&self, node: &Node) -> bool {
        matches!(node.kind(), "break_statement" | "continue_statement")
    }
}

impl CSharpDependencyExtractor {
    /// Extract constructor calls (new Class())
    fn extract_constructor_calls(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(type_node) = node.child_by_field_name("type") {
            let class_name = self.get_node_text(&type_node, source);
            let current_scope = context.current_scope();

            if !class_name.trim().is_empty() {
                let resolved_classes = context.find_symbols_global(&class_name);
                let target_class = if !resolved_classes.is_empty() {
                    resolved_classes[0].qualified_name()
                } else {
                    class_name
                };

                let dependency = self.create_dependency(
                    current_scope,
                    target_class,
                    DependencyType::Calls,
                    &node,
                    context,
                );

                dependencies.push(dependency);

                // Extract constructor arguments
                if let Some(args_node) = node.child_by_field_name("arguments") {
                    self.extract_argument_references(args_node, source, context, dependencies);
                }
            }
        }
    }

    /// Extract member access (obj.member)
    fn extract_member_access(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(expression_node) = node.child_by_field_name("expression") {
            let object_name = self.get_node_text(&expression_node, source);
            let current_scope = context.current_scope();

            if !object_name.trim().is_empty() && !self.is_csharp_keyword(&object_name) {
                let resolved_objects = context.find_symbols_global(&object_name);
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

    /// Extract class inheritance and interface implementation
    fn extract_class_inheritance(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(identifier_node) = node.child_by_field_name("name") {
            let class_name = self.get_node_text(&identifier_node, source);

            // Extract base types (class inheritance and interface implementation)
            if let Some(base_list_node) = node.child_by_field_name("base_list") {
                let mut cursor = base_list_node.walk();
                for child in base_list_node.children(&mut cursor) {
                    if child.kind() == "simple_base_type" {
                        if let Some(type_node) = child.child_by_field_name("type") {
                            let base_type = self.get_node_text(&type_node, source);
                            if !base_type.trim().is_empty() {
                                // Determine if it's inheritance or interface implementation
                                // In C#, the first base type is typically the base class, others are interfaces
                                let dependency_type = if self.is_likely_interface(&base_type) {
                                    DependencyType::Implements
                                } else {
                                    DependencyType::Inherits
                                };

                                let dependency = self.create_dependency(
                                    class_name.clone(),
                                    base_type,
                                    dependency_type,
                                    &type_node,
                                    context,
                                );
                                dependencies.push(dependency);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Extract attribute dependencies
    fn extract_attribute_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let attribute_name = self.get_node_text(&name_node, source);
            let current_scope = context.current_scope();

            if !attribute_name.trim().is_empty() {
                let dependency = self.create_dependency(
                    current_scope,
                    attribute_name,
                    DependencyType::Uses,
                    &name_node,
                    context,
                );
                dependencies.push(dependency);
            }
        }
    }

    /// Extract lambda expression dependencies
    fn extract_lambda_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        // Extract dependencies from lambda body
        if let Some(body_node) = node.child_by_field_name("body") {
            self.extract_expression_dependencies(body_node, source, context, dependencies, &current_scope);
        }

        // Mark as lambda dependency
        let dependency = self.create_dependency(
            current_scope,
            "lambda_expression".to_string(),
            DependencyType::ControlFlow,
            &node,
            context,
        );
        dependencies.push(dependency);
    }

    /// Check if a type name is likely an interface (starts with 'I' followed by uppercase)
    fn is_likely_interface(&self, type_name: &str) -> bool {
        type_name.len() > 1 &&
        type_name.starts_with('I') &&
        type_name.chars().nth(1).is_some_and(|c| c.is_uppercase())
    }

    /// Check if a string is a C# keyword
    fn is_csharp_keyword(&self, name: &str) -> bool {
        matches!(name,
            "abstract" | "as" | "base" | "bool" | "break" | "byte" | "case" | "catch" |
            "char" | "checked" | "class" | "const" | "continue" | "decimal" | "default" |
            "delegate" | "do" | "double" | "else" | "enum" | "event" | "explicit" |
            "extern" | "false" | "finally" | "fixed" | "float" | "for" | "foreach" |
            "goto" | "if" | "implicit" | "in" | "int" | "interface" | "internal" |
            "is" | "lock" | "long" | "namespace" | "new" | "null" | "object" |
            "operator" | "out" | "override" | "params" | "private" | "protected" |
            "public" | "readonly" | "ref" | "return" | "sbyte" | "sealed" | "short" |
            "sizeof" | "stackalloc" | "static" | "string" | "struct" | "switch" |
            "this" | "throw" | "true" | "try" | "typeof" | "uint" | "ulong" |
            "unchecked" | "unsafe" | "ushort" | "using" | "virtual" | "void" |
            "volatile" | "while" |
            // Built-in types and common classes
            "String" | "Object" | "Int32" | "Boolean" | "Double" | "Single" |
            "Int64" | "Int16" | "Byte" | "Char" | "System" | "Math" | "Console" |
            "var" | "dynamic" | "async" | "await" | "yield"
        )
    }

    /// Extract assignment dependencies
    fn extract_assignment_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(left_node) = node.child_by_field_name("left") {
            if let Some(right_node) = node.child_by_field_name("right") {
                let var_name = self.get_node_text(&left_node, source);

                // Extract dependencies from the right-hand side
                self.extract_expression_dependencies(
                    right_node, source, context, dependencies, &var_name
                );
            }
        }
    }

    /// Extract variable declaration dependencies
    fn extract_variable_declaration_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(identifier_node) = node.child_by_field_name("identifier") {
            if let Some(initializer_node) = node.child_by_field_name("initializer") {
                let var_name = self.get_node_text(&identifier_node, source);

                // Extract dependencies from the initializer expression
                self.extract_expression_dependencies(
                    initializer_node, source, context, dependencies, &var_name
                );
            }
        }
    }

    /// Extract conditional dependencies
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

        // Mark as conditional execution
        let dependency = self.create_dependency(
            current_scope,
            "if_block".to_string(),
            DependencyType::ConditionalExecution,
            &node,
            context,
        );
        dependencies.push(dependency);
    }

    /// Extract dependencies from expressions
    fn extract_expression_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
        assigner: &str,
    ) {
        match node.kind() {
            "identifier_name" => {
                let referenced_var = self.get_node_text(&node, source);
                if !self.is_csharp_keyword(&referenced_var) && !referenced_var.trim().is_empty() {
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
            "invocation_expression" | "object_creation_expression" => {
                // Method call or constructor in assignment creates both call and data flow dependencies
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
            "identifier_name" => {
                let var_name = self.get_node_text(&node, source);
                if !self.is_csharp_keyword(&var_name) && !var_name.trim().is_empty() {
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

    /// Extract argument references from method calls
    fn extract_argument_references(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier_name" {
                let arg_name = self.get_node_text(&child, source);
                let current_scope = context.current_scope();

                if !self.is_csharp_keyword(&arg_name) && !arg_name.trim().is_empty() {
                    let resolved_args = context.find_symbols_global(&arg_name);
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
                "method_declaration" | "class_declaration" | "interface_declaration" |
                "struct_declaration" | "variable_declarator" | "parameter" | "using_directive" => {
                    // Check if this identifier is the name being declared
                    if let Some(name_field) = parent.child_by_field_name("name") {
                        if name_field.id() == node.id() {
                            return false; // This is a declaration
                        }
                    }
                    if let Some(identifier_field) = parent.child_by_field_name("identifier") {
                        if identifier_field.id() == node.id() {
                            return false; // This is a declaration
                        }
                    }
                }
                "assignment_expression" => {
                    // Check if this is the left side of assignment
                    if let Some(left_field) = parent.child_by_field_name("left") {
                        if left_field.id() == node.id() {
                            return false; // This is an assignment target
                        }
                    }
                }
                _ => {}
            }
            current = parent;
        }

        true
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
            "for_statement" => {
                // for (init; condition; incrementors)
                if let Some(condition_node) = node.child_by_field_name("condition") {
                    self.extract_condition_variables(condition_node, source, context, dependencies, &current_scope);
                }
                if let Some(declaration_node) = node.child_by_field_name("declaration") {
                    self.extract_condition_variables(declaration_node, source, context, dependencies, &current_scope);
                }
                if let Some(incrementors_node) = node.child_by_field_name("incrementors") {
                    self.extract_condition_variables(incrementors_node, source, context, dependencies, &current_scope);
                }
            }
            "foreach_statement" => {
                // foreach (Type var in collection)
                if let Some(identifier_node) = node.child_by_field_name("identifier") {
                    let var_name = self.get_node_text(&identifier_node, source);
                    if !var_name.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            var_name,
                            DependencyType::LoopIteration,
                            &identifier_node,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }

                if let Some(expression_node) = node.child_by_field_name("expression") {
                    self.extract_condition_variables(expression_node, source, context, dependencies, &current_scope);
                }
            }
            "while_statement" | "do_statement" => {
                // while (condition) or do ... while (condition)
                if let Some(condition_node) = node.child_by_field_name("condition") {
                    self.extract_condition_variables(condition_node, source, context, dependencies, &current_scope);
                }
            }
            _ => {}
        }
    }

    /// Extract exception handling dependencies
    fn extract_exception_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        match node.kind() {
            "catch_clause" => {
                // catch (ExceptionType var)
                if let Some(declaration_node) = node.child_by_field_name("declaration") {
                    if let Some(type_node) = declaration_node.child_by_field_name("type") {
                        let exception_type = self.get_node_text(&type_node, source);
                        if !exception_type.trim().is_empty() {
                            let dependency = self.create_dependency(
                                current_scope.clone(),
                                exception_type,
                                DependencyType::ExceptionHandling,
                                &type_node,
                                context,
                            );
                            dependencies.push(dependency);
                        }
                    }

                    if let Some(identifier_node) = declaration_node.child_by_field_name("identifier") {
                        let var_name = self.get_node_text(&identifier_node, source);
                        if !var_name.trim().is_empty() {
                            let dependency = self.create_dependency(
                                current_scope.clone(),
                                var_name,
                                DependencyType::ExceptionHandling,
                                &identifier_node,
                                context,
                            );
                            dependencies.push(dependency);
                        }
                    }
                }
            }
            _ => {
                let dependency = self.create_dependency(
                    current_scope,
                    format!("{}_block", node.kind()),
                    DependencyType::ExceptionHandling,
                    &node,
                    context,
                );
                dependencies.push(dependency);
            }
        }
    }

    /// Extract switch dependencies
    fn extract_switch_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        // Extract switch expression/discriminant
        if let Some(expression_node) = node.child_by_field_name("expression") {
            self.extract_condition_variables(expression_node, source, context, dependencies, &current_scope);
        }

        let dependency = self.create_dependency(
            current_scope,
            "switch_statement".to_string(),
            DependencyType::SwitchCase,
            &node,
            context,
        );
        dependencies.push(dependency);
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
        if let Some(expression_node) = node.child_by_field_name("expression") {
            self.extract_expression_dependencies(expression_node, source, context, dependencies, &current_scope);
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
        _source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();
        let flow_type = if node.kind() == "break_statement" { "break" } else { "continue" };

        let dependency = self.create_dependency(
            current_scope,
            flow_type.to_string(),
            DependencyType::BreakContinue,
            &node,
            context,
        );
        dependencies.push(dependency);
    }
}

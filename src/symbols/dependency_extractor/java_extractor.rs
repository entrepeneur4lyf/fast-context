//! Java-specific dependency extraction
//! 
//! Extracts dependency relationships from Java source code, including:
//! - Method calls and constructor invocations
//! - Field access and variable references
//! - Class inheritance and interface implementation
//! - Import statements and package dependencies
//! - Annotation usage
//! - Exception handling (try/catch/finally)
//! - Control flow (if/else, loops, switch)

use crate::parsers::LanguageId;
use crate::symbols::{Dependency, DependencyType};
use super::{DependencyExtractor, ExtractionContext, BaseDependencyExtractor};
use tree_sitter::Node;

/// Java-specific dependency extractor
pub struct JavaDependencyExtractor;

impl DependencyExtractor for JavaDependencyExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Java
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
            "method_invocation" => {
                self.extract_function_calls(node, source, context, dependencies);
            }
            "object_creation_expression" => {
                self.extract_constructor_calls(node, source, context, dependencies);
            }
            "field_access" => {
                self.extract_field_access(node, source, context, dependencies);
            }
            "identifier" => {
                self.extract_variable_references(node, source, context, dependencies);
            }
            "import_declaration" => {
                self.extract_imports(node, source, context, dependencies);
            }
            "class_declaration" | "interface_declaration" => {
                self.extract_class_inheritance(node, source, context, dependencies);
            }
            "assignment_expression" | "variable_declarator" => {
                self.extract_assignments(node, source, context, dependencies);
            }
            "annotation" => {
                self.extract_annotation_dependencies(node, source, context, dependencies);
            }
            // Control flow
            "if_statement" => {
                self.extract_conditional_dependencies(node, source, context, dependencies);
            }
            "for_statement" | "enhanced_for_statement" | "while_statement" | "do_statement" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "try_statement" | "catch_clause" | "finally_clause" => {
                self.extract_exception_dependencies(node, source, context, dependencies);
            }
            "switch_expression" | "switch_statement" => {
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
    
    fn is_function_call(&self, node: &Node) -> bool {
        matches!(node.kind(), "method_invocation" | "object_creation_expression")
    }
    
    fn is_variable_reference(&self, node: &Node) -> bool {
        matches!(node.kind(), "identifier" | "field_access")
    }
    
    fn is_import_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "import_declaration")
    }
    
    fn is_inheritance(&self, node: &Node) -> bool {
        (node.kind() == "class_declaration" && node.child_by_field_name("superclass").is_some()) ||
        (node.kind() == "class_declaration" && node.child_by_field_name("interfaces").is_some()) ||
        (node.kind() == "interface_declaration" && node.child_by_field_name("extends").is_some())
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
        if let Some(name_node) = node.child_by_field_name("name") {
            let method_name = self.get_node_text(&name_node, source);
            let current_scope = context.current_scope();
            
            if !method_name.trim().is_empty() {
                // Handle object.method() calls
                let full_call = if let Some(object_node) = node.child_by_field_name("object") {
                    let object_name = self.get_node_text(&object_node, source);
                    format!("{object_name}.{method_name}")
                } else {
                    method_name
                };
                
                // Try to resolve the method in known symbols
                let resolved_methods = context.find_symbols_global(&full_call);
                let target_method = if !resolved_methods.is_empty() {
                    resolved_methods[0].qualified_name()
                } else {
                    full_call
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
            if self.is_java_keyword(&var_name) || var_name.trim().is_empty() {
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
        
        // import package.Class; or import package.*;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "scoped_identifier" || child.kind() == "identifier" {
                let import_path = self.get_node_text(&child, source);
                if !import_path.trim().is_empty() {
                    let dependency_type = if import_path.ends_with("*") {
                        DependencyType::ModuleDependency
                    } else {
                        DependencyType::Imports
                    };
                    
                    let dependency = self.create_dependency(
                        current_scope.clone(),
                        import_path,
                        dependency_type,
                        &child,
                        context,
                    );
                    dependencies.push(dependency);
                }
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
            "for_statement" | "enhanced_for_statement" | "while_statement" | "do_statement" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "try_statement" | "catch_clause" | "finally_clause" => {
                self.extract_exception_dependencies(node, source, context, dependencies);
            }
            "switch_expression" | "switch_statement" => {
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
        matches!(node.kind(), "for_statement" | "enhanced_for_statement" | "while_statement" | "do_statement")
    }
    
    fn is_exception_handling(&self, node: &Node) -> bool {
        matches!(node.kind(), "try_statement" | "catch_clause" | "finally_clause")
    }
    
    fn is_switch_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "switch_expression" | "switch_statement")
    }
    
    fn is_return_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "return_statement")
    }
    
    fn is_break_continue(&self, node: &Node) -> bool {
        matches!(node.kind(), "break_statement" | "continue_statement")
    }
}

impl JavaDependencyExtractor {
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

    /// Extract field access (obj.field)
    fn extract_field_access(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(object_node) = node.child_by_field_name("object") {
            let object_name = self.get_node_text(&object_node, source);
            let current_scope = context.current_scope();

            if !object_name.trim().is_empty() && !self.is_java_keyword(&object_name) {
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
        if let Some(class_name_node) = node.child_by_field_name("name") {
            let class_name = self.get_node_text(&class_name_node, source);

            // Extract superclass (extends)
            if let Some(superclass_node) = node.child_by_field_name("superclass") {
                let parent_class = self.get_node_text(&superclass_node, source);
                if !parent_class.trim().is_empty() {
                    let dependency = self.create_dependency(
                        class_name.clone(),
                        parent_class,
                        DependencyType::Inherits,
                        &superclass_node,
                        context,
                    );
                    dependencies.push(dependency);
                }
            }

            // Extract interfaces (implements)
            if let Some(interfaces_node) = node.child_by_field_name("interfaces") {
                let mut cursor = interfaces_node.walk();
                for child in interfaces_node.children(&mut cursor) {
                    if child.kind() == "type_identifier" || child.kind() == "generic_type" {
                        let interface_name = self.get_node_text(&child, source);
                        if !interface_name.trim().is_empty() {
                            let dependency = self.create_dependency(
                                class_name.clone(),
                                interface_name,
                                DependencyType::Implements,
                                &child,
                                context,
                            );
                            dependencies.push(dependency);
                        }
                    }
                }
            }

            // Extract interface extends
            if node.kind() == "interface_declaration" {
                if let Some(extends_node) = node.child_by_field_name("extends") {
                    let mut cursor = extends_node.walk();
                    for child in extends_node.children(&mut cursor) {
                        if child.kind() == "type_identifier" || child.kind() == "generic_type" {
                            let parent_interface = self.get_node_text(&child, source);
                            if !parent_interface.trim().is_empty() {
                                let dependency = self.create_dependency(
                                    class_name.clone(),
                                    parent_interface,
                                    DependencyType::Inherits,
                                    &child,
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

    /// Extract annotation dependencies
    fn extract_annotation_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let annotation_name = self.get_node_text(&name_node, source);
            let current_scope = context.current_scope();

            if !annotation_name.trim().is_empty() {
                let dependency = self.create_dependency(
                    current_scope,
                    annotation_name,
                    DependencyType::Uses,
                    &name_node,
                    context,
                );
                dependencies.push(dependency);
            }
        }
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
        if let Some(name_node) = node.child_by_field_name("name") {
            if let Some(value_node) = node.child_by_field_name("value") {
                let var_name = self.get_node_text(&name_node, source);

                // Extract dependencies from the value expression
                self.extract_expression_dependencies(
                    value_node, source, context, dependencies, &var_name
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

    /// Check if a string is a Java keyword
    fn is_java_keyword(&self, name: &str) -> bool {
        matches!(name,
            "abstract" | "assert" | "boolean" | "break" | "byte" | "case" | "catch" |
            "char" | "class" | "const" | "continue" | "default" | "do" | "double" |
            "else" | "enum" | "extends" | "final" | "finally" | "float" | "for" |
            "goto" | "if" | "implements" | "import" | "instanceof" | "int" |
            "interface" | "long" | "native" | "new" | "package" | "private" |
            "protected" | "public" | "return" | "short" | "static" | "strictfp" |
            "super" | "switch" | "synchronized" | "this" | "throw" | "throws" |
            "transient" | "try" | "void" | "volatile" | "while" |
            // Built-in types and common classes
            "String" | "Object" | "Integer" | "Boolean" | "Double" | "Float" |
            "Long" | "Short" | "Byte" | "Character" | "System" | "Math" |
            "true" | "false" | "null"
        )
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
            "identifier" => {
                let referenced_var = self.get_node_text(&node, source);
                if !self.is_java_keyword(&referenced_var) && !referenced_var.trim().is_empty() {
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
            "method_invocation" | "object_creation_expression" => {
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
            "identifier" => {
                let var_name = self.get_node_text(&node, source);
                if !self.is_java_keyword(&var_name) && !var_name.trim().is_empty() {
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
            if child.kind() == "identifier" {
                let arg_name = self.get_node_text(&child, source);
                let current_scope = context.current_scope();

                if !self.is_java_keyword(&arg_name) && !arg_name.trim().is_empty() {
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
                "variable_declarator" | "formal_parameter" | "import_declaration" => {
                    // Check if this identifier is the name being declared
                    if let Some(name_field) = parent.child_by_field_name("name") {
                        if name_field.id() == node.id() {
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
                // for (init; condition; update)
                if let Some(condition_node) = node.child_by_field_name("condition") {
                    self.extract_condition_variables(condition_node, source, context, dependencies, &current_scope);
                }
                if let Some(init_node) = node.child_by_field_name("init") {
                    self.extract_condition_variables(init_node, source, context, dependencies, &current_scope);
                }
                if let Some(update_node) = node.child_by_field_name("update") {
                    self.extract_condition_variables(update_node, source, context, dependencies, &current_scope);
                }
            }
            "enhanced_for_statement" => {
                // for (Type var : iterable)
                if let Some(name_node) = node.child_by_field_name("name") {
                    let var_name = self.get_node_text(&name_node, source);
                    if !var_name.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            var_name,
                            DependencyType::LoopIteration,
                            &name_node,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }

                if let Some(value_node) = node.child_by_field_name("value") {
                    self.extract_condition_variables(value_node, source, context, dependencies, &current_scope);
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
                if let Some(parameter_node) = node.child_by_field_name("parameter") {
                    if let Some(type_node) = parameter_node.child_by_field_name("type") {
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

                    if let Some(name_node) = parameter_node.child_by_field_name("name") {
                        let var_name = self.get_node_text(&name_node, source);
                        if !var_name.trim().is_empty() {
                            let dependency = self.create_dependency(
                                current_scope.clone(),
                                var_name,
                                DependencyType::ExceptionHandling,
                                &name_node,
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
        if let Some(condition_node) = node.child_by_field_name("condition") {
            self.extract_condition_variables(condition_node, source, context, dependencies, &current_scope);
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

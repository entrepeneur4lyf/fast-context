//! Dart-specific dependency extraction
//!
//! Extracts dependency relationships from Dart source code, including:
//! - Function calls and method invocations
//! - Class inheritance and mixin usage
//! - Extension methods and extension types
//! - Import statements and library dependencies
//! - Async/await patterns and futures
//! - Control flow (if/else, for/while loops, switch)
//! - Exception handling (try/catch/finally)
//! - Generic types and type parameters

use super::{BaseDependencyExtractor, DependencyExtractor, ExtractionContext};
use crate::parsers::LanguageId;
use crate::symbols::{Dependency, DependencyType};
use tree_sitter::Node;

/// Dart-specific dependency extractor
pub struct DartDependencyExtractor;

impl DependencyExtractor for DartDependencyExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Dart
    }

    fn extract_dependencies(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
        context: &mut ExtractionContext,
    ) -> Vec<Dependency> {
        let mut dependencies = Vec::new();
        BaseDependencyExtractor::traverse_node(
            self,
            tree.root_node(),
            source,
            context,
            &mut dependencies,
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
            "invocation" => {
                self.extract_function_calls(node, source, context, dependencies);
            }
            "selector" => {
                self.extract_property_access(node, source, context, dependencies);
            }
            "identifier" => {
                self.extract_variable_references(node, source, context, dependencies);
            }
            "import_specification" | "export_specification" => {
                self.extract_imports(node, source, context, dependencies);
            }
            "class_definition" | "mixin_declaration" | "extension_declaration" => {
                self.extract_class_inheritance(node, source, context, dependencies);
            }
            "assignment_expression" | "variable_declaration" => {
                self.extract_assignments(node, source, context, dependencies);
            }
            "lambda_expression" | "function_expression" => {
                self.extract_lambda_dependencies(node, source, context, dependencies);
            }
            // Control flow
            "if_statement" => {
                self.extract_conditional_dependencies(node, source, context, dependencies);
            }
            "for_statement" | "while_statement" | "do_statement" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "switch_statement" => {
                self.extract_switch_dependencies(node, source, context, dependencies);
            }
            "try_statement" => {
                self.extract_exception_dependencies(node, source, context, dependencies);
            }
            "return_statement" => {
                self.extract_return_dependencies(node, source, context, dependencies);
            }
            "break_statement" | "continue_statement" => {
                self.extract_break_continue_dependencies(node, source, context, dependencies);
            }
            "await_expression" => {
                self.extract_await_dependencies(node, source, context, dependencies);
            }
            _ => {}
        }
    }

    fn is_function_call(&self, node: &Node) -> bool {
        matches!(node.kind(), "invocation")
    }

    fn is_variable_reference(&self, node: &Node) -> bool {
        matches!(node.kind(), "identifier" | "selector")
    }

    fn is_import_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "import_specification" | "export_specification")
    }

    fn is_inheritance(&self, node: &Node) -> bool {
        matches!(node.kind(), "class_definition")
            && (node.child_by_field_name("superclass").is_some()
                || node.child_by_field_name("interfaces").is_some()
                || node.child_by_field_name("mixins").is_some())
    }

    fn is_assignment(&self, node: &Node) -> bool {
        matches!(
            node.kind(),
            "assignment_expression" | "variable_declaration"
        )
    }

    fn extract_function_calls(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(function_node) = node.child_by_field_name("function") {
            let function_name = self.get_node_text(&function_node, source);
            let current_scope = context.current_scope();

            if !function_name.trim().is_empty() && !function_name.contains('\n') {
                let resolved_functions = context.find_symbols_global(&function_name);
                let target_function = if !resolved_functions.is_empty() {
                    resolved_functions[0].qualified_name()
                } else {
                    function_name
                };

                let dependency = self.create_dependency(
                    current_scope,
                    target_function,
                    DependencyType::Calls,
                    &node,
                    context,
                );

                dependencies.push(dependency);

                // Extract arguments
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
        if self.is_reference_context(&node) {
            let var_name = self.get_node_text(&node, source);
            let current_scope = context.current_scope();

            if self.is_dart_keyword(&var_name) || var_name.trim().is_empty() {
                return;
            }

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

        // import 'dart:core'; import 'package:flutter/material.dart';
        if let Some(uri_node) = node.child_by_field_name("uri") {
            let import_uri = self
                .get_node_text(&uri_node, source)
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();

            if !import_uri.trim().is_empty() {
                let dependency_type =
                    if import_uri.starts_with("dart:") || import_uri.starts_with("package:") {
                        DependencyType::ModuleDependency
                    } else {
                        DependencyType::Imports
                    };

                let dependency = self.create_dependency(
                    current_scope.clone(),
                    import_uri,
                    dependency_type,
                    &uri_node,
                    context,
                );
                dependencies.push(dependency);
            }
        }

        // Extract show/hide clauses
        if let Some(combinators_node) = node.child_by_field_name("combinators") {
            let mut cursor = combinators_node.walk();
            for child in combinators_node.children(&mut cursor) {
                if child.kind() == "show_combinator" || child.kind() == "hide_combinator" {
                    let mut names_cursor = child.walk();
                    for name_child in child.children(&mut names_cursor) {
                        if name_child.kind() == "identifier" {
                            let symbol_name = self.get_node_text(&name_child, source);
                            if !symbol_name.trim().is_empty() {
                                let dependency = self.create_dependency(
                                    current_scope.clone(),
                                    symbol_name,
                                    DependencyType::Imports,
                                    &name_child,
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
            "variable_declaration" => {
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
            "for_statement" | "while_statement" | "do_statement" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "switch_statement" => {
                self.extract_switch_dependencies(node, source, context, dependencies);
            }
            "try_statement" => {
                self.extract_exception_dependencies(node, source, context, dependencies);
            }
            "return_statement" => {
                self.extract_return_dependencies(node, source, context, dependencies);
            }
            "break_statement" | "continue_statement" => {
                self.extract_break_continue_dependencies(node, source, context, dependencies);
            }
            "await_expression" => {
                self.extract_await_dependencies(node, source, context, dependencies);
            }
            _ => {}
        }
    }

    fn is_conditional_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "if_statement")
    }

    fn is_loop_statement(&self, node: &Node) -> bool {
        matches!(
            node.kind(),
            "for_statement" | "while_statement" | "do_statement"
        )
    }

    fn is_exception_handling(&self, node: &Node) -> bool {
        matches!(node.kind(), "try_statement")
    }

    fn is_switch_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "switch_statement")
    }

    fn is_return_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "return_statement")
    }

    fn is_break_continue(&self, node: &Node) -> bool {
        matches!(node.kind(), "break_statement" | "continue_statement")
    }
}

impl DartDependencyExtractor {
    /// Extract property access (obj.property)
    fn extract_property_access(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        // Dart selectors are method/property access
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                let property_name = self.get_node_text(&child, source);
                let current_scope = context.current_scope();

                if !property_name.trim().is_empty() && !self.is_dart_keyword(&property_name) {
                    let dependency = self.create_dependency(
                        current_scope,
                        property_name,
                        DependencyType::References,
                        &child,
                        context,
                    );
                    dependencies.push(dependency);
                }
            }
        }
    }

    /// Extract class inheritance, mixins, and extensions
    fn extract_class_inheritance(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let class_name = self.get_node_text(&name_node, source);

            match node.kind() {
                "class_definition" => {
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
                            if child.kind() == "type_identifier" {
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

                    // Extract mixins (with)
                    if let Some(mixins_node) = node.child_by_field_name("mixins") {
                        let mut cursor = mixins_node.walk();
                        for child in mixins_node.children(&mut cursor) {
                            if child.kind() == "type_identifier" {
                                let mixin_name = self.get_node_text(&child, source);
                                if !mixin_name.trim().is_empty() {
                                    let dependency = self.create_dependency(
                                        class_name.clone(),
                                        mixin_name,
                                        DependencyType::Uses,
                                        &child,
                                        context,
                                    );
                                    dependencies.push(dependency);
                                }
                            }
                        }
                    }
                }
                "mixin_declaration" => {
                    // Extract mixin constraints (on)
                    if let Some(on_clause_node) = node.child_by_field_name("on") {
                        let mut cursor = on_clause_node.walk();
                        for child in on_clause_node.children(&mut cursor) {
                            if child.kind() == "type_identifier" {
                                let constraint_type = self.get_node_text(&child, source);
                                if !constraint_type.trim().is_empty() {
                                    let dependency = self.create_dependency(
                                        class_name.clone(),
                                        constraint_type,
                                        DependencyType::References,
                                        &child,
                                        context,
                                    );
                                    dependencies.push(dependency);
                                }
                            }
                        }
                    }
                }
                "extension_declaration" => {
                    // Extract extended type
                    if let Some(type_node) = node.child_by_field_name("type") {
                        let extended_type = self.get_node_text(&type_node, source);
                        if !extended_type.trim().is_empty() {
                            let dependency = self.create_dependency(
                                class_name.clone(),
                                extended_type,
                                DependencyType::Uses,
                                &type_node,
                                context,
                            );
                            dependencies.push(dependency);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Extract lambda/function expression dependencies
    fn extract_lambda_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        // Extract parameters
        if let Some(params_node) = node.child_by_field_name("parameters") {
            self.extract_parameter_dependencies(
                params_node,
                source,
                context,
                dependencies,
                &current_scope,
            );
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

    /// Extract await expression dependencies
    fn extract_await_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        // Extract the awaited expression
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "await" {
                self.extract_expression_dependencies(
                    child,
                    source,
                    context,
                    dependencies,
                    &current_scope,
                );
            }
        }

        // Mark as async dependency
        let dependency = self.create_dependency(
            current_scope,
            "await_expression".to_string(),
            DependencyType::ControlFlow,
            &node,
            context,
        );
        dependencies.push(dependency);
    }

    /// Check if a string is a Dart keyword
    fn is_dart_keyword(&self, name: &str) -> bool {
        matches!(
            name,
            // Dart keywords
            "abstract" | "as" | "assert" | "async" | "await" | "break" | "case" |
            "catch" | "class" | "const" | "continue" | "covariant" | "default" |
            "deferred" | "do" | "dynamic" | "else" | "enum" | "export" | "extends" |
            "extension" | "external" | "factory" | "false" | "final" | "finally" |
            "for" | "Function" | "get" | "hide" | "if" | "implements" | "import" |
            "in" | "interface" | "is" | "late" | "library" | "mixin" | "new" |
            "null" | "on" | "operator" | "part" | "required" | "rethrow" |
            "return" | "set" | "show" | "static" | "super" | "switch" | "sync" |
            "this" | "throw" | "true" | "try" | "typedef" | "var" | "void" |
            "while" | "with" | "yield" |
            // Built-in types
            "bool" | "double" | "int" | "num" | "String" | "Object" | "List" |
            "Map" | "Set" | "Iterable" | "Future" | "Stream" | "Duration" |
            "DateTime" | "RegExp" | "Uri" | "Type"
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
                self.extract_expression_dependencies(
                    right_node,
                    source,
                    context,
                    dependencies,
                    &var_name,
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
                self.extract_expression_dependencies(
                    value_node,
                    source,
                    context,
                    dependencies,
                    &var_name,
                );
            }

            // Extract type annotation
            if let Some(type_node) = node.child_by_field_name("type") {
                let type_name = self.get_node_text(&type_node, source);
                let var_name = self.get_node_text(&name_node, source);

                if !type_name.trim().is_empty() && !self.is_dart_keyword(&type_name) {
                    let dependency = self.create_dependency(
                        var_name,
                        type_name,
                        DependencyType::References,
                        &type_node,
                        context,
                    );
                    dependencies.push(dependency);
                }
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

        if let Some(condition_node) = node.child_by_field_name("condition") {
            self.extract_condition_variables(
                condition_node,
                source,
                context,
                dependencies,
                &current_scope,
            );
        }

        let dependency = self.create_dependency(
            current_scope,
            "if_statement".to_string(),
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
            "identifier" => {
                let referenced_var = self.get_node_text(&node, source);
                if !self.is_dart_keyword(&referenced_var) && !referenced_var.trim().is_empty() {
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
            "invocation" => {
                self.extract_function_calls(node, source, context, dependencies);
            }
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.extract_expression_dependencies(
                        child,
                        source,
                        context,
                        dependencies,
                        assigner,
                    );
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
                if !self.is_dart_keyword(&var_name) && !var_name.trim().is_empty() {
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
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.extract_condition_variables(child, source, context, dependencies, scope);
                }
            }
        }
    }

    /// Extract parameter dependencies
    fn extract_parameter_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
        scope: &str,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "formal_parameter" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let param_name = self.get_node_text(&name_node, source);
                    if !param_name.trim().is_empty() {
                        let dependency = self.create_dependency(
                            scope.to_string(),
                            param_name,
                            DependencyType::References,
                            &name_node,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }

                if let Some(type_node) = child.child_by_field_name("type") {
                    let type_name = self.get_node_text(&type_node, source);
                    if !type_name.trim().is_empty() && !self.is_dart_keyword(&type_name) {
                        let dependency = self.create_dependency(
                            scope.to_string(),
                            type_name,
                            DependencyType::References,
                            &type_node,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }
            }
        }
    }

    /// Extract argument references
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

                if !self.is_dart_keyword(&arg_name) && !arg_name.trim().is_empty() {
                    let dependency = self.create_dependency(
                        current_scope,
                        arg_name,
                        DependencyType::References,
                        &child,
                        context,
                    );
                    dependencies.push(dependency);
                }
            } else {
                self.extract_argument_references(child, source, context, dependencies);
            }
        }
    }

    /// Check if a node is in a reference context
    fn is_reference_context(&self, node: &Node) -> bool {
        let mut current = *node;

        while let Some(parent) = current.parent() {
            match parent.kind() {
                "function_signature"
                | "class_definition"
                | "mixin_declaration"
                | "extension_declaration"
                | "variable_declaration"
                | "formal_parameter" => {
                    if let Some(name_field) = parent.child_by_field_name("name") {
                        if name_field.id() == node.id() {
                            return false;
                        }
                    }
                }
                "assignment_expression" => {
                    if let Some(left_field) = parent.child_by_field_name("left") {
                        if left_field.id() == node.id() {
                            return false;
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
                // for (var i = 0; i < 10; i++) or for (item in collection)
                if let Some(init_node) = node.child_by_field_name("initializer") {
                    self.extract_condition_variables(
                        init_node,
                        source,
                        context,
                        dependencies,
                        &current_scope,
                    );
                }
                if let Some(condition_node) = node.child_by_field_name("condition") {
                    self.extract_condition_variables(
                        condition_node,
                        source,
                        context,
                        dependencies,
                        &current_scope,
                    );
                }
                if let Some(update_node) = node.child_by_field_name("increment") {
                    self.extract_condition_variables(
                        update_node,
                        source,
                        context,
                        dependencies,
                        &current_scope,
                    );
                }

                // Enhanced for loop (for-in)
                if let Some(iterable_node) = node.child_by_field_name("iterable") {
                    self.extract_condition_variables(
                        iterable_node,
                        source,
                        context,
                        dependencies,
                        &current_scope,
                    );
                }
                if let Some(loop_var_node) = node.child_by_field_name("loop_variable") {
                    let var_name = self.get_node_text(&loop_var_node, source);
                    if !var_name.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            var_name,
                            DependencyType::LoopIteration,
                            &loop_var_node,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }
            }
            "while_statement" | "do_statement" => {
                if let Some(condition_node) = node.child_by_field_name("condition") {
                    self.extract_condition_variables(
                        condition_node,
                        source,
                        context,
                        dependencies,
                        &current_scope,
                    );
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

        // Extract catch clauses
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "catch_clause" {
                if let Some(exception_type_node) = child.child_by_field_name("exception_type") {
                    let exception_type = self.get_node_text(&exception_type_node, source);
                    if !exception_type.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            exception_type,
                            DependencyType::ExceptionHandling,
                            &exception_type_node,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }

                if let Some(exception_var_node) = child.child_by_field_name("exception_parameter") {
                    let var_name = self.get_node_text(&exception_var_node, source);
                    if !var_name.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            var_name,
                            DependencyType::ExceptionHandling,
                            &exception_var_node,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }
            }
        }

        let dependency = self.create_dependency(
            current_scope,
            "try_statement".to_string(),
            DependencyType::ExceptionHandling,
            &node,
            context,
        );
        dependencies.push(dependency);
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

        if let Some(expr_node) = node.child_by_field_name("expression") {
            self.extract_condition_variables(
                expr_node,
                source,
                context,
                dependencies,
                &current_scope,
            );
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

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "return" {
                self.extract_expression_dependencies(
                    child,
                    source,
                    context,
                    dependencies,
                    &current_scope,
                );
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
        let flow_type = if node.kind() == "break_statement" {
            "break"
        } else {
            "continue"
        };

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

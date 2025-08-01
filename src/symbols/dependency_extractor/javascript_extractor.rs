//! JavaScript/TypeScript-specific dependency extraction
//!
//! Extracts dependency relationships from JavaScript and TypeScript source code, including:
//! - Function calls and method invocations
//! - Variable references and assignments  
//! - Module imports and exports (ES6, CommonJS)
//! - Class inheritance and interfaces (TypeScript)
//! - Exception handling (try/catch/finally)
//! - Control flow (if/else, loops, switch)
//! - Async/await patterns

use super::{BaseDependencyExtractor, DependencyExtractor, ExtractionContext};
use crate::parsers::LanguageId;
use crate::symbols::{Dependency, DependencyType};
use tree_sitter::Node;

/// JavaScript/TypeScript-specific dependency extractor
pub struct JavaScriptDependencyExtractor;

impl DependencyExtractor for JavaScriptDependencyExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::JavaScript
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
            "call_expression" => {
                self.extract_function_calls(node, source, context, dependencies);
            }
            "member_expression" => {
                self.extract_member_access(node, source, context, dependencies);
            }
            "identifier" => {
                self.extract_variable_references(node, source, context, dependencies);
            }
            "import_statement" | "import_clause" => {
                self.extract_imports(node, source, context, dependencies);
            }
            "export_statement" => {
                self.extract_exports(node, source, context, dependencies);
            }
            "class_declaration" => {
                self.extract_class_inheritance(node, source, context, dependencies);
            }
            "assignment_expression" | "variable_declarator" => {
                self.extract_assignments(node, source, context, dependencies);
            }
            // Control flow
            "if_statement" => {
                self.extract_conditional_dependencies(node, source, context, dependencies);
            }
            "for_statement" | "for_in_statement" | "for_of_statement" | "while_statement"
            | "do_statement" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "try_statement" | "catch_clause" | "finally_clause" => {
                self.extract_exception_dependencies(node, source, context, dependencies);
            }
            "switch_statement" | "switch_case" => {
                self.extract_switch_dependencies(node, source, context, dependencies);
            }
            "return_statement" => {
                self.extract_return_dependencies(node, source, context, dependencies);
            }
            "break_statement" | "continue_statement" => {
                self.extract_break_continue_dependencies(node, source, context, dependencies);
            }
            "await_expression" | "yield_expression" => {
                self.extract_async_dependencies(node, source, context, dependencies);
            }
            _ => {}
        }
    }

    fn is_function_call(&self, node: &Node) -> bool {
        matches!(node.kind(), "call_expression")
    }

    fn is_variable_reference(&self, node: &Node) -> bool {
        matches!(node.kind(), "identifier" | "member_expression")
    }

    fn is_import_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "import_statement" | "import_clause")
    }

    fn is_inheritance(&self, node: &Node) -> bool {
        node.kind() == "class_declaration" && node.child_by_field_name("superclass").is_some()
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
        if let Some(function_node) = node.child_by_field_name("function") {
            let function_name = self.get_node_text(&function_node, source);
            let current_scope = context.current_scope();

            if !function_name.trim().is_empty() && !function_name.contains('\n') {
                // Try to resolve the function in known symbols
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
            if self.is_javascript_keyword(&var_name) || var_name.trim().is_empty() {
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

        if node.kind() == "import_statement" {
            // import { name1, name2 } from 'module'
            // import * as name from 'module'
            // import name from 'module'
            if let Some(source_node) = node.child_by_field_name("source") {
                let module_name = self
                    .get_node_text(&source_node, source)
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();

                if !module_name.trim().is_empty() {
                    let dependency = self.create_dependency(
                        current_scope.clone(),
                        module_name,
                        DependencyType::ModuleDependency,
                        &source_node,
                        context,
                    );
                    dependencies.push(dependency);
                }
            }

            // Extract imported names
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                match child.kind() {
                    "import_specifier" | "import_clause" => {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            let import_name = self.get_node_text(&name_node, source);
                            if !import_name.trim().is_empty() {
                                let dependency = self.create_dependency(
                                    current_scope.clone(),
                                    import_name,
                                    DependencyType::Imports,
                                    &name_node,
                                    context,
                                );
                                dependencies.push(dependency);
                            }
                        }
                    }
                    "namespace_import" => {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            let namespace_name = self.get_node_text(&name_node, source);
                            if !namespace_name.trim().is_empty() {
                                let dependency = self.create_dependency(
                                    current_scope.clone(),
                                    namespace_name,
                                    DependencyType::NamespaceUsage,
                                    &name_node,
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
            "for_statement" | "for_in_statement" | "for_of_statement" | "while_statement"
            | "do_statement" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "try_statement" | "catch_clause" | "finally_clause" => {
                self.extract_exception_dependencies(node, source, context, dependencies);
            }
            "switch_statement" | "switch_case" => {
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
        matches!(
            node.kind(),
            "for_statement"
                | "for_in_statement"
                | "for_of_statement"
                | "while_statement"
                | "do_statement"
        )
    }

    fn is_exception_handling(&self, node: &Node) -> bool {
        matches!(
            node.kind(),
            "try_statement" | "catch_clause" | "finally_clause"
        )
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

impl JavaScriptDependencyExtractor {
    /// Extract member access (obj.prop)
    fn extract_member_access(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(object_node) = node.child_by_field_name("object") {
            let object_name = self.get_node_text(&object_node, source);
            let current_scope = context.current_scope();

            if !object_name.trim().is_empty() && !self.is_javascript_keyword(&object_name) {
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

    /// Extract exports
    fn extract_exports(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        // Extract exported names
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "export_specifier" => {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let export_name = self.get_node_text(&name_node, source);
                        if !export_name.trim().is_empty() {
                            let dependency = self.create_dependency(
                                current_scope.clone(),
                                export_name,
                                DependencyType::Export,
                                &name_node,
                                context,
                            );
                            dependencies.push(dependency);
                        }
                    }
                }
                "identifier" => {
                    let export_name = self.get_node_text(&child, source);
                    if !export_name.trim().is_empty() && !self.is_javascript_keyword(&export_name) {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            export_name,
                            DependencyType::Export,
                            &child,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }
                _ => {}
            }
        }
    }

    /// Extract class inheritance dependencies
    fn extract_class_inheritance(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(class_name_node) = node.child_by_field_name("name") {
            let class_name = self.get_node_text(&class_name_node, source);

            if let Some(superclass_node) = node.child_by_field_name("superclass") {
                let parent_class = self.get_node_text(&superclass_node, source);
                if !parent_class.trim().is_empty() {
                    let dependency = self.create_dependency(
                        class_name,
                        parent_class,
                        DependencyType::Inherits,
                        &superclass_node,
                        context,
                    );
                    dependencies.push(dependency);
                }
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

                // Extract dependencies from the value expression
                self.extract_expression_dependencies(
                    value_node,
                    source,
                    context,
                    dependencies,
                    &var_name,
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
            self.extract_condition_variables(
                condition_node,
                source,
                context,
                dependencies,
                &current_scope,
            );
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
                    self.extract_condition_variables(
                        condition_node,
                        source,
                        context,
                        dependencies,
                        &current_scope,
                    );
                }
                if let Some(init_node) = node.child_by_field_name("init") {
                    self.extract_condition_variables(
                        init_node,
                        source,
                        context,
                        dependencies,
                        &current_scope,
                    );
                }
                if let Some(update_node) = node.child_by_field_name("update") {
                    self.extract_condition_variables(
                        update_node,
                        source,
                        context,
                        dependencies,
                        &current_scope,
                    );
                }
            }
            "for_in_statement" | "for_of_statement" => {
                // for (left in/of right)
                if let Some(left_node) = node.child_by_field_name("left") {
                    let target_name = self.get_node_text(&left_node, source);
                    if !target_name.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            target_name,
                            DependencyType::LoopIteration,
                            &left_node,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }

                if let Some(right_node) = node.child_by_field_name("right") {
                    self.extract_condition_variables(
                        right_node,
                        source,
                        context,
                        dependencies,
                        &current_scope,
                    );
                }
            }
            "while_statement" | "do_statement" => {
                // while (condition) or do ... while (condition)
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

        match node.kind() {
            "catch_clause" => {
                // catch (error)
                if let Some(parameter_node) = node.child_by_field_name("parameter") {
                    let error_var = self.get_node_text(&parameter_node, source);
                    if !error_var.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            error_var,
                            DependencyType::ExceptionHandling,
                            &parameter_node,
                            context,
                        );
                        dependencies.push(dependency);
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

        match node.kind() {
            "switch_statement" => {
                // switch (discriminant)
                if let Some(discriminant_node) = node.child_by_field_name("value") {
                    self.extract_condition_variables(
                        discriminant_node,
                        source,
                        context,
                        dependencies,
                        &current_scope,
                    );
                }
            }
            "switch_case" => {
                // case value:
                if let Some(value_node) = node.child_by_field_name("value") {
                    self.extract_condition_variables(
                        value_node,
                        source,
                        context,
                        dependencies,
                        &current_scope,
                    );
                }

                let dependency = self.create_dependency(
                    current_scope,
                    "switch_case".to_string(),
                    DependencyType::SwitchCase,
                    &node,
                    context,
                );
                dependencies.push(dependency);
            }
            _ => {}
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

    /// Extract async/await dependencies
    fn extract_async_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        // Extract the awaited/yielded expression
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "await" && child.kind() != "yield" {
                self.extract_expression_dependencies(
                    child,
                    source,
                    context,
                    dependencies,
                    &current_scope,
                );
            }
        }

        let async_type = if node.kind() == "await_expression" {
            "await"
        } else {
            "yield"
        };
        let dependency = self.create_dependency(
            current_scope,
            async_type.to_string(),
            DependencyType::ControlFlow,
            &node,
            context,
        );
        dependencies.push(dependency);
    }

    /// Check if a string is a JavaScript keyword
    fn is_javascript_keyword(&self, name: &str) -> bool {
        matches!(
            name,
            "abstract" | "arguments" | "await" | "boolean" | "break" | "byte" | "case" |
            "catch" | "char" | "class" | "const" | "continue" | "debugger" | "default" |
            "delete" | "do" | "double" | "else" | "enum" | "eval" | "export" | "extends" |
            "false" | "final" | "finally" | "float" | "for" | "function" | "goto" |
            "if" | "implements" | "import" | "in" | "instanceof" | "int" | "interface" |
            "let" | "long" | "native" | "new" | "null" | "package" | "private" |
            "protected" | "public" | "return" | "short" | "static" | "super" | "switch" |
            "synchronized" | "this" | "throw" | "throws" | "transient" | "true" | "try" |
            "typeof" | "var" | "void" | "volatile" | "while" | "with" | "yield" |
            // Built-in objects and functions
            "Array" | "Boolean" | "Date" | "Error" | "Function" | "JSON" | "Math" |
            "Number" | "Object" | "RegExp" | "String" | "console" | "document" | "window" |
            "undefined" | "NaN" | "Infinity"
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
                if !self.is_javascript_keyword(&referenced_var) && !referenced_var.trim().is_empty()
                {
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
                if !self.is_javascript_keyword(&var_name) && !var_name.trim().is_empty() {
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

                if !self.is_javascript_keyword(&arg_name) && !arg_name.trim().is_empty() {
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
                "function_declaration"
                | "class_declaration"
                | "variable_declarator"
                | "formal_parameter"
                | "import_specifier"
                | "export_specifier" => {
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
}

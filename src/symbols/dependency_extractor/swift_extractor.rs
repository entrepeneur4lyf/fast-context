//! Swift-specific dependency extraction
//!
//! Extracts dependency relationships from Swift source code, including:
//! - Function calls and method invocations
//! - Property access and variable references
//! - Protocol conformance and class inheritance
//! - Import statements and module dependencies
//! - Extension usage and associated types
//! - Error handling (do/try/catch, throws)
//! - Control flow (if/guard/switch, loops)
//! - Closures and capture lists

use super::{BaseDependencyExtractor, DependencyExtractor, ExtractionContext};
use crate::parsers::LanguageId;
use crate::symbols::{Dependency, DependencyType};
use tree_sitter::Node;

/// Swift-specific dependency extractor
pub struct SwiftDependencyExtractor;

impl DependencyExtractor for SwiftDependencyExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Swift
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
            "navigation_expression" => {
                self.extract_property_access(node, source, context, dependencies);
            }
            "simple_identifier" => {
                self.extract_variable_references(node, source, context, dependencies);
            }
            "import_declaration" => {
                self.extract_imports(node, source, context, dependencies);
            }
            "class_declaration"
            | "protocol_declaration"
            | "struct_declaration"
            | "enum_declaration" => {
                self.extract_type_inheritance(node, source, context, dependencies);
            }
            "assignment" | "property_declaration" => {
                self.extract_assignments(node, source, context, dependencies);
            }
            "extension_declaration" => {
                self.extract_extension_dependencies(node, source, context, dependencies);
            }
            // Control flow
            "if_statement" | "guard_statement" => {
                self.extract_conditional_dependencies(node, source, context, dependencies);
            }
            "for_statement" | "while_statement" | "repeat_while_statement" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "do_statement" | "catch_clause" => {
                self.extract_exception_dependencies(node, source, context, dependencies);
            }
            "switch_statement" => {
                self.extract_switch_dependencies(node, source, context, dependencies);
            }
            "return_statement" => {
                self.extract_return_dependencies(node, source, context, dependencies);
            }
            "break_statement" | "continue_statement" => {
                self.extract_break_continue_dependencies(node, source, context, dependencies);
            }
            "closure_expression" => {
                self.extract_closure_dependencies(node, source, context, dependencies);
            }
            _ => {}
        }
    }

    fn is_function_call(&self, node: &Node) -> bool {
        matches!(node.kind(), "call_expression")
    }

    fn is_variable_reference(&self, node: &Node) -> bool {
        matches!(node.kind(), "simple_identifier" | "navigation_expression")
    }

    fn is_import_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "import_declaration")
    }

    fn is_inheritance(&self, node: &Node) -> bool {
        matches!(
            node.kind(),
            "class_declaration"
                | "protocol_declaration"
                | "struct_declaration"
                | "enum_declaration"
        ) && node.child_by_field_name("inheritance_clause").is_some()
    }

    fn is_assignment(&self, node: &Node) -> bool {
        matches!(node.kind(), "assignment" | "property_declaration")
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
            if self.is_swift_keyword(&var_name) || var_name.trim().is_empty() {
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

        // import Foundation, import UIKit.UIView, import class MyModule.MyClass
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" || child.kind() == "navigation_expression" {
                let import_path = self.get_node_text(&child, source);
                if !import_path.trim().is_empty() {
                    let dependency_type = if import_path.contains('.') {
                        DependencyType::Imports
                    } else {
                        DependencyType::ModuleDependency
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
        self.extract_type_inheritance(node, source, context, dependencies);
    }

    fn extract_assignments(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        match node.kind() {
            "assignment" => {
                self.extract_assignment_dependencies(node, source, context, dependencies);
            }
            "property_declaration" => {
                self.extract_property_declaration_dependencies(node, source, context, dependencies);
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
            "if_statement" | "guard_statement" => {
                self.extract_conditional_dependencies(node, source, context, dependencies);
            }
            "for_statement" | "while_statement" | "repeat_while_statement" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "do_statement" | "catch_clause" => {
                self.extract_exception_dependencies(node, source, context, dependencies);
            }
            "switch_statement" => {
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
        matches!(node.kind(), "if_statement" | "guard_statement")
    }

    fn is_loop_statement(&self, node: &Node) -> bool {
        matches!(
            node.kind(),
            "for_statement" | "while_statement" | "repeat_while_statement"
        )
    }

    fn is_exception_handling(&self, node: &Node) -> bool {
        matches!(node.kind(), "do_statement" | "catch_clause")
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

impl SwiftDependencyExtractor {
    /// Extract property access (obj.property)
    fn extract_property_access(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(target_node) = node.child_by_field_name("target") {
            let object_name = self.get_node_text(&target_node, source);
            let current_scope = context.current_scope();

            if !object_name.trim().is_empty() && !self.is_swift_keyword(&object_name) {
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

    /// Extract type inheritance and protocol conformance
    fn extract_type_inheritance(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let type_name = self.get_node_text(&name_node, source);

            // Extract inheritance clause (: SuperClass, Protocol1, Protocol2)
            if let Some(inheritance_node) = node.child_by_field_name("inheritance_clause") {
                let mut cursor = inheritance_node.walk();
                for child in inheritance_node.children(&mut cursor) {
                    if child.kind() == "type_identifier" || child.kind() == "user_type" {
                        let inherited_type = self.get_node_text(&child, source);
                        if !inherited_type.trim().is_empty() {
                            // Determine if it's class inheritance or protocol conformance
                            let dependency_type = if self.is_likely_protocol(&inherited_type) {
                                DependencyType::Implements
                            } else {
                                DependencyType::Inherits
                            };

                            let dependency = self.create_dependency(
                                type_name.clone(),
                                inherited_type,
                                dependency_type,
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

    /// Extract extension dependencies
    fn extract_extension_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(type_node) = node.child_by_field_name("type") {
            let extended_type = self.get_node_text(&type_node, source);
            let current_scope = context.current_scope();

            if !extended_type.trim().is_empty() {
                let dependency = self.create_dependency(
                    current_scope.clone(),
                    extended_type,
                    DependencyType::Uses,
                    &type_node,
                    context,
                );
                dependencies.push(dependency);
            }
        }

        // Extract protocol conformance in extensions
        if let Some(inheritance_node) = node.child_by_field_name("inheritance_clause") {
            let current_scope = context.current_scope();
            let mut cursor = inheritance_node.walk();
            for child in inheritance_node.children(&mut cursor) {
                if child.kind() == "type_identifier" || child.kind() == "user_type" {
                    let protocol_name = self.get_node_text(&child, source);
                    if !protocol_name.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            protocol_name,
                            DependencyType::Implements,
                            &child,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }
            }
        }
    }

    /// Extract closure dependencies and capture lists
    fn extract_closure_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        // Extract capture list [weak self, unowned delegate]
        if let Some(capture_list_node) = node.child_by_field_name("capture_list") {
            let mut cursor = capture_list_node.walk();
            for child in capture_list_node.children(&mut cursor) {
                if child.kind() == "capture_list_item" {
                    if let Some(identifier_node) = child.child_by_field_name("name") {
                        let captured_var = self.get_node_text(&identifier_node, source);
                        if !captured_var.trim().is_empty() {
                            let dependency = self.create_dependency(
                                current_scope.clone(),
                                captured_var,
                                DependencyType::References,
                                &identifier_node,
                                context,
                            );
                            dependencies.push(dependency);
                        }
                    }
                }
            }
        }

        // Mark as closure dependency
        let dependency = self.create_dependency(
            current_scope,
            "closure_expression".to_string(),
            DependencyType::ControlFlow,
            &node,
            context,
        );
        dependencies.push(dependency);
    }

    /// Check if a type name is likely a protocol (starts with uppercase, often ends with 'able' or 'Protocol')
    fn is_likely_protocol(&self, type_name: &str) -> bool {
        type_name.ends_with("able") ||
        type_name.ends_with("Protocol") ||
        type_name.ends_with("Delegate") ||
        type_name.ends_with("DataSource") ||
        // Common Swift protocols
        matches!(type_name,
            "Equatable" | "Hashable" | "Comparable" | "Codable" | "CustomStringConvertible" |
            "ExpressibleByLiteral" | "Collection" | "Sequence" | "IteratorProtocol"
        )
    }

    /// Check if a string is a Swift keyword
    fn is_swift_keyword(&self, name: &str) -> bool {
        matches!(
            name,
            "associatedtype" | "class" | "deinit" | "enum" | "extension" | "fileprivate" |
            "func" | "import" | "init" | "inout" | "internal" | "let" | "open" |
            "operator" | "private" | "protocol" | "public" | "rethrows" | "static" |
            "struct" | "subscript" | "typealias" | "var" | "break" | "case" |
            "continue" | "default" | "defer" | "do" | "else" | "fallthrough" |
            "for" | "guard" | "if" | "in" | "repeat" | "return" | "switch" |
            "where" | "while" | "as" | "catch" | "false" | "is" | "nil" | "super" |
            "self" | "Self" | "throw" | "throws" | "true" | "try" | "async" | "await" |
            // Built-in types
            "Any" | "AnyObject" | "Bool" | "Character" | "Double" | "Float" |
            "Int" | "String" | "UInt" | "Void" | "Array" | "Dictionary" | "Set" |
            "Optional" | "Result"
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
        if let Some(target_node) = node.child_by_field_name("target") {
            if let Some(value_node) = node.child_by_field_name("value") {
                let var_name = self.get_node_text(&target_node, source);

                // Extract dependencies from the right-hand side
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

    /// Extract property declaration dependencies
    fn extract_property_declaration_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            if let Some(value_node) = node.child_by_field_name("value") {
                let var_name = self.get_node_text(&name_node, source);

                // Extract dependencies from the initializer expression
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

    /// Extract conditional dependencies (if/guard statements)
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

        // Extract guard let/var bindings
        if node.kind() == "guard_statement" {
            if let Some(condition_list_node) = node.child_by_field_name("condition") {
                self.extract_guard_bindings(
                    condition_list_node,
                    source,
                    context,
                    dependencies,
                    &current_scope,
                );
            }
        }

        // Mark as conditional execution
        let dependency = self.create_dependency(
            current_scope,
            format!("{}_block", node.kind()),
            DependencyType::ConditionalExecution,
            &node,
            context,
        );
        dependencies.push(dependency);
    }

    /// Extract guard statement bindings
    fn extract_guard_bindings(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
        scope: &str,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "optional_binding_condition" {
                if let Some(value_node) = child.child_by_field_name("value") {
                    self.extract_condition_variables(
                        value_node,
                        source,
                        context,
                        dependencies,
                        scope,
                    );
                }
            }
        }
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
            "simple_identifier" => {
                let referenced_var = self.get_node_text(&node, source);
                if !self.is_swift_keyword(&referenced_var) && !referenced_var.trim().is_empty() {
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
            "simple_identifier" => {
                let var_name = self.get_node_text(&node, source);
                if !self.is_swift_keyword(&var_name) && !var_name.trim().is_empty() {
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
            if child.kind() == "simple_identifier" {
                let arg_name = self.get_node_text(&child, source);
                let current_scope = context.current_scope();

                if !self.is_swift_keyword(&arg_name) && !arg_name.trim().is_empty() {
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
                | "struct_declaration"
                | "enum_declaration"
                | "protocol_declaration"
                | "property_declaration"
                | "parameter"
                | "import_declaration" => {
                    // Check if this identifier is the name being declared
                    if let Some(name_field) = parent.child_by_field_name("name") {
                        if name_field.id() == node.id() {
                            return false; // This is a declaration
                        }
                    }
                }
                "assignment" => {
                    // Check if this is the target of assignment
                    if let Some(target_field) = parent.child_by_field_name("target") {
                        if target_field.id() == node.id() {
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
                // for item in collection
                if let Some(pattern_node) = node.child_by_field_name("pattern") {
                    let var_name = self.get_node_text(&pattern_node, source);
                    if !var_name.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            var_name,
                            DependencyType::LoopIteration,
                            &pattern_node,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }

                if let Some(value_node) = node.child_by_field_name("value") {
                    self.extract_condition_variables(
                        value_node,
                        source,
                        context,
                        dependencies,
                        &current_scope,
                    );
                }
            }
            "while_statement" | "repeat_while_statement" => {
                // while condition or repeat { } while condition
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
                // catch let error as SpecificError
                if let Some(pattern_node) = node.child_by_field_name("pattern") {
                    let error_var = self.get_node_text(&pattern_node, source);
                    if !error_var.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            error_var,
                            DependencyType::ExceptionHandling,
                            &pattern_node,
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

        // Extract switch expression
        if let Some(expr_node) = node.child_by_field_name("expr") {
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
}

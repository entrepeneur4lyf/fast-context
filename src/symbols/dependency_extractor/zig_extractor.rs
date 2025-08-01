//! Zig-specific dependency extraction
//!
//! Extracts dependency relationships from Zig source code, including:
//! - Function calls and method invocations
//! - Struct and enum definitions with field access
//! - Import statements (@import) and module dependencies
//! - Comptime expressions and type generation
//! - Error handling (try/catch, error unions)
//! - Control flow (if/else, for/while loops, switch)
//! - Memory management and allocators
//! - Generic types and functions

use super::{BaseDependencyExtractor, DependencyExtractor, ExtractionContext};
use crate::parsers::LanguageId;
use crate::symbols::{Dependency, DependencyType};
use tree_sitter::Node;

/// Zig-specific dependency extractor
pub struct ZigDependencyExtractor;

impl DependencyExtractor for ZigDependencyExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Zig
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
            "field_access" => {
                self.extract_property_access(node, source, context, dependencies);
            }
            "identifier" => {
                self.extract_variable_references(node, source, context, dependencies);
            }
            "builtin_call_expression" => {
                self.extract_builtin_calls(node, source, context, dependencies);
            }
            "struct_declaration" | "enum_declaration" | "union_declaration" => {
                self.extract_type_dependencies(node, source, context, dependencies);
            }
            "assignment_expression" | "variable_declaration" => {
                self.extract_assignments(node, source, context, dependencies);
            }
            "comptime_expression" => {
                self.extract_comptime_dependencies(node, source, context, dependencies);
            }
            // Control flow
            "if_expression" => {
                self.extract_conditional_dependencies(node, source, context, dependencies);
            }
            "for_statement" | "while_statement" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "switch_expression" => {
                self.extract_switch_dependencies(node, source, context, dependencies);
            }
            "try_expression" | "catch_expression" => {
                self.extract_exception_dependencies(node, source, context, dependencies);
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
        matches!(node.kind(), "call_expression" | "builtin_call_expression")
    }

    fn is_variable_reference(&self, node: &Node) -> bool {
        matches!(node.kind(), "identifier" | "field_access")
    }

    fn is_import_statement(&self, node: &Node) -> bool {
        node.kind() == "builtin_call_expression" && self.is_import_builtin(node, "")
    }

    fn is_inheritance(&self, _node: &Node) -> bool {
        false // Zig doesn't have inheritance
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

            if self.is_zig_keyword(&var_name) || var_name.trim().is_empty() {
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
        self.extract_builtin_calls(node, source, context, dependencies);
    }

    fn extract_inheritance(
        &self,
        _node: Node,
        _source: &str,
        _context: &mut ExtractionContext,
        _dependencies: &mut Vec<Dependency>,
    ) {
        // Zig doesn't have inheritance
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
            "if_expression" => {
                self.extract_conditional_dependencies(node, source, context, dependencies);
            }
            "for_statement" | "while_statement" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "switch_expression" => {
                self.extract_switch_dependencies(node, source, context, dependencies);
            }
            "try_expression" | "catch_expression" => {
                self.extract_exception_dependencies(node, source, context, dependencies);
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
        matches!(node.kind(), "if_expression")
    }

    fn is_loop_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "for_statement" | "while_statement")
    }

    fn is_exception_handling(&self, node: &Node) -> bool {
        matches!(node.kind(), "try_expression" | "catch_expression")
    }

    fn is_switch_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "switch_expression")
    }

    fn is_return_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "return_statement")
    }

    fn is_break_continue(&self, node: &Node) -> bool {
        matches!(node.kind(), "break_statement" | "continue_statement")
    }
}

impl ZigDependencyExtractor {
    /// Extract property/field access (obj.field)
    fn extract_property_access(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(object_node) = node.child_by_field_name("object") {
            let object_name = self.get_node_text(&object_node, source);
            let current_scope = context.current_scope();

            if !object_name.trim().is_empty() && !self.is_zig_keyword(&object_name) {
                let dependency = self.create_dependency(
                    current_scope,
                    object_name,
                    DependencyType::References,
                    &node,
                    context,
                );
                dependencies.push(dependency);
            }
        }
    }

    /// Extract builtin function calls (@import, @typeInfo, etc.)
    fn extract_builtin_calls(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(function_node) = node.child_by_field_name("function") {
            let builtin_name = self.get_node_text(&function_node, source);
            let current_scope = context.current_scope();

            if builtin_name == "@import" {
                // Extract import path
                if let Some(args_node) = node.child_by_field_name("arguments") {
                    let mut cursor = args_node.walk();
                    for child in args_node.children(&mut cursor) {
                        if child.kind() == "string_literal" {
                            let import_path = self
                                .get_node_text(&child, source)
                                .trim_matches('"')
                                .to_string();

                            if !import_path.trim().is_empty() {
                                let dependency = self.create_dependency(
                                    current_scope.clone(),
                                    import_path,
                                    DependencyType::Imports,
                                    &child,
                                    context,
                                );
                                dependencies.push(dependency);
                            }
                        }
                    }
                }
            } else {
                // Other builtin functions
                let dependency = self.create_dependency(
                    current_scope,
                    builtin_name,
                    DependencyType::Calls,
                    &function_node,
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

    /// Check if a builtin call is an import
    fn is_import_builtin(&self, node: &Node, source: &str) -> bool {
        if let Some(function_node) = node.child_by_field_name("function") {
            let builtin_name = self.get_node_text(&function_node, source);
            builtin_name == "@import"
        } else {
            false
        }
    }

    /// Extract type dependencies (struct, enum, union)
    fn extract_type_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let type_name = self.get_node_text(&name_node, source);

            // Extract field types and dependencies
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "field_declaration" {
                    if let Some(type_node) = child.child_by_field_name("type") {
                        let field_type = self.get_node_text(&type_node, source);
                        if !field_type.trim().is_empty() && !self.is_zig_keyword(&field_type) {
                            let dependency = self.create_dependency(
                                type_name.clone(),
                                field_type,
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
    }

    /// Extract comptime dependencies
    fn extract_comptime_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        // Extract comptime expression dependencies
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "comptime" {
                self.extract_expression_dependencies(
                    child,
                    source,
                    context,
                    dependencies,
                    &current_scope,
                );
            }
        }

        // Mark as comptime dependency
        let dependency = self.create_dependency(
            current_scope,
            "comptime_expression".to_string(),
            DependencyType::ControlFlow,
            &node,
            context,
        );
        dependencies.push(dependency);
    }

    /// Check if a string is a Zig keyword
    fn is_zig_keyword(&self, name: &str) -> bool {
        matches!(
            name,
            // Zig keywords
            "align" | "allowzero" | "and" | "anyframe" | "anytype" | "asm" |
            "async" | "await" | "break" | "callconv" | "catch" | "comptime" |
            "const" | "continue" | "defer" | "else" | "enum" | "errdefer" |
            "error" | "export" | "extern" | "fn" | "for" | "if" | "inline" |
            "noalias" | "noinline" | "nosuspend" | "opaque" | "or" | "orelse" |
            "packed" | "pub" | "resume" | "return" | "linksection" | "struct" |
            "suspend" | "switch" | "test" | "threadlocal" | "try" | "union" |
            "unreachable" | "usingnamespace" | "var" | "volatile" | "while" |
            // Built-in types
            "bool" | "c_int" | "c_long" | "c_short" | "c_uint" | "c_ulong" |
            "c_ushort" | "f16" | "f32" | "f64" | "f80" | "f128" | "i8" | "i16" |
            "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" |
            "u128" | "usize" | "void" | "noreturn" | "type" | "anyerror" |
            "comptime_int" | "comptime_float" | "true" | "false" | "null" | "undefined"
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

                if !type_name.trim().is_empty() && !self.is_zig_keyword(&type_name) {
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
            "if_expression".to_string(),
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
                if !self.is_zig_keyword(&referenced_var) && !referenced_var.trim().is_empty() {
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
            "call_expression" | "builtin_call_expression" => {
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
                if !self.is_zig_keyword(&var_name) && !var_name.trim().is_empty() {
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

                if !self.is_zig_keyword(&arg_name) && !arg_name.trim().is_empty() {
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
                "function_declaration"
                | "struct_declaration"
                | "enum_declaration"
                | "union_declaration"
                | "variable_declaration"
                | "parameter_declaration" => {
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
                // for (items) |item, index|
                if let Some(iterable_node) = node.child_by_field_name("iterable") {
                    self.extract_condition_variables(
                        iterable_node,
                        source,
                        context,
                        dependencies,
                        &current_scope,
                    );
                }

                if let Some(capture_node) = node.child_by_field_name("capture") {
                    let mut cursor = capture_node.walk();
                    for child in capture_node.children(&mut cursor) {
                        if child.kind() == "identifier" {
                            let var_name = self.get_node_text(&child, source);
                            if !var_name.trim().is_empty() {
                                let dependency = self.create_dependency(
                                    current_scope.clone(),
                                    var_name,
                                    DependencyType::LoopIteration,
                                    &child,
                                    context,
                                );
                                dependencies.push(dependency);
                            }
                        }
                    }
                }
            }
            "while_statement" => {
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

    /// Extract exception handling dependencies (try/catch)
    fn extract_exception_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        match node.kind() {
            "try_expression" => {
                // Extract the expression being tried
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() != "try" {
                        self.extract_expression_dependencies(
                            child,
                            source,
                            context,
                            dependencies,
                            &current_scope,
                        );
                    }
                }
            }
            "catch_expression" => {
                // Extract error capture
                if let Some(capture_node) = node.child_by_field_name("capture") {
                    let error_var = self.get_node_text(&capture_node, source);
                    if !error_var.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            error_var,
                            DependencyType::ExceptionHandling,
                            &capture_node,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }
            }
            _ => {}
        }

        let dependency = self.create_dependency(
            current_scope,
            format!("{}_block", node.kind()),
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
            "switch_expression".to_string(),
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

//! Go-specific dependency extraction
//!
//! Extracts dependency relationships from Go source code, including:
//! - Function calls and method invocations
//! - Variable references and assignments
//! - Package imports and module dependencies
//! - Interface implementation (implicit)
//! - Struct embedding and composition
//! - Error handling patterns
//! - Control flow (if/else, loops, switch, select)
//! - Goroutines and channels

use super::{BaseDependencyExtractor, DependencyExtractor, ExtractionContext};
use crate::parsers::LanguageId;
use crate::symbols::{Dependency, DependencyType};
use tree_sitter::Node;

/// Go-specific dependency extractor
pub struct GoDependencyExtractor;

impl DependencyExtractor for GoDependencyExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Go
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
            "selector_expression" => {
                self.extract_method_calls(node, source, context, dependencies);
            }
            "identifier" => {
                self.extract_variable_references(node, source, context, dependencies);
            }
            "import_declaration" | "import_spec" => {
                self.extract_imports(node, source, context, dependencies);
            }
            "type_declaration" | "type_spec" => {
                self.extract_type_dependencies(node, source, context, dependencies);
            }
            "assignment_expression" | "short_var_declaration" | "var_declaration" => {
                self.extract_assignments(node, source, context, dependencies);
            }
            "go_statement" => {
                self.extract_goroutine_dependencies(node, source, context, dependencies);
            }
            "channel_type" | "send_statement" | "receive_expression" => {
                self.extract_channel_dependencies(node, source, context, dependencies);
            }
            // Control flow
            "if_statement" => {
                self.extract_conditional_dependencies(node, source, context, dependencies);
            }
            "for_statement" | "range_clause" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "switch_statement" | "type_switch_statement" | "select_statement" => {
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
        matches!(node.kind(), "call_expression")
    }

    fn is_variable_reference(&self, node: &Node) -> bool {
        matches!(node.kind(), "identifier" | "selector_expression")
    }

    fn is_import_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "import_declaration" | "import_spec")
    }

    fn is_inheritance(&self, node: &Node) -> bool {
        // Go doesn't have explicit inheritance, but has struct embedding
        node.kind() == "type_spec" && self.has_embedded_fields(node)
    }

    fn is_assignment(&self, node: &Node) -> bool {
        matches!(
            node.kind(),
            "assignment_expression" | "short_var_declaration" | "var_declaration"
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
            if self.is_go_keyword(&var_name) || var_name.trim().is_empty() {
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

        match node.kind() {
            "import_declaration" => {
                // import "package" or import ( ... )
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "import_spec" {
                        self.extract_import_spec(
                            child,
                            source,
                            context,
                            dependencies,
                            &current_scope,
                        );
                    }
                }
            }
            "import_spec" => {
                self.extract_import_spec(node, source, context, dependencies, &current_scope);
            }
            _ => {}
        }
    }

    fn extract_inheritance(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        self.extract_type_dependencies(node, source, context, dependencies);
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
            "short_var_declaration" => {
                self.extract_short_var_dependencies(node, source, context, dependencies);
            }
            "var_declaration" => {
                self.extract_var_declaration_dependencies(node, source, context, dependencies);
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
            "for_statement" | "range_clause" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "switch_statement" | "type_switch_statement" | "select_statement" => {
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
        matches!(node.kind(), "for_statement" | "range_clause")
    }

    fn is_exception_handling(&self, _node: &Node) -> bool {
        // Go doesn't have exceptions, but has error handling patterns
        false
    }

    fn is_switch_statement(&self, node: &Node) -> bool {
        matches!(
            node.kind(),
            "switch_statement" | "type_switch_statement" | "select_statement"
        )
    }

    fn is_return_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "return_statement")
    }

    fn is_break_continue(&self, node: &Node) -> bool {
        matches!(node.kind(), "break_statement" | "continue_statement")
    }
}

impl GoDependencyExtractor {
    /// Extract method calls from selector expressions (obj.method())
    fn extract_method_calls(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        // Check if this selector expression is part of a call
        if let Some(parent) = node.parent() {
            if parent.kind() == "call_expression" {
                // This is handled by extract_function_calls
                return;
            }
        }

        // This is a field/method access
        if let Some(operand_node) = node.child_by_field_name("operand") {
            let object_name = self.get_node_text(&operand_node, source);
            let current_scope = context.current_scope();

            if !object_name.trim().is_empty() && !self.is_go_keyword(&object_name) {
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

    /// Extract import spec dependencies
    fn extract_import_spec(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
        current_scope: &str,
    ) {
        // import "package/path" or import alias "package/path"
        if let Some(path_node) = node.child_by_field_name("path") {
            let import_path = self
                .get_node_text(&path_node, source)
                .trim_matches('"')
                .to_string();

            if !import_path.trim().is_empty() {
                let dependency = self.create_dependency(
                    current_scope.to_string(),
                    import_path,
                    DependencyType::ModuleDependency,
                    &path_node,
                    context,
                );
                dependencies.push(dependency);
            }
        }

        // Extract alias if present
        if let Some(name_node) = node.child_by_field_name("name") {
            let alias_name = self.get_node_text(&name_node, source);
            if !alias_name.trim().is_empty() && alias_name != "." && alias_name != "_" {
                let dependency = self.create_dependency(
                    current_scope.to_string(),
                    alias_name,
                    DependencyType::Imports,
                    &name_node,
                    context,
                );
                dependencies.push(dependency);
            }
        }
    }

    /// Extract type dependencies (struct embedding, interface implementation)
    fn extract_type_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let type_name = self.get_node_text(&name_node, source);

            if let Some(type_node) = node.child_by_field_name("type") {
                match type_node.kind() {
                    "struct_type" => {
                        self.extract_struct_embedding(
                            type_node,
                            source,
                            context,
                            dependencies,
                            &type_name,
                        );
                    }
                    "interface_type" => {
                        self.extract_interface_embedding(
                            type_node,
                            source,
                            context,
                            dependencies,
                            &type_name,
                        );
                    }
                    _ => {
                        // Type alias
                        let aliased_type = self.get_node_text(&type_node, source);
                        if !aliased_type.trim().is_empty() {
                            let dependency = self.create_dependency(
                                type_name,
                                aliased_type,
                                DependencyType::TypeOf,
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

    /// Extract struct embedding dependencies
    fn extract_struct_embedding(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
        struct_name: &str,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "field_declaration_list" {
                let mut field_cursor = child.walk();
                for field in child.children(&mut field_cursor) {
                    if field.kind() == "field_declaration" {
                        // Check for embedded field (no name, just type)
                        if field.child_by_field_name("name").is_none() {
                            if let Some(type_node) = field.child_by_field_name("type") {
                                let embedded_type = self.get_node_text(&type_node, source);
                                if !embedded_type.trim().is_empty() {
                                    let dependency = self.create_dependency(
                                        struct_name.to_string(),
                                        embedded_type,
                                        DependencyType::Uses,
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
    }

    /// Extract interface embedding dependencies
    fn extract_interface_embedding(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
        interface_name: &str,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "method_spec_list" {
                let mut method_cursor = child.walk();
                for method in child.children(&mut method_cursor) {
                    if method.kind() == "method_spec" {
                        // Interface method
                        if let Some(name_node) = method.child_by_field_name("name") {
                            let method_name = self.get_node_text(&name_node, source);
                            if !method_name.trim().is_empty() {
                                let dependency = self.create_dependency(
                                    interface_name.to_string(),
                                    method_name,
                                    DependencyType::Declares,
                                    &name_node,
                                    context,
                                );
                                dependencies.push(dependency);
                            }
                        }
                    } else if method.kind() == "type_identifier" {
                        // Embedded interface
                        let embedded_interface = self.get_node_text(&method, source);
                        if !embedded_interface.trim().is_empty() {
                            let dependency = self.create_dependency(
                                interface_name.to_string(),
                                embedded_interface,
                                DependencyType::Uses,
                                &method,
                                context,
                            );
                            dependencies.push(dependency);
                        }
                    }
                }
            }
        }
    }

    /// Extract goroutine dependencies
    fn extract_goroutine_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        // go function_call()
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "call_expression" {
                self.extract_function_calls(child, source, context, dependencies);

                // Also mark as goroutine dependency
                if let Some(function_node) = child.child_by_field_name("function") {
                    let function_name = self.get_node_text(&function_node, source);
                    if !function_name.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            format!("goroutine:{function_name}"),
                            DependencyType::ControlFlow,
                            &child,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }
            }
        }
    }

    /// Extract channel dependencies
    fn extract_channel_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        match node.kind() {
            "channel_type" => {
                // chan Type or <-chan Type or chan<- Type
                if let Some(element_node) = node.child_by_field_name("element") {
                    let element_type = self.get_node_text(&element_node, source);
                    if !element_type.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope,
                            format!("chan:{element_type}"),
                            DependencyType::TypeOf,
                            &element_node,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }
            }
            "send_statement" => {
                // channel <- value
                if let Some(channel_node) = node.child_by_field_name("channel") {
                    let channel_name = self.get_node_text(&channel_node, source);
                    if !channel_name.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            channel_name,
                            DependencyType::References,
                            &channel_node,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }

                if let Some(value_node) = node.child_by_field_name("value") {
                    self.extract_expression_dependencies(
                        value_node,
                        source,
                        context,
                        dependencies,
                        &current_scope,
                    );
                }
            }
            "receive_expression" => {
                // <-channel
                if let Some(operand_node) = node.child_by_field_name("operand") {
                    let channel_name = self.get_node_text(&operand_node, source);
                    if !channel_name.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope,
                            channel_name,
                            DependencyType::References,
                            &operand_node,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }
            }
            _ => {}
        }
    }

    /// Check if a type has embedded fields
    fn has_embedded_fields(&self, node: &Node) -> bool {
        if let Some(type_node) = node.child_by_field_name("type") {
            if type_node.kind() == "struct_type" {
                let mut cursor = type_node.walk();
                for child in type_node.children(&mut cursor) {
                    if child.kind() == "field_declaration_list" {
                        let mut field_cursor = child.walk();
                        for field in child.children(&mut field_cursor) {
                            if field.kind() == "field_declaration"
                                && field.child_by_field_name("name").is_none()
                            {
                                return true; // Found embedded field
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Check if a string is a Go keyword
    fn is_go_keyword(&self, name: &str) -> bool {
        matches!(
            name,
            "break" | "case" | "chan" | "const" | "continue" | "default" | "defer" |
            "else" | "fallthrough" | "for" | "func" | "go" | "goto" | "if" |
            "import" | "interface" | "map" | "package" | "range" | "return" |
            "select" | "struct" | "switch" | "type" | "var" |
            // Built-in types and functions
            "bool" | "byte" | "complex64" | "complex128" | "error" | "float32" |
            "float64" | "int" | "int8" | "int16" | "int32" | "int64" | "rune" |
            "string" | "uint" | "uint8" | "uint16" | "uint32" | "uint64" | "uintptr" |
            "true" | "false" | "iota" | "nil" |
            "append" | "cap" | "close" | "complex" | "copy" | "delete" | "imag" |
            "len" | "make" | "new" | "panic" | "print" | "println" | "real" | "recover"
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
                    right_node,
                    source,
                    context,
                    dependencies,
                    &var_name,
                );
            }
        }
    }

    /// Extract short variable declaration dependencies (x := value)
    fn extract_short_var_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(left_node) = node.child_by_field_name("left") {
            if let Some(right_node) = node.child_by_field_name("right") {
                let var_names = self.get_node_text(&left_node, source);

                // Extract dependencies from the right-hand side
                self.extract_expression_dependencies(
                    right_node,
                    source,
                    context,
                    dependencies,
                    &var_names,
                );
            }
        }
    }

    /// Extract var declaration dependencies
    fn extract_var_declaration_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "var_spec" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    if let Some(value_node) = child.child_by_field_name("value") {
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

        // Extract initialization statement (if x := value; condition)
        if let Some(init_node) = node.child_by_field_name("initializer") {
            self.extract_condition_variables(
                init_node,
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
                if !self.is_go_keyword(&referenced_var) && !referenced_var.trim().is_empty() {
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
                if !self.is_go_keyword(&var_name) && !var_name.trim().is_empty() {
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

                if !self.is_go_keyword(&arg_name) && !arg_name.trim().is_empty() {
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
                | "method_declaration"
                | "type_declaration"
                | "var_declaration"
                | "short_var_declaration"
                | "parameter_declaration"
                | "import_spec"
                | "field_declaration" => {
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
                // for init; condition; post
                if let Some(condition_node) = node.child_by_field_name("condition") {
                    self.extract_condition_variables(
                        condition_node,
                        source,
                        context,
                        dependencies,
                        &current_scope,
                    );
                }
                if let Some(init_node) = node.child_by_field_name("initializer") {
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
            "range_clause" => {
                // for key, value := range iterable
                if let Some(left_node) = node.child_by_field_name("left") {
                    let var_names = self.get_node_text(&left_node, source);
                    if !var_names.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            var_names,
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
            _ => {}
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
                // switch expr { ... }
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
            "type_switch_statement" => {
                // switch x := expr.(type) { ... }
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
            "select_statement" => {
                // select { case <-ch: ... }
                let dependency = self.create_dependency(
                    current_scope,
                    "select_statement".to_string(),
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
}

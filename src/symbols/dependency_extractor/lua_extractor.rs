//! Lua-specific dependency extraction
//!
//! Extracts dependency relationships from Lua source code, including:
//! - Function calls and method invocations
//! - Variable references and table access
//! - Module requires and dependencies
//! - Metatable operations and metamethods
//! - Control flow (if/else, for/while loops)
//! - Coroutine operations
//! - Table construction and field access

use super::{BaseDependencyExtractor, DependencyExtractor, ExtractionContext};
use crate::parsers::LanguageId;
use crate::symbols::{Dependency, DependencyType};
use tree_sitter::Node;

/// Lua-specific dependency extractor
pub struct LuaDependencyExtractor;

impl DependencyExtractor for LuaDependencyExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Lua
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
            "function_call" => {
                self.extract_function_calls(node, source, context, dependencies);
            }
            "dot_index_expression" | "bracket_index_expression" => {
                self.extract_property_access(node, source, context, dependencies);
            }
            "identifier" => {
                self.extract_variable_references(node, source, context, dependencies);
            }
            "assignment_statement" | "local_variable_declaration" => {
                self.extract_assignments(node, source, context, dependencies);
            }
            // Control flow
            "if_statement" => {
                self.extract_conditional_dependencies(node, source, context, dependencies);
            }
            "for_statement" | "while_statement" | "repeat_statement" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "return_statement" => {
                self.extract_return_dependencies(node, source, context, dependencies);
            }
            "break_statement" => {
                self.extract_break_continue_dependencies(node, source, context, dependencies);
            }
            _ => {}
        }
    }

    fn is_function_call(&self, node: &Node) -> bool {
        matches!(node.kind(), "function_call")
    }

    fn is_variable_reference(&self, node: &Node) -> bool {
        matches!(
            node.kind(),
            "identifier" | "dot_index_expression" | "bracket_index_expression"
        )
    }

    fn is_import_statement(&self, node: &Node) -> bool {
        node.kind() == "function_call" && self.is_require_call(node, "")
    }

    fn is_inheritance(&self, _node: &Node) -> bool {
        false // Lua doesn't have traditional inheritance
    }

    fn is_assignment(&self, node: &Node) -> bool {
        matches!(
            node.kind(),
            "assignment_statement" | "local_variable_declaration"
        )
    }

    fn extract_function_calls(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(function_node) = node.child_by_field_name("name") {
            let function_name = self.get_node_text(&function_node, source);
            let current_scope = context.current_scope();

            // Check if this is a require call
            if function_name == "require"
                || function_name == "dofile"
                || function_name == "loadfile"
            {
                self.extract_require_dependencies(node, source, context, dependencies);
                return;
            }

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

            if self.is_lua_keyword(&var_name) || var_name.trim().is_empty() {
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
        self.extract_require_dependencies(node, source, context, dependencies);
    }

    fn extract_inheritance(
        &self,
        _node: Node,
        _source: &str,
        _context: &mut ExtractionContext,
        _dependencies: &mut Vec<Dependency>,
    ) {
        // Lua doesn't have traditional inheritance
    }

    fn extract_assignments(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        match node.kind() {
            "assignment_statement" => {
                self.extract_assignment_dependencies(node, source, context, dependencies);
            }
            "local_variable_declaration" => {
                self.extract_local_variable_dependencies(node, source, context, dependencies);
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
            "for_statement" | "while_statement" | "repeat_statement" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "return_statement" => {
                self.extract_return_dependencies(node, source, context, dependencies);
            }
            "break_statement" => {
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
            "for_statement" | "while_statement" | "repeat_statement"
        )
    }

    fn is_exception_handling(&self, _node: &Node) -> bool {
        false // Lua doesn't have traditional exception handling
    }

    fn is_switch_statement(&self, _node: &Node) -> bool {
        false // Lua doesn't have switch statements
    }

    fn is_return_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "return_statement")
    }

    fn is_break_continue(&self, node: &Node) -> bool {
        matches!(node.kind(), "break_statement")
    }
}

impl LuaDependencyExtractor {
    /// Extract property/table access (obj.field, obj[key])
    fn extract_property_access(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(table_node) = node.child_by_field_name("table") {
            let table_name = self.get_node_text(&table_node, source);
            let current_scope = context.current_scope();

            if !table_name.trim().is_empty() && !self.is_lua_keyword(&table_name) {
                let dependency = self.create_dependency(
                    current_scope,
                    table_name,
                    DependencyType::References,
                    &node,
                    context,
                );
                dependencies.push(dependency);
            }
        }
    }

    /// Check if a function call is a require statement
    fn is_require_call(&self, node: &Node, source: &str) -> bool {
        if let Some(name_node) = node.child_by_field_name("name") {
            let function_name = self.get_node_text(&name_node, source);
            matches!(function_name.as_str(), "require" | "dofile" | "loadfile")
        } else {
            false
        }
    }

    /// Extract require dependencies
    fn extract_require_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        if let Some(name_node) = node.child_by_field_name("name") {
            let function_name = self.get_node_text(&name_node, source);

            // Extract module name from arguments
            if let Some(args_node) = node.child_by_field_name("arguments") {
                let mut cursor = args_node.walk();
                for child in args_node.children(&mut cursor) {
                    if child.kind() == "string" {
                        let module_name = self
                            .get_node_text(&child, source)
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string();

                        if !module_name.trim().is_empty() {
                            let dependency_type = match function_name.as_str() {
                                "require" => DependencyType::Imports,
                                "dofile" | "loadfile" => DependencyType::ModuleDependency,
                                _ => DependencyType::Imports,
                            };

                            let dependency = self.create_dependency(
                                current_scope.clone(),
                                module_name,
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

    /// Check if a string is a Lua keyword
    fn is_lua_keyword(&self, name: &str) -> bool {
        matches!(
            name,
            // Lua keywords
            "and" | "break" | "do" | "else" | "elseif" | "end" | "false" |
            "for" | "function" | "goto" | "if" | "in" | "local" | "nil" |
            "not" | "or" | "repeat" | "return" | "then" | "true" | "until" | "while" |
            // Built-in functions and variables
            "_G" | "_VERSION" | "assert" | "collectgarbage" | "dofile" | "error" |
            "getmetatable" | "ipairs" | "load" | "loadfile" | "next" | "pairs" |
            "pcall" | "print" | "rawequal" | "rawget" | "rawlen" | "rawset" |
            "require" | "select" | "setmetatable" | "tonumber" | "tostring" |
            "type" | "unpack" | "xpcall" |
            // Standard library modules
            "coroutine" | "debug" | "io" | "math" | "os" | "package" | "string" | "table" | "utf8"
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
        // Lua assignments can have multiple variables and values
        if let Some(variables_node) = node.child_by_field_name("variable") {
            if let Some(values_node) = node.child_by_field_name("value") {
                // For simplicity, extract the first variable name
                let var_name = self.get_node_text(&variables_node, source);
                self.extract_expression_dependencies(
                    values_node,
                    source,
                    context,
                    dependencies,
                    &var_name,
                );
            }
        }
    }

    /// Extract local variable dependencies
    fn extract_local_variable_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(names_node) = node.child_by_field_name("name") {
            if let Some(values_node) = node.child_by_field_name("value") {
                let var_name = self.get_node_text(&names_node, source);
                self.extract_expression_dependencies(
                    values_node,
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
                if !self.is_lua_keyword(&referenced_var) && !referenced_var.trim().is_empty() {
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
            "function_call" => {
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
                if !self.is_lua_keyword(&var_name) && !var_name.trim().is_empty() {
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

                if !self.is_lua_keyword(&arg_name) && !arg_name.trim().is_empty() {
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
                | "local_function"
                | "local_variable_declaration"
                | "parameter_list" => {
                    if let Some(name_field) = parent.child_by_field_name("name") {
                        if name_field.id() == node.id() {
                            return false;
                        }
                    }
                }
                "assignment_statement" => {
                    if let Some(variable_field) = parent.child_by_field_name("variable") {
                        if variable_field.id() == node.id() {
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
                // for i = 1, 10 do or for k, v in pairs(table) do
                if let Some(clause_node) = node.child_by_field_name("clause") {
                    match clause_node.kind() {
                        "numeric_for_clause" => {
                            // for i = start, end, step
                            if let Some(name_node) = clause_node.child_by_field_name("name") {
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

                            if let Some(start_node) = clause_node.child_by_field_name("start") {
                                self.extract_condition_variables(
                                    start_node,
                                    source,
                                    context,
                                    dependencies,
                                    &current_scope,
                                );
                            }
                            if let Some(end_node) = clause_node.child_by_field_name("end") {
                                self.extract_condition_variables(
                                    end_node,
                                    source,
                                    context,
                                    dependencies,
                                    &current_scope,
                                );
                            }
                        }
                        "generic_for_clause" => {
                            // for k, v in pairs(table)
                            if let Some(names_node) = clause_node.child_by_field_name("name") {
                                let var_names = self.get_node_text(&names_node, source);
                                if !var_names.trim().is_empty() {
                                    let dependency = self.create_dependency(
                                        current_scope.clone(),
                                        var_names,
                                        DependencyType::LoopIteration,
                                        &names_node,
                                        context,
                                    );
                                    dependencies.push(dependency);
                                }
                            }

                            if let Some(expressions_node) =
                                clause_node.child_by_field_name("expression")
                            {
                                self.extract_condition_variables(
                                    expressions_node,
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
            "repeat_statement" => {
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

    /// Extract return dependencies
    fn extract_return_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        // Extract returned values
        if let Some(expressions_node) = node.child_by_field_name("expression") {
            self.extract_expression_dependencies(
                expressions_node,
                source,
                context,
                dependencies,
                &current_scope,
            );
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

    /// Extract break dependencies
    fn extract_break_continue_dependencies(
        &self,
        node: Node,
        _source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        let dependency = self.create_dependency(
            current_scope,
            "break".to_string(),
            DependencyType::BreakContinue,
            &node,
            context,
        );
        dependencies.push(dependency);
    }
}

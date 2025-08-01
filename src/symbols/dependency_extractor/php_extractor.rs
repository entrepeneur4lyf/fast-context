//! PHP-specific dependency extraction
//!
//! Extracts dependency relationships from PHP source code, including:
//! - Function calls and method invocations
//! - Variable references and property access
//! - Class inheritance and trait usage
//! - Include/require statements and namespace usage
//! - Interface implementation
//! - Exception handling (try/catch/finally)
//! - Control flow (if/else, loops, switch)
//! - Magic methods and constants

use super::{BaseDependencyExtractor, DependencyExtractor, ExtractionContext};
use crate::parsers::LanguageId;
use crate::symbols::{Dependency, DependencyType};
use tree_sitter::Node;

/// PHP-specific dependency extractor
pub struct PhpDependencyExtractor;

impl DependencyExtractor for PhpDependencyExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::PHP
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
            "function_call_expression" => {
                self.extract_function_calls(node, source, context, dependencies);
            }
            "member_call_expression" | "scoped_call_expression" => {
                self.extract_method_calls(node, source, context, dependencies);
            }
            "member_access_expression" | "scoped_property_access_expression" => {
                self.extract_property_access(node, source, context, dependencies);
            }
            "variable_name" => {
                self.extract_variable_references(node, source, context, dependencies);
            }
            "include_expression"
            | "include_once_expression"
            | "require_expression"
            | "require_once_expression" => {
                self.extract_includes(node, source, context, dependencies);
            }
            "namespace_use_declaration" => {
                self.extract_imports(node, source, context, dependencies);
            }
            "class_declaration" | "interface_declaration" | "trait_declaration" => {
                self.extract_class_inheritance(node, source, context, dependencies);
            }
            "assignment_expression" | "property_declaration" => {
                self.extract_assignments(node, source, context, dependencies);
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

    fn is_function_call(&self, node: &Node) -> bool {
        matches!(
            node.kind(),
            "function_call_expression" | "member_call_expression" | "scoped_call_expression"
        )
    }

    fn is_variable_reference(&self, node: &Node) -> bool {
        matches!(
            node.kind(),
            "variable_name" | "member_access_expression" | "scoped_property_access_expression"
        )
    }

    fn is_import_statement(&self, node: &Node) -> bool {
        matches!(
            node.kind(),
            "namespace_use_declaration"
                | "include_expression"
                | "include_once_expression"
                | "require_expression"
                | "require_once_expression"
        )
    }

    fn is_inheritance(&self, node: &Node) -> bool {
        matches!(node.kind(), "class_declaration" | "interface_declaration")
            && (node.child_by_field_name("base_clause").is_some()
                || node.child_by_field_name("implements_clause").is_some())
    }

    fn is_assignment(&self, node: &Node) -> bool {
        matches!(
            node.kind(),
            "assignment_expression" | "property_declaration"
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
        // Only extract variables that aren't part of declarations
        if self.is_reference_context(&node) {
            let var_name = self.get_node_text(&node, source);
            let current_scope = context.current_scope();

            // Skip superglobals and built-ins
            if self.is_php_builtin(&var_name) || var_name.trim().is_empty() {
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

        // use Namespace\Class; or use Namespace\Class as Alias;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "namespace_use_clause" {
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

                // Extract alias if present
                if let Some(alias_node) = child.child_by_field_name("alias") {
                    let alias_name = self.get_node_text(&alias_node, source);
                    if !alias_name.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            alias_name,
                            DependencyType::Imports,
                            &alias_node,
                            context,
                        );
                        dependencies.push(dependency);
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
            "if_statement" => {
                self.extract_conditional_dependencies(node, source, context, dependencies);
            }
            "for_statement" | "foreach_statement" | "while_statement" | "do_statement" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "try_statement" | "catch_clause" | "finally_clause" => {
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
        matches!(node.kind(), "if_statement")
    }

    fn is_loop_statement(&self, node: &Node) -> bool {
        matches!(
            node.kind(),
            "for_statement" | "foreach_statement" | "while_statement" | "do_statement"
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

impl PhpDependencyExtractor {
    /// Extract method calls ($obj->method() or Class::method())
    fn extract_method_calls(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        // Extract object/class reference
        if let Some(object_node) = node.child_by_field_name("object") {
            let object_name = self.get_node_text(&object_node, source);
            let current_scope = context.current_scope();

            if !object_name.trim().is_empty() && !self.is_php_builtin(&object_name) {
                let resolved_objects = context.find_symbols_global(&object_name);
                let target_object = if !resolved_objects.is_empty() {
                    resolved_objects[0].qualified_name()
                } else {
                    object_name
                };

                let dependency = self.create_dependency(
                    current_scope.clone(),
                    target_object,
                    DependencyType::References,
                    &object_node,
                    context,
                );

                dependencies.push(dependency);
            }
        }

        // Extract method name
        if let Some(name_node) = node.child_by_field_name("name") {
            let method_name = self.get_node_text(&name_node, source);
            let current_scope = context.current_scope();

            if !method_name.trim().is_empty() {
                let dependency = self.create_dependency(
                    current_scope,
                    method_name,
                    DependencyType::Calls,
                    &name_node,
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

    /// Extract property access ($obj->prop or Class::$prop)
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

            if !object_name.trim().is_empty() && !self.is_php_builtin(&object_name) {
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

    /// Extract include/require dependencies
    fn extract_includes(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        // include 'file.php', require_once 'path/to/file.php'
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "string" || child.kind() == "encapsed_string" {
                let file_path = self
                    .get_node_text(&child, source)
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();

                if !file_path.trim().is_empty() {
                    let dependency_type = match node.kind() {
                        "include_expression" | "include_once_expression" => DependencyType::Imports,
                        "require_expression" | "require_once_expression" => {
                            DependencyType::ModuleDependency
                        }
                        _ => DependencyType::Imports,
                    };

                    let dependency = self.create_dependency(
                        current_scope.clone(),
                        file_path,
                        dependency_type,
                        &child,
                        context,
                    );
                    dependencies.push(dependency);
                }
            }
        }
    }

    /// Extract class inheritance and trait usage
    fn extract_class_inheritance(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let class_name = self.get_node_text(&name_node, source);

            // Extract extends clause
            if let Some(base_clause_node) = node.child_by_field_name("base_clause") {
                if let Some(base_name_node) = base_clause_node.child_by_field_name("name") {
                    let parent_class = self.get_node_text(&base_name_node, source);
                    if !parent_class.trim().is_empty() {
                        let dependency = self.create_dependency(
                            class_name.clone(),
                            parent_class,
                            DependencyType::Inherits,
                            &base_name_node,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }
            }

            // Extract implements clause
            if let Some(implements_clause_node) = node.child_by_field_name("implements_clause") {
                let mut cursor = implements_clause_node.walk();
                for child in implements_clause_node.children(&mut cursor) {
                    if child.kind() == "name" || child.kind() == "qualified_name" {
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

            // Extract trait usage
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "trait_use_clause" {
                    let mut trait_cursor = child.walk();
                    for trait_child in child.children(&mut trait_cursor) {
                        if trait_child.kind() == "name" || trait_child.kind() == "qualified_name" {
                            let trait_name = self.get_node_text(&trait_child, source);
                            if !trait_name.trim().is_empty() {
                                let dependency = self.create_dependency(
                                    class_name.clone(),
                                    trait_name,
                                    DependencyType::Uses,
                                    &trait_child,
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

    /// Check if a string is a PHP built-in or superglobal
    fn is_php_builtin(&self, name: &str) -> bool {
        matches!(
            name,
            // Superglobals
            "$GLOBALS" | "$_SERVER" | "$_GET" | "$_POST" | "$_FILES" | "$_COOKIE" |
            "$_SESSION" | "$_REQUEST" | "$_ENV" |
            // Magic constants
            "__LINE__" | "__FILE__" | "__DIR__" | "__FUNCTION__" | "__CLASS__" |
            "__TRAIT__" | "__METHOD__" | "__NAMESPACE__" |
            // Keywords
            "abstract" | "and" | "array" | "as" | "break" | "callable" | "case" |
            "catch" | "class" | "clone" | "const" | "continue" | "declare" |
            "default" | "die" | "do" | "echo" | "else" | "elseif" | "empty" |
            "enddeclare" | "endfor" | "endforeach" | "endif" | "endswitch" |
            "endwhile" | "eval" | "exit" | "extends" | "final" | "finally" |
            "fn" | "for" | "foreach" | "function" | "global" | "goto" | "if" |
            "implements" | "include" | "include_once" | "instanceof" | "insteadof" |
            "interface" | "isset" | "list" | "namespace" | "new" | "or" | "print" |
            "private" | "protected" | "public" | "require" | "require_once" |
            "return" | "switch" | "throw" | "trait" | "try" | "unset" |
            "use" | "var" | "while" | "xor" | "yield" | "yield_from" |
            // Built-in types
            "bool" | "int" | "float" | "string" | "object" | "resource" |
            "null" | "false" | "true" | "self" | "parent"
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

    /// Extract property declaration dependencies
    fn extract_property_declaration_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "property_element" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    if let Some(value_node) = child.child_by_field_name("default_value") {
                        let var_name = self.get_node_text(&name_node, source);

                        // Extract dependencies from the default value expression
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
            "variable_name" => {
                let referenced_var = self.get_node_text(&node, source);
                if !self.is_php_builtin(&referenced_var) && !referenced_var.trim().is_empty() {
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
            "function_call_expression" | "member_call_expression" | "scoped_call_expression" => {
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
            "variable_name" => {
                let var_name = self.get_node_text(&node, source);
                if !self.is_php_builtin(&var_name) && !var_name.trim().is_empty() {
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
            if child.kind() == "variable_name" {
                let arg_name = self.get_node_text(&child, source);
                let current_scope = context.current_scope();

                if !self.is_php_builtin(&arg_name) && !arg_name.trim().is_empty() {
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
                // Skip variables in these declaration contexts
                "function_definition"
                | "method_declaration"
                | "class_declaration"
                | "property_declaration"
                | "parameter"
                | "namespace_use_declaration" => {
                    // Check if this variable is the name being declared
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
                // for ($i = 0; $i < 10; $i++)
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
            "foreach_statement" => {
                // foreach ($array as $key => $value)
                if let Some(value_node) = node.child_by_field_name("value") {
                    self.extract_condition_variables(
                        value_node,
                        source,
                        context,
                        dependencies,
                        &current_scope,
                    );
                }

                if let Some(key_node) = node.child_by_field_name("key") {
                    let key_name = self.get_node_text(&key_node, source);
                    if !key_name.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            key_name,
                            DependencyType::LoopIteration,
                            &key_node,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }

                if let Some(value_var_node) = node.child_by_field_name("value_variable") {
                    let value_name = self.get_node_text(&value_var_node, source);
                    if !value_name.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            value_name,
                            DependencyType::LoopIteration,
                            &value_var_node,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }
            }
            "while_statement" | "do_statement" => {
                // while ($condition) or do { } while ($condition)
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
                // catch (ExceptionType $e)
                if let Some(type_node) = node.child_by_field_name("type") {
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

                if let Some(name_node) = node.child_by_field_name("name") {
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

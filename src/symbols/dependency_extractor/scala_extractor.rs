//! Scala-specific dependency extraction
//!
//! Extracts dependency relationships from Scala source code, including:
//! - Function calls and method invocations
//! - Class and object definitions with inheritance
//! - Trait usage and mixin composition
//! - Case classes and pattern matching
//! - Import statements and package dependencies
//! - Higher-order functions and lambdas
//! - Control flow (if/else, for comprehensions, match expressions)
//! - Exception handling (try/catch/finally)
//! - Implicit parameters and conversions

use super::{BaseDependencyExtractor, DependencyExtractor, ExtractionContext};
use crate::parsers::LanguageId;
use crate::symbols::{Dependency, DependencyType};
use tree_sitter::Node;

/// Scala-specific dependency extractor
pub struct ScalaDependencyExtractor;

impl DependencyExtractor for ScalaDependencyExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Scala
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
            "field_expression" => {
                self.extract_property_access(node, source, context, dependencies);
            }
            "identifier" => {
                self.extract_variable_references(node, source, context, dependencies);
            }
            "import_declaration" => {
                self.extract_imports(node, source, context, dependencies);
            }
            "class_definition" | "object_definition" | "trait_definition" => {
                self.extract_class_inheritance(node, source, context, dependencies);
            }
            "case_class_definition" => {
                self.extract_case_class_dependencies(node, source, context, dependencies);
            }
            "assignment" | "val_definition" | "var_definition" => {
                self.extract_assignments(node, source, context, dependencies);
            }
            "lambda_expression" | "function_definition" => {
                self.extract_lambda_dependencies(node, source, context, dependencies);
            }
            // Control flow
            "if_expression" => {
                self.extract_conditional_dependencies(node, source, context, dependencies);
            }
            "for_expression" => {
                self.extract_for_comprehension_dependencies(node, source, context, dependencies);
            }
            "while_expression" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "match_expression" => {
                self.extract_match_dependencies(node, source, context, dependencies);
            }
            "try_expression" => {
                self.extract_exception_dependencies(node, source, context, dependencies);
            }
            "return_expression" => {
                self.extract_return_dependencies(node, source, context, dependencies);
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
        matches!(node.kind(), "import_declaration")
    }

    fn is_inheritance(&self, node: &Node) -> bool {
        matches!(
            node.kind(),
            "class_definition" | "object_definition" | "trait_definition"
        ) && (node.child_by_field_name("extends").is_some()
            || node.child_by_field_name("with").is_some())
    }

    fn is_assignment(&self, node: &Node) -> bool {
        matches!(
            node.kind(),
            "assignment" | "val_definition" | "var_definition"
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

            if self.is_scala_keyword(&var_name) || var_name.trim().is_empty() {
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

        // import scala.collection.mutable.{Map, Set}
        // import java.util._
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "import_selectors" || child.kind() == "stable_identifier" {
                let import_path = self.get_node_text(&child, source);
                if !import_path.trim().is_empty() {
                    let dependency_type = if import_path.contains('.') {
                        DependencyType::NamespaceUsage
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
            "assignment" => {
                self.extract_assignment_dependencies(node, source, context, dependencies);
            }
            "val_definition" | "var_definition" => {
                self.extract_val_var_dependencies(node, source, context, dependencies);
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
            "for_expression" => {
                self.extract_for_comprehension_dependencies(node, source, context, dependencies);
            }
            "while_expression" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "match_expression" => {
                self.extract_match_dependencies(node, source, context, dependencies);
            }
            "try_expression" => {
                self.extract_exception_dependencies(node, source, context, dependencies);
            }
            "return_expression" => {
                self.extract_return_dependencies(node, source, context, dependencies);
            }
            _ => {}
        }
    }

    fn is_conditional_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "if_expression")
    }

    fn is_loop_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "for_expression" | "while_expression")
    }

    fn is_exception_handling(&self, node: &Node) -> bool {
        matches!(node.kind(), "try_expression")
    }

    fn is_switch_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "match_expression")
    }

    fn is_return_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "return_expression")
    }

    fn is_break_continue(&self, _node: &Node) -> bool {
        false // Scala doesn't have break/continue
    }
}

impl ScalaDependencyExtractor {
    /// Extract property access (obj.field)
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

            if !object_name.trim().is_empty() && !self.is_scala_keyword(&object_name) {
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

    /// Extract class, object, and trait inheritance
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
            if let Some(extends_node) = node.child_by_field_name("extends") {
                let parent_class = self.get_node_text(&extends_node, source);
                if !parent_class.trim().is_empty() {
                    let dependency = self.create_dependency(
                        class_name.clone(),
                        parent_class,
                        DependencyType::Inherits,
                        &extends_node,
                        context,
                    );
                    dependencies.push(dependency);
                }
            }

            // Extract with clause (trait mixins)
            if let Some(with_node) = node.child_by_field_name("with") {
                let mut cursor = with_node.walk();
                for child in with_node.children(&mut cursor) {
                    if child.kind() == "type_identifier" || child.kind() == "stable_identifier" {
                        let trait_name = self.get_node_text(&child, source);
                        if !trait_name.trim().is_empty() {
                            let dependency = self.create_dependency(
                                class_name.clone(),
                                trait_name,
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
    }

    /// Extract case class dependencies
    fn extract_case_class_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        // Case classes automatically get companion objects and pattern matching support
        if let Some(name_node) = node.child_by_field_name("name") {
            let case_class_name = self.get_node_text(&name_node, source);
            let current_scope = context.current_scope();

            // Mark as case class pattern
            let dependency = self.create_dependency(
                current_scope,
                format!("{case_class_name}_companion"),
                DependencyType::Uses,
                &name_node,
                context,
            );
            dependencies.push(dependency);

            // Extract constructor parameters
            if let Some(params_node) = node.child_by_field_name("parameters") {
                self.extract_parameter_dependencies(
                    params_node,
                    source,
                    context,
                    dependencies,
                    &case_class_name,
                );
            }
        }

        // Also extract inheritance like regular classes
        self.extract_class_inheritance(node, source, context, dependencies);
    }

    /// Extract lambda and function dependencies
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

        // Mark as lambda/function dependency
        let dependency = self.create_dependency(
            current_scope,
            "lambda_expression".to_string(),
            DependencyType::ControlFlow,
            &node,
            context,
        );
        dependencies.push(dependency);
    }

    /// Extract for comprehension dependencies
    fn extract_for_comprehension_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        // for (x <- collection; y <- anotherCollection if condition) yield result
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "enumerator" => {
                    // Extract pattern and collection
                    if let Some(pattern_node) = child.child_by_field_name("pattern") {
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

                    if let Some(collection_node) = child.child_by_field_name("collection") {
                        self.extract_condition_variables(
                            collection_node,
                            source,
                            context,
                            dependencies,
                            &current_scope,
                        );
                    }
                }
                "guard" => {
                    // Extract guard conditions
                    self.extract_condition_variables(
                        child,
                        source,
                        context,
                        dependencies,
                        &current_scope,
                    );
                }
                _ => {}
            }
        }
    }

    /// Extract match expression dependencies (pattern matching)
    fn extract_match_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        // Extract matched expression
        if let Some(expr_node) = node.child_by_field_name("expression") {
            self.extract_condition_variables(
                expr_node,
                source,
                context,
                dependencies,
                &current_scope,
            );
        }

        // Extract case patterns
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "case_clause" {
                if let Some(pattern_node) = child.child_by_field_name("pattern") {
                    self.extract_pattern_dependencies(
                        pattern_node,
                        source,
                        context,
                        dependencies,
                        &current_scope,
                    );
                }

                if let Some(guard_node) = child.child_by_field_name("guard") {
                    self.extract_condition_variables(
                        guard_node,
                        source,
                        context,
                        dependencies,
                        &current_scope,
                    );
                }
            }
        }

        let dependency = self.create_dependency(
            current_scope,
            "match_expression".to_string(),
            DependencyType::SwitchCase,
            &node,
            context,
        );
        dependencies.push(dependency);
    }

    /// Check if a string is a Scala keyword
    fn is_scala_keyword(&self, name: &str) -> bool {
        matches!(
            name,
            // Scala keywords
            "abstract" | "case" | "catch" | "class" | "def" | "do" | "else" |
            "extends" | "false" | "final" | "finally" | "for" | "forSome" |
            "if" | "implicit" | "import" | "lazy" | "match" | "new" | "null" |
            "object" | "override" | "package" | "private" | "protected" |
            "return" | "sealed" | "super" | "this" | "throw" | "trait" |
            "try" | "true" | "type" | "val" | "var" | "while" | "with" | "yield" |
            // Built-in types
            "Any" | "AnyRef" | "AnyVal" | "Boolean" | "Byte" | "Char" | "Double" |
            "Float" | "Int" | "Long" | "Nothing" | "Null" | "Short" | "String" |
            "Unit" | "Array" | "List" | "Map" | "Set" | "Option" | "Some" | "None"
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

    /// Extract val/var definition dependencies
    fn extract_val_var_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(pattern_node) = node.child_by_field_name("pattern") {
            if let Some(value_node) = node.child_by_field_name("value") {
                let var_name = self.get_node_text(&pattern_node, source);
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
                if !self.is_scala_keyword(&referenced_var) && !referenced_var.trim().is_empty() {
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
                if !self.is_scala_keyword(&var_name) && !var_name.trim().is_empty() {
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

    /// Extract pattern dependencies (for pattern matching)
    fn extract_pattern_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
        scope: &str,
    ) {
        match node.kind() {
            "constructor_pattern" => {
                // Case class pattern matching
                if let Some(type_node) = node.child_by_field_name("type") {
                    let type_name = self.get_node_text(&type_node, source);
                    if !type_name.trim().is_empty() {
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
            "identifier" => {
                let var_name = self.get_node_text(&node, source);
                if !self.is_scala_keyword(&var_name) && !var_name.trim().is_empty() {
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
                    self.extract_pattern_dependencies(child, source, context, dependencies, scope);
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
            if child.kind() == "parameter" {
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
                    if !type_name.trim().is_empty() && !self.is_scala_keyword(&type_name) {
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

                if !self.is_scala_keyword(&arg_name) && !arg_name.trim().is_empty() {
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
                "function_definition"
                | "class_definition"
                | "object_definition"
                | "trait_definition"
                | "val_definition"
                | "var_definition"
                | "parameter" => {
                    if let Some(name_field) = parent.child_by_field_name("name") {
                        if name_field.id() == node.id() {
                            return false;
                        }
                    }
                    if let Some(pattern_field) = parent.child_by_field_name("pattern") {
                        if pattern_field.id() == node.id() {
                            return false;
                        }
                    }
                }
                "assignment" => {
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
                if let Some(pattern_node) = child.child_by_field_name("pattern") {
                    self.extract_pattern_dependencies(
                        pattern_node,
                        source,
                        context,
                        dependencies,
                        &current_scope,
                    );
                }
            }
        }

        let dependency = self.create_dependency(
            current_scope,
            "try_expression".to_string(),
            DependencyType::ExceptionHandling,
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
}

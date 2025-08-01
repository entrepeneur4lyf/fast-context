//! Objective-C-specific dependency extraction
//!
//! Extracts dependency relationships from Objective-C source code, including:
//! - Method calls and message sending
//! - Property access and instance variables
//! - Class inheritance and protocol conformance
//! - Import statements (#import, @import)
//! - Category and extension usage
//! - Exception handling (@try/@catch/@finally)
//! - Control flow (if/else, loops, switch)
//! - Memory management patterns

use super::{BaseDependencyExtractor, DependencyExtractor, ExtractionContext};
use crate::parsers::LanguageId;
use crate::symbols::{Dependency, DependencyType};
use tree_sitter::Node;

/// Objective-C-specific dependency extractor
pub struct ObjectiveCDependencyExtractor;

impl DependencyExtractor for ObjectiveCDependencyExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::ObjectiveC
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
            "message_expression" => {
                self.extract_function_calls(node, source, context, dependencies);
            }
            "property_access" | "field_access" => {
                self.extract_property_access(node, source, context, dependencies);
            }
            "identifier" => {
                self.extract_variable_references(node, source, context, dependencies);
            }
            "preproc_import" | "import_declaration" => {
                self.extract_imports(node, source, context, dependencies);
            }
            "class_interface" | "class_implementation" | "protocol_declaration" => {
                self.extract_class_inheritance(node, source, context, dependencies);
            }
            "assignment_expression" => {
                self.extract_assignments(node, source, context, dependencies);
            }
            // Control flow
            "if_statement" => {
                self.extract_conditional_dependencies(node, source, context, dependencies);
            }
            "for_statement" | "while_statement" | "do_statement" => {
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
        matches!(node.kind(), "message_expression")
    }

    fn is_variable_reference(&self, node: &Node) -> bool {
        matches!(
            node.kind(),
            "identifier" | "property_access" | "field_access"
        )
    }

    fn is_import_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "preproc_import" | "import_declaration")
    }

    fn is_inheritance(&self, node: &Node) -> bool {
        matches!(node.kind(), "class_interface" | "class_implementation")
            && node.child_by_field_name("superclass").is_some()
    }

    fn is_assignment(&self, node: &Node) -> bool {
        matches!(node.kind(), "assignment_expression")
    }

    fn extract_function_calls(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        // Extract receiver
        if let Some(receiver_node) = node.child_by_field_name("receiver") {
            let receiver_name = self.get_node_text(&receiver_node, source);
            let current_scope = context.current_scope();

            if !receiver_name.trim().is_empty() && !self.is_objc_keyword(&receiver_name) {
                let dependency = self.create_dependency(
                    current_scope.clone(),
                    receiver_name,
                    DependencyType::References,
                    &receiver_node,
                    context,
                );
                dependencies.push(dependency);
            }
        }

        // Extract selector (method name)
        if let Some(selector_node) = node.child_by_field_name("selector") {
            let selector_name = self.get_node_text(&selector_node, source);
            let current_scope = context.current_scope();

            if !selector_name.trim().is_empty() {
                let dependency = self.create_dependency(
                    current_scope,
                    selector_name,
                    DependencyType::Calls,
                    &selector_node,
                    context,
                );
                dependencies.push(dependency);
            }
        }

        // Extract arguments
        if let Some(args_node) = node.child_by_field_name("arguments") {
            self.extract_argument_references(args_node, source, context, dependencies);
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

            if self.is_objc_keyword(&var_name) || var_name.trim().is_empty() {
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

        // #import "Header.h" or @import Foundation;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "string_literal" || child.kind() == "system_lib_string" {
                let import_path = self
                    .get_node_text(&child, source)
                    .trim_matches('"')
                    .trim_matches('<')
                    .trim_matches('>')
                    .to_string();

                if !import_path.trim().is_empty() {
                    let dependency_type =
                        if import_path.contains('/') || import_path.ends_with(".h") {
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
        self.extract_class_inheritance(node, source, context, dependencies);
    }

    fn extract_assignments(
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
            "for_statement" | "while_statement" | "do_statement"
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

impl ObjectiveCDependencyExtractor {
    /// Extract property access (obj.property)
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

            if !object_name.trim().is_empty() && !self.is_objc_keyword(&object_name) {
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

    /// Extract class inheritance and protocol conformance
    fn extract_class_inheritance(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let class_name = self.get_node_text(&name_node, source);

            // Extract superclass
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

            // Extract protocol conformance
            if let Some(protocols_node) = node.child_by_field_name("protocols") {
                let mut cursor = protocols_node.walk();
                for child in protocols_node.children(&mut cursor) {
                    if child.kind() == "type_identifier" {
                        let protocol_name = self.get_node_text(&child, source);
                        if !protocol_name.trim().is_empty() {
                            let dependency = self.create_dependency(
                                class_name.clone(),
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
    }

    /// Check if a string is an Objective-C keyword
    fn is_objc_keyword(&self, name: &str) -> bool {
        matches!(
            name,
            // Objective-C keywords
            "@interface" | "@implementation" | "@protocol" | "@end" | "@class" |
            "@selector" | "@encode" | "@synchronized" | "@autoreleasepool" |
            "@try" | "@catch" | "@finally" | "@throw" | "@import" |
            "self" | "super" | "_cmd" | "nil" | "Nil" | "YES" | "NO" |
            // C keywords
            "auto" | "break" | "case" | "char" | "const" | "continue" | "default" |
            "do" | "double" | "else" | "enum" | "extern" | "float" | "for" |
            "goto" | "if" | "int" | "long" | "register" | "return" | "short" |
            "signed" | "sizeof" | "static" | "struct" | "switch" | "typedef" |
            "union" | "unsigned" | "void" | "volatile" | "while" |
            // Common Foundation types
            "NSString" | "NSArray" | "NSDictionary" | "NSObject" | "NSNumber" |
            "BOOL" | "NSInteger" | "NSUInteger" | "CGFloat" | "id"
        )
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

                if !self.is_objc_keyword(&arg_name) && !arg_name.trim().is_empty() {
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
                "method_declaration"
                | "class_interface"
                | "class_implementation"
                | "protocol_declaration"
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

    /// Extract expression dependencies
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
                if !self.is_objc_keyword(&referenced_var) && !referenced_var.trim().is_empty() {
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
            "message_expression" => {
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
            "if_block".to_string(),
            DependencyType::ConditionalExecution,
            &node,
            context,
        );
        dependencies.push(dependency);
    }

    /// Extract condition variables
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
                if !self.is_objc_keyword(&var_name) && !var_name.trim().is_empty() {
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

    /// Extract exception dependencies
    fn extract_exception_dependencies(
        &self,
        node: Node,
        _source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

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

//! Ruby-specific dependency extraction
//! 
//! Extracts dependency relationships from Ruby source code, including:
//! - Method calls and message sending
//! - Variable references and instance variables
//! - Class inheritance and module inclusion
//! - Require statements and gem dependencies
//! - Block and proc usage
//! - Exception handling (begin/rescue/ensure)
//! - Control flow (if/unless, loops, case)
//! - Metaprogramming patterns

use crate::parsers::LanguageId;
use crate::symbols::{Dependency, DependencyType};
use super::{DependencyExtractor, ExtractionContext, BaseDependencyExtractor};
use tree_sitter::Node;

/// Ruby-specific dependency extractor
pub struct RubyDependencyExtractor;

impl DependencyExtractor for RubyDependencyExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Ruby
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
            "call" => {
                if self.is_require_call(&node, source) {
                    self.extract_requires(node, source, context, dependencies);
                } else {
                    self.extract_function_calls(node, source, context, dependencies);
                }
            }
            "method_call" => {
                self.extract_method_calls(node, source, context, dependencies);
            }
            "identifier" | "instance_variable" | "class_variable" | "global_variable" => {
                self.extract_variable_references(node, source, context, dependencies);
            }
            "class" | "module" => {
                self.extract_class_inheritance(node, source, context, dependencies);
            }
            "assignment" | "operator_assignment" => {
                self.extract_assignments(node, source, context, dependencies);
            }
            "block" | "do_block" => {
                self.extract_block_dependencies(node, source, context, dependencies);
            }
            // Control flow
            "if" | "unless" => {
                self.extract_conditional_dependencies(node, source, context, dependencies);
            }
            "for" | "while" | "until" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "begin" | "rescue" | "ensure" => {
                self.extract_exception_dependencies(node, source, context, dependencies);
            }
            "case" => {
                self.extract_switch_dependencies(node, source, context, dependencies);
            }
            "return" => {
                self.extract_return_dependencies(node, source, context, dependencies);
            }
            "break" | "next" => {
                self.extract_break_continue_dependencies(node, source, context, dependencies);
            }
            _ => {}
        }
    }
    
    fn is_function_call(&self, node: &Node) -> bool {
        matches!(node.kind(), "call" | "method_call")
    }
    
    fn is_variable_reference(&self, node: &Node) -> bool {
        matches!(node.kind(), "identifier" | "instance_variable" | "class_variable" | "global_variable")
    }
    
    fn is_import_statement(&self, node: &Node) -> bool {
        node.kind() == "call" && self.is_require_call(node, "")
    }
    
    fn is_inheritance(&self, node: &Node) -> bool {
        (node.kind() == "class" && node.child_by_field_name("superclass").is_some()) ||
        (node.kind() == "module" && self.has_include_extend(node))
    }
    
    fn is_assignment(&self, node: &Node) -> bool {
        matches!(node.kind(), "assignment" | "operator_assignment")
    }
    
    fn extract_function_calls(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(method_node) = node.child_by_field_name("method") {
            let method_name = self.get_node_text(&method_node, source);
            let current_scope = context.current_scope();
            
            if !method_name.trim().is_empty() && !method_name.contains('\n') {
                // Try to resolve the method in known symbols
                let resolved_methods = context.find_symbols_global(&method_name);
                let target_method = if !resolved_methods.is_empty() {
                    resolved_methods[0].qualified_name()
                } else {
                    method_name
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
        // Only extract variables that aren't part of declarations
        if self.is_reference_context(&node) {
            let var_name = self.get_node_text(&node, source);
            let current_scope = context.current_scope();
            
            // Skip keywords and built-ins
            if self.is_ruby_keyword(&var_name) || var_name.trim().is_empty() {
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
        self.extract_requires(node, source, context, dependencies);
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
            "operator_assignment" => {
                self.extract_operator_assignment_dependencies(node, source, context, dependencies);
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
            "if" | "unless" => {
                self.extract_conditional_dependencies(node, source, context, dependencies);
            }
            "for" | "while" | "until" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "begin" | "rescue" | "ensure" => {
                self.extract_exception_dependencies(node, source, context, dependencies);
            }
            "case" => {
                self.extract_switch_dependencies(node, source, context, dependencies);
            }
            "return" => {
                self.extract_return_dependencies(node, source, context, dependencies);
            }
            "break" | "next" => {
                self.extract_break_continue_dependencies(node, source, context, dependencies);
            }
            _ => {}
        }
    }
    
    fn is_conditional_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "if" | "unless")
    }
    
    fn is_loop_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "for" | "while" | "until")
    }
    
    fn is_exception_handling(&self, node: &Node) -> bool {
        matches!(node.kind(), "begin" | "rescue" | "ensure")
    }
    
    fn is_switch_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "case")
    }
    
    fn is_return_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "return")
    }
    
    fn is_break_continue(&self, node: &Node) -> bool {
        matches!(node.kind(), "break" | "next")
    }
}

impl RubyDependencyExtractor {
    /// Extract method calls (obj.method)
    fn extract_method_calls(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        // Extract receiver (object)
        if let Some(receiver_node) = node.child_by_field_name("receiver") {
            let object_name = self.get_node_text(&receiver_node, source);
            let current_scope = context.current_scope();

            if !object_name.trim().is_empty() && !self.is_ruby_keyword(&object_name) {
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
                    &receiver_node,
                    context,
                );

                dependencies.push(dependency);
            }
        }

        // Extract method name
        if let Some(method_node) = node.child_by_field_name("method") {
            let method_name = self.get_node_text(&method_node, source);
            let current_scope = context.current_scope();

            if !method_name.trim().is_empty() {
                let dependency = self.create_dependency(
                    current_scope,
                    method_name,
                    DependencyType::Calls,
                    &method_node,
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

    /// Check if a call node is a require statement
    fn is_require_call(&self, node: &Node, source: &str) -> bool {
        if let Some(method_node) = node.child_by_field_name("method") {
            let method_name = self.get_node_text(&method_node, source);
            matches!(method_name.as_str(), "require" | "require_relative" | "load" | "gem")
        } else {
            false
        }
    }

    /// Extract require statements
    fn extract_requires(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        if let Some(method_node) = node.child_by_field_name("method") {
            let method_name = self.get_node_text(&method_node, source);

            // Extract required file/gem name
            if let Some(args_node) = node.child_by_field_name("arguments") {
                let mut cursor = args_node.walk();
                for child in args_node.children(&mut cursor) {
                    if child.kind() == "string" || child.kind() == "simple_symbol" {
                        let required_name = self.get_node_text(&child, source)
                            .trim_matches('"')
                            .trim_matches('\'')
                            .trim_matches(':')
                            .to_string();

                        if !required_name.trim().is_empty() {
                            let dependency_type = match method_name.as_str() {
                                "gem" => DependencyType::ModuleDependency,
                                "require" | "require_relative" | "load" => DependencyType::Imports,
                                _ => DependencyType::Imports,
                            };

                            let dependency = self.create_dependency(
                                current_scope.clone(),
                                required_name,
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

    /// Extract class inheritance and module inclusion
    fn extract_class_inheritance(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let class_name = self.get_node_text(&name_node, source);

            // Extract superclass (class Child < Parent)
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

            // Extract include/extend/prepend statements within the class/module
            self.extract_module_inclusions(node, source, context, dependencies, &class_name);
        }
    }

    /// Extract module inclusions (include, extend, prepend)
    fn extract_module_inclusions(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
        class_name: &str,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "call" {
                if let Some(method_node) = child.child_by_field_name("method") {
                    let method_name = self.get_node_text(&method_node, source);

                    if matches!(method_name.as_str(), "include" | "extend" | "prepend") {
                        if let Some(args_node) = child.child_by_field_name("arguments") {
                            let mut args_cursor = args_node.walk();
                            for arg in args_node.children(&mut args_cursor) {
                                if arg.kind() == "constant" || arg.kind() == "identifier" {
                                    let module_name = self.get_node_text(&arg, source);
                                    if !module_name.trim().is_empty() {
                                        let dependency_type = match method_name.as_str() {
                                            "include" | "prepend" => DependencyType::Uses,
                                            "extend" => DependencyType::Implements,
                                            _ => DependencyType::Uses,
                                        };

                                        let dependency = self.create_dependency(
                                            class_name.to_string(),
                                            module_name,
                                            dependency_type,
                                            &arg,
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
    }

    /// Check if a class/module has include/extend statements
    fn has_include_extend(&self, node: &Node) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "call" {
                if let Some(method_node) = child.child_by_field_name("method") {
                    let method_name = self.get_node_text(&method_node, "");
                    if matches!(method_name.as_str(), "include" | "extend" | "prepend") {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Extract block dependencies
    fn extract_block_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        // Extract block parameters
        if let Some(params_node) = node.child_by_field_name("parameters") {
            let mut cursor = params_node.walk();
            for child in params_node.children(&mut cursor) {
                if child.kind() == "identifier" {
                    let param_name = self.get_node_text(&child, source);
                    if !param_name.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            param_name,
                            DependencyType::LoopIteration,
                            &child,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }
            }
        }

        // Mark as block dependency
        let dependency = self.create_dependency(
            current_scope,
            "block_expression".to_string(),
            DependencyType::ControlFlow,
            &node,
            context,
        );
        dependencies.push(dependency);
    }

    /// Check if a string is a Ruby keyword
    fn is_ruby_keyword(&self, name: &str) -> bool {
        matches!(name,
            "alias" | "and" | "begin" | "break" | "case" | "class" | "def" | "defined?" |
            "do" | "else" | "elsif" | "end" | "ensure" | "false" | "for" | "if" |
            "in" | "module" | "next" | "nil" | "not" | "or" | "redo" | "rescue" |
            "retry" | "return" | "self" | "super" | "then" | "true" | "undef" |
            "unless" | "until" | "when" | "while" | "yield" | "__FILE__" | "__LINE__" |
            // Built-in classes and modules
            "Object" | "Class" | "Module" | "String" | "Array" | "Hash" | "Integer" |
            "Float" | "Symbol" | "Proc" | "Method" | "Kernel" | "Enumerable" |
            "Comparable" | "File" | "IO" | "Thread" | "Exception" | "StandardError"
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
                    right_node, source, context, dependencies, &var_name
                );
            }
        }
    }

    /// Extract operator assignment dependencies (+=, -=, etc.)
    fn extract_operator_assignment_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(left_node) = node.child_by_field_name("left") {
            if let Some(right_node) = node.child_by_field_name("right") {
                let var_name = self.get_node_text(&left_node, source);

                // Operator assignment also references the left side
                let dependency = self.create_dependency(
                    var_name.clone(),
                    var_name.clone(),
                    DependencyType::References,
                    &left_node,
                    context,
                );
                dependencies.push(dependency);

                // Extract dependencies from the right-hand side
                self.extract_expression_dependencies(
                    right_node, source, context, dependencies, &var_name
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
            format!("{}_block", node.kind()),
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
            "identifier" | "instance_variable" | "class_variable" | "global_variable" => {
                let referenced_var = self.get_node_text(&node, source);
                if !self.is_ruby_keyword(&referenced_var) && !referenced_var.trim().is_empty() {
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
            "call" | "method_call" => {
                // Method call in assignment creates both call and data flow dependencies
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
            "identifier" | "instance_variable" | "class_variable" | "global_variable" => {
                let var_name = self.get_node_text(&node, source);
                if !self.is_ruby_keyword(&var_name) && !var_name.trim().is_empty() {
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
            if matches!(child.kind(), "identifier" | "instance_variable" | "class_variable" | "global_variable") {
                let arg_name = self.get_node_text(&child, source);
                let current_scope = context.current_scope();

                if !self.is_ruby_keyword(&arg_name) && !arg_name.trim().is_empty() {
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
                "method" | "class" | "module" | "assignment" | "parameter" => {
                    // Check if this variable is the name being declared
                    if let Some(name_field) = parent.child_by_field_name("name") {
                        if name_field.id() == node.id() {
                            return false; // This is a declaration
                        }
                    }
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
            "for" => {
                // for var in collection
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
                    self.extract_condition_variables(value_node, source, context, dependencies, &current_scope);
                }
            }
            "while" | "until" => {
                // while condition or until condition
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
            "rescue" => {
                // rescue ExceptionType => var
                if let Some(exceptions_node) = node.child_by_field_name("exceptions") {
                    let mut cursor = exceptions_node.walk();
                    for child in exceptions_node.children(&mut cursor) {
                        if child.kind() == "constant" || child.kind() == "identifier" {
                            let exception_type = self.get_node_text(&child, source);
                            if !exception_type.trim().is_empty() {
                                let dependency = self.create_dependency(
                                    current_scope.clone(),
                                    exception_type,
                                    DependencyType::ExceptionHandling,
                                    &child,
                                    context,
                                );
                                dependencies.push(dependency);
                            }
                        }
                    }
                }

                if let Some(variable_node) = node.child_by_field_name("variable") {
                    let var_name = self.get_node_text(&variable_node, source);
                    if !var_name.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            var_name,
                            DependencyType::ExceptionHandling,
                            &variable_node,
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

    /// Extract switch dependencies (case statements)
    fn extract_switch_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        // Extract case expression
        if let Some(value_node) = node.child_by_field_name("value") {
            self.extract_condition_variables(value_node, source, context, dependencies, &current_scope);
        }

        let dependency = self.create_dependency(
            current_scope,
            "case_statement".to_string(),
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

    /// Extract break/continue dependencies (break/next in Ruby)
    fn extract_break_continue_dependencies(
        &self,
        node: Node,
        _source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();
        let flow_type = if node.kind() == "break" { "break" } else { "next" };

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

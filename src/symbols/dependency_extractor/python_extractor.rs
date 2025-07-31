//! Python-specific dependency extraction
//! 
//! Extracts dependency relationships from Python source code, including:
//! - Function calls and method invocations
//! - Variable references and assignments  
//! - Module imports and from statements
//! - Class inheritance and decorators
//! - Exception handling (try/except/finally)
//! - Control flow (if/elif/else, loops, comprehensions)

use crate::parsers::LanguageId;
use crate::symbols::{Dependency, DependencyType};
use super::{DependencyExtractor, ExtractionContext, BaseDependencyExtractor};
use tree_sitter::Node;

/// Python-specific dependency extractor
pub struct PythonDependencyExtractor;

impl DependencyExtractor for PythonDependencyExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Python
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
                self.extract_function_calls(node, source, context, dependencies);
            }
            "attribute" => {
                self.extract_attribute_access(node, source, context, dependencies);
            }
            "identifier" => {
                self.extract_variable_references(node, source, context, dependencies);
            }
            "import_statement" | "import_from_statement" => {
                self.extract_imports(node, source, context, dependencies);
            }
            "class_definition" => {
                self.extract_class_inheritance(node, source, context, dependencies);
            }
            "assignment" | "augmented_assignment" => {
                self.extract_assignments(node, source, context, dependencies);
            }
            "decorator" => {
                self.extract_decorator_dependencies(node, source, context, dependencies);
            }
            // Control flow
            "if_statement" | "elif_clause" | "else_clause" => {
                self.extract_conditional_dependencies(node, source, context, dependencies);
            }
            "for_statement" | "while_statement" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "try_statement" | "except_clause" | "finally_clause" => {
                self.extract_exception_dependencies(node, source, context, dependencies);
            }
            "return_statement" => {
                self.extract_return_dependencies(node, source, context, dependencies);
            }
            "break_statement" | "continue_statement" => {
                self.extract_break_continue_dependencies(node, source, context, dependencies);
            }
            "with_statement" => {
                self.extract_context_manager_dependencies(node, source, context, dependencies);
            }
            _ => {}
        }
    }
    
    fn is_function_call(&self, node: &Node) -> bool {
        matches!(node.kind(), "call")
    }
    
    fn is_variable_reference(&self, node: &Node) -> bool {
        matches!(node.kind(), "identifier" | "attribute")
    }
    
    fn is_import_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "import_statement" | "import_from_statement")
    }
    
    fn is_inheritance(&self, node: &Node) -> bool {
        node.kind() == "class_definition" && node.child_by_field_name("superclasses").is_some()
    }
    
    fn is_assignment(&self, node: &Node) -> bool {
        matches!(node.kind(), "assignment" | "augmented_assignment")
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
            if self.is_python_keyword(&var_name) || var_name.trim().is_empty() {
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
            "import_statement" => {
                // import module1, module2
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "dotted_name" || child.kind() == "identifier" {
                        let module_name = self.get_node_text(&child, source);
                        if !module_name.trim().is_empty() {
                            let dependency = self.create_dependency(
                                current_scope.clone(),
                                module_name,
                                DependencyType::Imports,
                                &child,
                                context,
                            );
                            dependencies.push(dependency);
                        }
                    }
                }
            }
            "import_from_statement" => {
                // from module import name1, name2
                if let Some(module_node) = node.child_by_field_name("module_name") {
                    let module_name = self.get_node_text(&module_node, source);
                    if !module_name.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            module_name,
                            DependencyType::ModuleDependency,
                            &module_node,
                            context,
                        );
                        dependencies.push(dependency);
                    }
                }
                
                // Extract imported names
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "import_list" {
                        let mut import_cursor = child.walk();
                        for import_child in child.children(&mut import_cursor) {
                            if import_child.kind() == "identifier" || import_child.kind() == "aliased_import" {
                                let import_name = self.get_node_text(&import_child, source);
                                if !import_name.trim().is_empty() {
                                    let dependency = self.create_dependency(
                                        current_scope.clone(),
                                        import_name,
                                        DependencyType::Imports,
                                        &import_child,
                                        context,
                                    );
                                    dependencies.push(dependency);
                                }
                            }
                        }
                    }
                }
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
            "augmented_assignment" => {
                self.extract_augmented_assignment_dependencies(node, source, context, dependencies);
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
            "if_statement" | "elif_clause" | "else_clause" => {
                self.extract_conditional_dependencies(node, source, context, dependencies);
            }
            "for_statement" | "while_statement" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "try_statement" | "except_clause" | "finally_clause" => {
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
        matches!(node.kind(), "if_statement" | "elif_clause" | "else_clause")
    }
    
    fn is_loop_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "for_statement" | "while_statement")
    }
    
    fn is_exception_handling(&self, node: &Node) -> bool {
        matches!(node.kind(), "try_statement" | "except_clause" | "finally_clause")
    }
    
    fn is_switch_statement(&self, node: &Node) -> bool {
        // Python doesn't have switch statements (until match in 3.10)
        matches!(node.kind(), "match_statement")
    }
    
    fn is_return_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "return_statement")
    }
    
    fn is_break_continue(&self, node: &Node) -> bool {
        matches!(node.kind(), "break_statement" | "continue_statement")
    }
}

impl PythonDependencyExtractor {
    /// Extract attribute access (obj.attr)
    fn extract_attribute_access(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(object_node) = node.child_by_field_name("object") {
            let object_name = self.get_node_text(&object_node, source);
            let current_scope = context.current_scope();

            if !object_name.trim().is_empty() && !self.is_python_keyword(&object_name) {
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

            if let Some(superclasses_node) = node.child_by_field_name("superclasses") {
                let mut cursor = superclasses_node.walk();
                for child in superclasses_node.children(&mut cursor) {
                    if child.kind() == "identifier" || child.kind() == "attribute" {
                        let parent_class = self.get_node_text(&child, source);
                        if !parent_class.trim().is_empty() {
                            let dependency = self.create_dependency(
                                class_name.clone(),
                                parent_class,
                                DependencyType::Inherits,
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

    /// Extract decorator dependencies
    fn extract_decorator_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" || child.kind() == "attribute" || child.kind() == "call" {
                let decorator_name = self.get_node_text(&child, source);
                if !decorator_name.trim().is_empty() {
                    let dependency = self.create_dependency(
                        current_scope.clone(),
                        decorator_name,
                        DependencyType::Uses,
                        &child,
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
                    right_node, source, context, dependencies, &var_name
                );
            }
        }
    }

    /// Extract augmented assignment dependencies (+=, -=, etc.)
    fn extract_augmented_assignment_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(left_node) = node.child_by_field_name("left") {
            if let Some(right_node) = node.child_by_field_name("right") {
                let var_name = self.get_node_text(&left_node, source);

                // Augmented assignment also references the left side
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
                // for target in iter:
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
                    self.extract_condition_variables(right_node, source, context, dependencies, &current_scope);
                }
            }
            "while_statement" => {
                // while condition:
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
            "except_clause" => {
                // except ExceptionType as var:
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

    /// Extract break/continue dependencies
    fn extract_break_continue_dependencies(
        &self,
        node: Node,
        _source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();
        let flow_type = if node.kind() == "break_statement" { "break" } else { "continue" };

        let dependency = self.create_dependency(
            current_scope,
            flow_type.to_string(),
            DependencyType::BreakContinue,
            &node,
            context,
        );
        dependencies.push(dependency);
    }

    /// Extract context manager dependencies (with statement)
    fn extract_context_manager_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        // Extract context manager expression
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "with_item" {
                if let Some(value_node) = child.child_by_field_name("value") {
                    self.extract_condition_variables(value_node, source, context, dependencies, &current_scope);
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
            "identifier" => {
                let referenced_var = self.get_node_text(&node, source);
                if !self.is_python_keyword(&referenced_var) && !referenced_var.trim().is_empty() {
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
            "call" => {
                // Function call in assignment creates both call and data flow dependencies
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
            "identifier" => {
                let var_name = self.get_node_text(&node, source);
                if !self.is_python_keyword(&var_name) && !var_name.trim().is_empty() {
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

                if !self.is_python_keyword(&arg_name) && !arg_name.trim().is_empty() {
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
                "function_definition" | "class_definition" | "assignment" |
                "parameter" | "import_statement" | "import_from_statement" => {
                    // Check if this identifier is the name being declared
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

    /// Check if a string is a Python keyword
    fn is_python_keyword(&self, name: &str) -> bool {
        matches!(name,
            "False" | "None" | "True" | "and" | "as" | "assert" | "async" | "await" |
            "break" | "class" | "continue" | "def" | "del" | "elif" | "else" | "except" |
            "finally" | "for" | "from" | "global" | "if" | "import" | "in" | "is" |
            "lambda" | "nonlocal" | "not" | "or" | "pass" | "raise" | "return" | "try" |
            "while" | "with" | "yield" |
            // Built-in functions and types
            "abs" | "all" | "any" | "ascii" | "bin" | "bool" | "bytearray" | "bytes" |
            "callable" | "chr" | "classmethod" | "compile" | "complex" | "delattr" |
            "dict" | "dir" | "divmod" | "enumerate" | "eval" | "exec" | "filter" |
            "float" | "format" | "frozenset" | "getattr" | "globals" | "hasattr" |
            "hash" | "help" | "hex" | "id" | "input" | "int" | "isinstance" | "issubclass" |
            "iter" | "len" | "list" | "locals" | "map" | "max" | "memoryview" | "min" |
            "next" | "object" | "oct" | "open" | "ord" | "pow" | "print" | "property" |
            "range" | "repr" | "reversed" | "round" | "set" | "setattr" | "slice" |
            "sorted" | "staticmethod" | "str" | "sum" | "super" | "tuple" | "type" |
            "vars" | "zip" | "__import__"
        )
    }
}

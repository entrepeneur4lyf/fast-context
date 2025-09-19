//! Bash-specific dependency extraction
//!
//! Extracts dependency relationships from Bash shell scripts, including:
//! - Command invocations and function calls
//! - Variable references and expansions
//! - Source statements and script dependencies
//! - Control flow (if/else, for/while loops, case)
//! - Pipeline operations and command substitution
//! - Environment variable usage

use super::{BaseDependencyExtractor, DependencyExtractor, ExtractionContext};
use crate::parsers::LanguageId;
use crate::symbols::{Dependency, DependencyType};
use tree_sitter::Node;

/// Bash-specific dependency extractor
pub struct BashDependencyExtractor;

impl DependencyExtractor for BashDependencyExtractor {
    fn language(&self) -> LanguageId {
        LanguageId::Bash
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
            "command" => {
                self.extract_function_calls(node, source, context, dependencies);
            }
            "variable_name" | "simple_expansion" | "expansion" => {
                self.extract_variable_references(node, source, context, dependencies);
            }
            "source_command" => {
                self.extract_imports(node, source, context, dependencies);
            }
            "variable_assignment" => {
                self.extract_assignments(node, source, context, dependencies);
            }
            // Control flow
            "if_statement" => {
                self.extract_conditional_dependencies(node, source, context, dependencies);
            }
            "for_statement" | "while_statement" | "until_statement" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "case_statement" => {
                self.extract_switch_dependencies(node, source, context, dependencies);
            }
            "return_statement" => {
                self.extract_return_dependencies(node, source, context, dependencies);
            }
            "break_statement" | "continue_statement" => {
                self.extract_break_continue_dependencies(node, source, context, dependencies);
            }
            "pipeline" => {
                self.extract_pipeline_dependencies(node, source, context, dependencies);
            }
            "command_substitution" => {
                self.extract_command_substitution_dependencies(node, source, context, dependencies);
            }
            _ => {}
        }
    }

    fn is_function_call(&self, node: &Node) -> bool {
        matches!(node.kind(), "command")
    }

    fn is_variable_reference(&self, node: &Node) -> bool {
        matches!(
            node.kind(),
            "variable_name" | "simple_expansion" | "expansion"
        )
    }

    fn is_import_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "source_command")
    }

    fn is_inheritance(&self, _node: &Node) -> bool {
        false // Bash doesn't have inheritance
    }

    fn is_assignment(&self, node: &Node) -> bool {
        matches!(node.kind(), "variable_assignment")
    }

    fn extract_function_calls(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let command_name = self.get_node_text(&name_node, source);
            let current_scope = context.current_scope();

            if !command_name.trim().is_empty() && !command_name.contains('\n') {
                // Skip common shell built-ins unless they're user-defined functions
                if !self.is_common_builtin(&command_name) {
                    let resolved_commands = context.find_symbols_global(&command_name);
                    let target_command = if !resolved_commands.is_empty() {
                        resolved_commands[0].qualified_name()
                    } else {
                        command_name
                    };

                    let dependency = self.create_dependency(
                        current_scope,
                        target_command,
                        DependencyType::Calls,
                        &node,
                        context,
                    );

                    dependencies.push(dependency);
                }

                // Extract arguments for variable references
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.kind() == "word" || child.kind() == "string" {
                        self.extract_argument_references(child, source, context, dependencies);
                    }
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
            let var_name = self
                .get_node_text(&node, source)
                .trim_start_matches('$')
                .trim_start_matches('{')
                .trim_end_matches('}')
                .to_string();

            let current_scope = context.current_scope();

            if self.is_bash_builtin_var(&var_name) || var_name.trim().is_empty() {
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

        // source script.sh or . script.sh
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "word" || child.kind() == "string" {
                let script_path = self
                    .get_node_text(&child, source)
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();

                if !script_path.trim().is_empty() && script_path != "source" && script_path != "." {
                    let dependency = self.create_dependency(
                        current_scope.clone(),
                        script_path,
                        DependencyType::Imports,
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
        _node: Node,
        _source: &str,
        _context: &mut ExtractionContext,
        _dependencies: &mut Vec<Dependency>,
    ) {
        // Bash doesn't have inheritance
    }

    fn extract_assignments(
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
            "for_statement" | "while_statement" | "until_statement" => {
                self.extract_loop_dependencies(node, source, context, dependencies);
            }
            "case_statement" => {
                self.extract_switch_dependencies(node, source, context, dependencies);
            }
            "return_statement" => {
                self.extract_return_dependencies(node, source, context, dependencies);
            }
            "break_statement" | "continue_statement" => {
                self.extract_break_continue_dependencies(node, source, context, dependencies);
            }
            "pipeline" => {
                self.extract_pipeline_dependencies(node, source, context, dependencies);
            }
            "command_substitution" => {
                self.extract_command_substitution_dependencies(node, source, context, dependencies);
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
            "for_statement" | "while_statement" | "until_statement"
        )
    }

    fn is_exception_handling(&self, _node: &Node) -> bool {
        false // Bash doesn't have traditional exception handling
    }

    fn is_switch_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "case_statement")
    }

    fn is_return_statement(&self, node: &Node) -> bool {
        matches!(node.kind(), "return_statement")
    }

    fn is_break_continue(&self, node: &Node) -> bool {
        matches!(node.kind(), "break_statement" | "continue_statement")
    }
}

impl BashDependencyExtractor {
    /// Extract pipeline dependencies (cmd1 | cmd2 | cmd3)
    fn extract_pipeline_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        // Extract each command in the pipeline
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "command" {
                self.extract_function_calls(child, source, context, dependencies);
            }
        }

        // Mark as pipeline dependency
        let dependency = self.create_dependency(
            current_scope,
            "pipeline".to_string(),
            DependencyType::ControlFlow,
            &node,
            context,
        );
        dependencies.push(dependency);
    }

    /// Extract command substitution dependencies ($(command) or `command`)
    fn extract_command_substitution_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        // Extract the substituted command
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "command" {
                self.extract_function_calls(child, source, context, dependencies);
            }
        }

        // Mark as command substitution dependency
        let dependency = self.create_dependency(
            current_scope,
            "command_substitution".to_string(),
            DependencyType::ControlFlow,
            &node,
            context,
        );
        dependencies.push(dependency);
    }

    /// Check if a command is a common shell builtin
    fn is_common_builtin(&self, command: &str) -> bool {
        matches!(
            command,
            // Common shell built-ins
            "echo"
                | "printf"
                | "read"
                | "cd"
                | "pwd"
                | "ls"
                | "cp"
                | "mv"
                | "rm"
                | "mkdir"
                | "rmdir"
                | "touch"
                | "cat"
                | "grep"
                | "sed"
                | "awk"
                | "sort"
                | "uniq"
                | "head"
                | "tail"
                | "wc"
                | "find"
                | "xargs"
                | "test"
                | "["
                | "[["
                | "true"
                | "false"
                | "exit"
                | "return"
                | "break"
                | "continue"
                | "shift"
                | "export"
                | "unset"
                | "declare"
                | "local"
                | "readonly"
                | "typeset"
                | "alias"
                | "unalias"
                | "history"
                | "fc"
                | "jobs"
                | "bg"
                | "fg"
                | "wait"
                | "kill"
                | "trap"
                | "exec"
                | "eval"
                | "source"
                | "."
                | "type"
                | "which"
                | "command"
                | "builtin"
                | "enable"
                | "help"
                | "set"
                | "shopt"
                | "ulimit"
                | "umask"
                | "getopts"
                | "let"
                | "expr"
                | "basename"
                | "dirname"
        )
    }

    /// Check if a variable is a bash built-in variable
    fn is_bash_builtin_var(&self, var_name: &str) -> bool {
        matches!(
            var_name,
            // Special parameters
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" |
            "*" | "@" | "#" | "?" | "-" | "$" | "!" | "_" |
            // Built-in variables
            "BASH" | "BASH_VERSION" | "BASH_VERSINFO" | "BASHPID" | "PPID" | "UID" |
            "EUID" | "GROUPS" | "HOSTNAME" | "HOSTTYPE" | "MACHTYPE" | "OSTYPE" |
            "SHELLOPTS" | "BASHOPTS" | "BASH_ALIASES" | "BASH_ARGC" | "BASH_ARGV" |
            "BASH_CMDS" | "BASH_COMMAND" | "BASH_ENV" | "BASH_EXECUTION_STRING" |
            "BASH_LINENO" | "BASH_REMATCH" | "BASH_SOURCE" | "BASH_SUBSHELL" |
            "COLUMNS" | "COMP_CWORD" | "COMP_LINE" | "COMP_POINT" | "COMP_TYPE" |
            "COMP_KEY" | "COMP_WORDBREAKS" | "COMP_WORDS" | "COMPREPLY" | "DIRSTACK" |
            "FUNCNAME" | "GLOBIGNORE" | "HISTCMD" | "HISTCONTROL" | "HISTFILE" |
            "HISTFILESIZE" | "HISTIGNORE" | "HISTSIZE" | "HISTTIMEFORMAT" | "HOME" |
            "IFS" | "IGNOREEOF" | "INPUTRC" | "LANG" | "LC_ALL" | "LC_COLLATE" |
            "LC_CTYPE" | "LC_MESSAGES" | "LC_NUMERIC" | "LINENO" | "LINES" |
            "MAIL" | "MAILCHECK" | "MAILPATH" | "OLDPWD" | "OPTARG" |
            "OPTERR" | "OPTIND" | "PATH" | "PIPESTATUS" | "POSIXLY_CORRECT" |
            "PS1" | "PS2" | "PS3" | "PS4" | "PWD" | "RANDOM" | "REPLY" | "SECONDS" |
            "SHELL" | "SHLVL" | "TIMEFORMAT" | "TMOUT" | "TMPDIR" | "USER"
        )
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
            "simple_expansion" | "expansion" => {
                let referenced_var = self
                    .get_node_text(&node, source)
                    .trim_start_matches('$')
                    .trim_start_matches('{')
                    .trim_end_matches('}')
                    .to_string();

                if !self.is_bash_builtin_var(&referenced_var) && !referenced_var.trim().is_empty() {
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
            "command" => {
                self.extract_function_calls(node, source, context, dependencies);
            }
            "command_substitution" => {
                self.extract_command_substitution_dependencies(node, source, context, dependencies);
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
            "simple_expansion" | "expansion" => {
                let var_name = self
                    .get_node_text(&node, source)
                    .trim_start_matches('$')
                    .trim_start_matches('{')
                    .trim_end_matches('}')
                    .to_string();

                if !self.is_bash_builtin_var(&var_name) && !var_name.trim().is_empty() {
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
            "command" => {
                self.extract_function_calls(node, source, context, dependencies);
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
        let text = self.get_node_text(&node, source);
        let current_scope = context.current_scope();

        // Look for variable expansions in arguments
        if text.contains('$') {
            // Simple heuristic: extract $VAR patterns
            let chars = text.chars().peekable();
            let mut var_start = None;
            let mut pos = 0;

            for ch in chars {
                if ch == '$' {
                    var_start = Some(pos + 1);
                } else if var_start.is_some() && (!ch.is_alphanumeric() && ch != '_') {
                    if let Some(start) = var_start {
                        let var_name = &text[start..pos];
                        if !var_name.is_empty() && !self.is_bash_builtin_var(var_name) {
                            let dependency = self.create_dependency(
                                current_scope.clone(),
                                var_name.to_string(),
                                DependencyType::References,
                                &node,
                                context,
                            );
                            dependencies.push(dependency);
                        }
                        var_start = None;
                    }
                }
                pos += ch.len_utf8();
            }

            // Handle variable at end of string
            if let Some(start) = var_start {
                let var_name = text.get(start..).unwrap_or("");
                if !var_name.is_empty() && !self.is_bash_builtin_var(var_name) {
                    let dependency = self.create_dependency(
                        current_scope,
                        var_name.to_string(),
                        DependencyType::References,
                        &node,
                        context,
                    );
                    dependencies.push(dependency);
                }
            }
        }
    }

    /// Check if a node is in a reference context
    fn is_reference_context(&self, node: &Node) -> bool {
        let mut current = *node;

        while let Some(parent) = current.parent() {
            match parent.kind() {
                "function_definition" | "variable_assignment" => {
                    if let Some(name_field) = parent.child_by_field_name("name") {
                        if name_field.id() == node.id() {
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
                // for var in list or for ((i=0; i<10; i++))
                if let Some(variable_node) = node.child_by_field_name("variable") {
                    let var_name = self.get_node_text(&variable_node, source);
                    if !var_name.trim().is_empty() {
                        let dependency = self.create_dependency(
                            current_scope.clone(),
                            var_name,
                            DependencyType::LoopIteration,
                            &variable_node,
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
            "while_statement" | "until_statement" => {
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

    /// Extract switch dependencies (case statements)
    fn extract_switch_dependencies(
        &self,
        node: Node,
        source: &str,
        context: &mut ExtractionContext,
        dependencies: &mut Vec<Dependency>,
    ) {
        let current_scope = context.current_scope();

        if let Some(word_node) = node.child_by_field_name("word") {
            self.extract_condition_variables(
                word_node,
                source,
                context,
                dependencies,
                &current_scope,
            );
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

        // Extract return value if present
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

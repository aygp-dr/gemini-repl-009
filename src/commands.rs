//! Slash command definitions and completion helper for the REPL
//!
//! Provides tab completion and hints when typing slash commands.

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hint, Hinter};
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::borrow::Cow;

/// A slash command definition with metadata
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SlashCommand {
    /// The command name (e.g., "/help")
    pub name: &'static str,
    /// Brief description shown in completion menu
    pub description: &'static str,
    /// Optional argument hint (e.g., "<name>", "[format]")
    pub args: Option<&'static str>,
    /// Category for grouping
    pub category: CommandCategory,
}

/// Command categories for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandCategory {
    General,
    Session,
    Memory,
    Queue,
    Context,
}

#[allow(dead_code)]
impl CommandCategory {
    pub fn label(&self) -> &'static str {
        match self {
            CommandCategory::General => "General",
            CommandCategory::Session => "Session",
            CommandCategory::Memory => "Memory",
            CommandCategory::Queue => "Queue",
            CommandCategory::Context => "Context",
        }
    }
}

/// All available slash commands
pub static COMMANDS: &[SlashCommand] = &[
    // General commands
    SlashCommand {
        name: "/help",
        description: "Show help message",
        args: None,
        category: CommandCategory::General,
    },
    SlashCommand {
        name: "/exit",
        description: "Exit the REPL",
        args: None,
        category: CommandCategory::General,
    },
    SlashCommand {
        name: "/quit",
        description: "Exit the REPL (alias)",
        args: None,
        category: CommandCategory::General,
    },
    SlashCommand {
        name: "/model",
        description: "Show current model",
        args: None,
        category: CommandCategory::General,
    },
    SlashCommand {
        name: "/clear",
        description: "Clear the screen",
        args: None,
        category: CommandCategory::General,
    },
    SlashCommand {
        name: "/tools",
        description: "List available tools",
        args: None,
        category: CommandCategory::General,
    },
    // Context commands
    SlashCommand {
        name: "/context",
        description: "Show conversation history",
        args: None,
        category: CommandCategory::Context,
    },
    SlashCommand {
        name: "/reset",
        description: "Clear conversation history",
        args: None,
        category: CommandCategory::Context,
    },
    SlashCommand {
        name: "/stats",
        description: "Show session statistics",
        args: None,
        category: CommandCategory::Context,
    },
    SlashCommand {
        name: "/tokens",
        description: "Show token usage",
        args: None,
        category: CommandCategory::Context,
    },
    SlashCommand {
        name: "/compact",
        description: "Compact conversation history",
        args: Some("[truncate|summarize]"),
        category: CommandCategory::Context,
    },
    // Session commands
    SlashCommand {
        name: "/save",
        description: "Save current session",
        args: Some("[name]"),
        category: CommandCategory::Session,
    },
    SlashCommand {
        name: "/load",
        description: "Load a saved session",
        args: Some("<name>"),
        category: CommandCategory::Session,
    },
    SlashCommand {
        name: "/continue",
        description: "Load most recent session",
        args: None,
        category: CommandCategory::Session,
    },
    SlashCommand {
        name: "/sessions",
        description: "List saved sessions",
        args: None,
        category: CommandCategory::Session,
    },
    SlashCommand {
        name: "/delete",
        description: "Delete a saved session",
        args: Some("<name>"),
        category: CommandCategory::Session,
    },
    SlashCommand {
        name: "/export",
        description: "Export session",
        args: Some("[json|markdown] [file]"),
        category: CommandCategory::Session,
    },
    // Memory commands
    SlashCommand {
        name: "/memory",
        description: "List remembered facts",
        args: None,
        category: CommandCategory::Memory,
    },
    SlashCommand {
        name: "/remember",
        description: "Remember a fact",
        args: Some("<key> <value>"),
        category: CommandCategory::Memory,
    },
    SlashCommand {
        name: "/forget",
        description: "Forget a fact",
        args: Some("<key>"),
        category: CommandCategory::Memory,
    },
    SlashCommand {
        name: "/memory-clear",
        description: "Clear all facts",
        args: None,
        category: CommandCategory::Memory,
    },
    // Queue commands
    SlashCommand {
        name: "/queue",
        description: "Show pending requests",
        args: None,
        category: CommandCategory::Queue,
    },
    SlashCommand {
        name: "/queue-submit",
        description: "Submit a queue request",
        args: Some("<prompt>"),
        category: CommandCategory::Queue,
    },
];

/// Helper that provides command completion and hints
#[derive(Default)]
pub struct SlashCommandHelper;

impl SlashCommandHelper {
    pub fn new() -> Self {
        Self
    }

    /// Get commands matching a prefix
    fn matching_commands(&self, prefix: &str) -> Vec<&'static SlashCommand> {
        if !prefix.starts_with('/') {
            return vec![];
        }

        let prefix_lower = prefix.to_lowercase();
        COMMANDS
            .iter()
            .filter(|cmd| cmd.name.to_lowercase().starts_with(&prefix_lower))
            .collect()
    }

    /// Format a command for display in completion menu
    #[allow(dead_code)]
    fn format_completion(cmd: &SlashCommand) -> String {
        match cmd.args {
            Some(args) => format!("{} {}", cmd.name, args),
            None => cmd.name.to_string(),
        }
    }
}

impl Completer for SlashCommandHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        // Only complete at the start of a line for slash commands
        let prefix = &line[..pos];

        // Check if we're typing a slash command
        if !prefix.starts_with('/') {
            return Ok((0, vec![]));
        }

        // Don't complete if there's a space (user is typing arguments)
        if prefix.contains(' ') {
            return Ok((0, vec![]));
        }

        let matches = self.matching_commands(prefix);
        let completions: Vec<Pair> = matches
            .iter()
            .map(|cmd| Pair {
                display: format!("{:<16} {}", cmd.name, cmd.description),
                replacement: cmd.name.to_string(),
            })
            .collect();

        // Return position 0 to replace from start of line
        Ok((0, completions))
    }
}

/// Hint shown inline after cursor
#[derive(Debug)]
pub struct CommandHint {
    text: String,
}

impl Hint for CommandHint {
    fn display(&self) -> &str {
        &self.text
    }

    fn completion(&self) -> Option<&str> {
        Some(&self.text)
    }
}

impl Hinter for SlashCommandHelper {
    type Hint = CommandHint;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        // Only hint at end of line
        if pos < line.len() {
            return None;
        }

        // Only hint for slash commands
        if !line.starts_with('/') {
            return None;
        }

        // Don't hint if there's already a space (typing arguments)
        if line.contains(' ') {
            return None;
        }

        let matches = self.matching_commands(line);

        // If exactly one match, show the rest of the command as hint
        if matches.len() == 1 {
            let cmd = matches[0];
            if cmd.name.len() > line.len() {
                let hint_text = &cmd.name[line.len()..];
                return Some(CommandHint {
                    text: hint_text.to_string(),
                });
            }
        }

        None
    }
}

impl Highlighter for SlashCommandHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        // Could add syntax highlighting for commands here
        Cow::Borrowed(line)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        // Dim the hint text
        Cow::Owned(format!("\x1b[90m{}\x1b[0m", hint))
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: bool) -> bool {
        false
    }
}

impl Validator for SlashCommandHelper {}

impl Helper for SlashCommandHelper {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matching_commands_empty() {
        let helper = SlashCommandHelper::new();
        let matches = helper.matching_commands("");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_matching_commands_slash_only() {
        let helper = SlashCommandHelper::new();
        let matches = helper.matching_commands("/");
        assert_eq!(matches.len(), COMMANDS.len());
    }

    #[test]
    fn test_matching_commands_prefix() {
        let helper = SlashCommandHelper::new();
        let matches = helper.matching_commands("/c");
        // Should match /clear, /context, /continue, /compact
        assert!(matches.iter().any(|c| c.name == "/clear"));
        assert!(matches.iter().any(|c| c.name == "/context"));
        assert!(matches.iter().any(|c| c.name == "/continue"));
        assert!(matches.iter().any(|c| c.name == "/compact"));
    }

    #[test]
    fn test_matching_commands_exact() {
        let helper = SlashCommandHelper::new();
        let matches = helper.matching_commands("/help");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "/help");
    }

    #[test]
    fn test_matching_commands_case_insensitive() {
        let helper = SlashCommandHelper::new();
        let matches = helper.matching_commands("/HELP");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "/help");
    }

    #[test]
    fn test_command_categories() {
        // Verify all commands have proper categories
        for cmd in COMMANDS {
            assert!(!cmd.name.is_empty());
            assert!(!cmd.description.is_empty());
            let _ = cmd.category.label(); // Should not panic
        }
    }
}

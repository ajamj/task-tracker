//! CLI argument definitions using clap.

use clap::{Parser, Subcommand};

/// tt - Git-friendly personal task tracking CLI
///
/// A local-first task tracker that stores tasks in TOML files and daily logs
/// in Markdown, all within a dedicated Git repository.
#[derive(Parser)]
#[command(name = "tt")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The subcommand to run
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new tt workspace
    Init,

    /// Add a new task
    Add {
        /// Task title
        #[arg(required = true)]
        title: String,

        /// Project to add the task to (default: workspace default)
        #[arg(long, short)]
        project: Option<String>,

        /// Due date (YYYY-MM-DD)
        #[arg(long, short)]
        due: Option<String>,

        /// Priority (P0, P1, P2, P3)
        #[arg(long, short, default_value = "P2")]
        priority: String,

        /// Tags (can be used multiple times)
        #[arg(long, short = 't')]
        tag: Vec<String>,

        /// Task notes
        #[arg(long, short)]
        notes: Option<String>,

        /// Time estimate (e.g., "2h", "1d")
        #[arg(long, short)]
        estimate: Option<String>,
    },

    /// List tasks
    Ls {
        /// Project to list tasks from (default: workspace default)
        #[arg(long, short)]
        project: Option<String>,

        /// Filter by status (todo, doing, done, blocked, canceled)
        #[arg(long, short)]
        status: Option<String>,

        /// Show all tasks including done and canceled
        #[arg(long, short)]
        all: bool,
    },

    /// Show task details
    Show {
        /// Task ID (e.g., tt-000001)
        id: String,

        /// Project the task belongs to (default: workspace default)
        #[arg(long, short)]
        project: Option<String>,
    },

    /// Start working on a task
    Start {
        /// Task ID (e.g., tt-000001)
        id: String,

        /// Project the task belongs to (default: workspace default)
        #[arg(long, short)]
        project: Option<String>,
    },

    /// Mark a task as done
    Done {
        /// Task ID (e.g., tt-000001)
        id: String,

        /// Project the task belongs to (default: workspace default)
        #[arg(long, short)]
        project: Option<String>,
    },

    /// Append to daily log
    Log {
        /// Log text to append
        #[arg(required = true)]
        text: String,

        /// Project to log to (default: workspace default)
        #[arg(long, short)]
        project: Option<String>,

        /// Open log in editor instead of appending text
        #[arg(long, short)]
        edit: bool,

        /// Specific date (YYYY-MM-DD), defaults to today
        #[arg(long, short)]
        date: Option<String>,
    },

    /// Generate weekly report
    Report {
        /// Week to report on (e.g., 2026-W13), defaults to current week
        #[arg(long, short)]
        week: Option<String>,

        /// Project to generate report for (default: workspace default)
        #[arg(long, short)]
        project: Option<String>,
    },

    /// Search tasks and logs
    Search {
        /// Search query
        #[arg(required = true)]
        query: String,

        /// Filter by project
        #[arg(long, short)]
        project: Option<String>,

        /// Filter by status (can be used multiple times)
        #[arg(long, short)]
        status: Vec<String>,

        /// Filter by tag (can be used multiple times)
        #[arg(long, short = 't')]
        tag: Vec<String>,

        /// Filter by date range (from, YYYY-MM-DD)
        #[arg(long, short)]
        from: Option<String>,

        /// Filter by date range (to, YYYY-MM-DD)
        #[arg(long, short)]
        to: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Maximum results
        #[arg(long, short, default_value = "20")]
        limit: usize,
    },
}

impl Cli {
    /// Parse CLI arguments.
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_init() {
        let cli = Cli::parse_from(["tt", "init"]);
        assert!(matches!(cli.command, Some(Commands::Init)));
    }

    #[test]
    fn test_cli_parse_add() {
        let cli = Cli::parse_from(["tt", "add", "Test task"]);
        if let Some(Commands::Add { title, .. }) = cli.command {
            assert_eq!(title, "Test task");
        } else {
            panic!("Expected Add command");
        }
    }

    #[test]
    fn test_cli_parse_add_with_options() {
        let cli = Cli::parse_from([
            "tt",
            "add",
            "Test task",
            "--project",
            "work",
            "--due",
            "2026-04-03",
            "--priority",
            "P1",
            "--tag",
            "rust",
            "--notes",
            "Some notes",
        ]);

        if let Some(Commands::Add {
            title,
            project,
            due,
            priority,
            tag,
            notes,
            ..
        }) = cli.command
        {
            assert_eq!(title, "Test task");
            assert_eq!(project, Some("work".to_string()));
            assert_eq!(due, Some("2026-04-03".to_string()));
            assert_eq!(priority, "P1");
            assert_eq!(tag, vec!["rust".to_string()]);
            assert_eq!(notes, Some("Some notes".to_string()));
        } else {
            panic!("Expected Add command");
        }
    }

    #[test]
    fn test_cli_parse_ls() {
        let cli = Cli::parse_from(["tt", "ls"]);
        assert!(matches!(cli.command, Some(Commands::Ls { .. })));
    }

    #[test]
    fn test_cli_parse_show() {
        let cli = Cli::parse_from(["tt", "show", "tt-000001"]);
        if let Some(Commands::Show { id, .. }) = cli.command {
            assert_eq!(id, "tt-000001");
        } else {
            panic!("Expected Show command");
        }
    }

    #[test]
    fn test_cli_parse_start() {
        let cli = Cli::parse_from(["tt", "start", "tt-000001"]);
        if let Some(Commands::Start { id, .. }) = cli.command {
            assert_eq!(id, "tt-000001");
        } else {
            panic!("Expected Start command");
        }
    }

    #[test]
    fn test_cli_parse_done() {
        let cli = Cli::parse_from(["tt", "done", "tt-000001"]);
        if let Some(Commands::Done { id, .. }) = cli.command {
            assert_eq!(id, "tt-000001");
        } else {
            panic!("Expected Done command");
        }
    }

    #[test]
    fn test_cli_parse_log() {
        let cli = Cli::parse_from(["tt", "log", "Worked on task"]);
        if let Some(Commands::Log { text, .. }) = cli.command {
            assert_eq!(text, "Worked on task");
        } else {
            panic!("Expected Log command");
        }
    }

    #[test]
    fn test_cli_parse_report() {
        let cli = Cli::parse_from(["tt", "report", "--week", "2026-W13"]);
        if let Some(Commands::Report { week, .. }) = cli.command {
            assert_eq!(week, Some("2026-W13".to_string()));
        } else {
            panic!("Expected Report command");
        }
    }
}

//! CLI command implementations.

use chrono::Local;

use crate::cli::args::Commands;
use crate::cli::format::{format_task, format_task_list, GitSuggestion};
use crate::error::{Result, TtError};
use crate::models::{NewTask, Priority, TaskStatus, WeekRange};
use crate::reports::WeeklyReport;
use crate::storage::{LogStorage, TaskStorage, Workspace};
// use crate::search::{SearchIndex, SearchFilters, query::format_results}; // Disabled

/// Execute a CLI command.
pub fn execute(command: Commands) -> Result<()> {
    match command {
        Commands::Init => cmd_init(),
        Commands::Add {
            title,
            project,
            due,
            priority,
            tag,
            notes,
            estimate,
        } => cmd_add(AddArgs {
            title,
            project,
            due,
            priority,
            tag,
            notes,
            estimate,
        }),
        Commands::Ls {
            project,
            status,
            all,
        } => cmd_ls(LsArgs {
            project,
            status,
            all,
        }),
        Commands::Show { id, project } => cmd_show(id, project),
        Commands::Start { id, project } => cmd_start(id, project),
        Commands::Done { id, project } => cmd_done(id, project),
        Commands::Log {
            text,
            project,
            edit,
            date,
        } => cmd_log(LogArgs {
            text,
            project,
            edit,
            date,
        }),
        Commands::Report { week, project } => cmd_report(week, project),
        Commands::Search {
            query: _,
            project: _,
            status: _,
            tag: _,
            from: _,
            to: _,
            json: _,
            limit: _,
        } => {
            // Search WIP - disabled temporarily
            println!("Search feature is under development.");
            println!("Use 'tt ls' with filters instead:");
            println!("  tt ls --status todo");
            println!("  tt ls --priority P1");
            Ok(())
        },
        Commands::Dashboard { .. } => Ok(()),
    }
}

/// Arguments for the add command.
pub struct AddArgs {
    pub title: String,
    pub project: Option<String>,
    pub due: Option<String>,
    pub priority: String,
    pub tag: Vec<String>,
    pub notes: Option<String>,
    pub estimate: Option<String>,
}

/// Arguments for the ls command.
pub struct LsArgs {
    pub project: Option<String>,
    pub status: Option<String>,
    pub all: bool,
}

/// Arguments for the log command.
pub struct LogArgs {
    pub text: String,
    pub project: Option<String>,
    pub edit: bool,
    pub date: Option<String>,
}

/// Arguments for the search command.
pub struct SearchArgs {
    pub query: String,
    pub project: Option<String>,
    pub status: Vec<String>,
    pub tag: Vec<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub json: bool,
    pub limit: usize,
}

/// Initialize a new workspace.
fn cmd_init() -> Result<()> {
    let cwd = std::env::current_dir().map_err(TtError::IoError)?;
    let _workspace = Workspace::init(cwd.clone())?;

    println!("Initialized tt workspace (version 1)");
    println!("- Created: tt.toml");
    println!("- Created: projects/work/project.toml");
    println!("- Created: projects/work/tasks/2026/03/");
    println!("- Created: projects/work/logs/2026/");
    println!("- Created: projects/work/reports/weekly/");

    let git_suggestion = GitSuggestion::for_init();
    git_suggestion.display();

    Ok(())
}

/// Add a new task.
fn cmd_add(args: AddArgs) -> Result<()> {
    let cwd = std::env::current_dir().map_err(TtError::IoError)?;
    let workspace = Workspace::load(cwd)?;

    // Get project
    let project_slug = args.project.as_deref().unwrap_or(workspace.default_project_slug());
    let project = workspace
        .get_project(project_slug)
        .ok_or_else(|| TtError::ProjectNotFound(project_slug.to_string()))?;

    // Create task storage
    let task_storage = TaskStorage::new(project.tasks_dir.clone());

    // Get next ID
    let next_id = task_storage.next_id()?;

    // Parse priority
    let priority = match args.priority.as_str() {
        "P0" => Priority::P0,
        "P1" => Priority::P1,
        "P2" => Priority::P2,
        "P3" => Priority::P3,
        _ => Priority::P2,
    };

    // Build task
    let mut builder = NewTask::builder(&args.title)
        .project(project_slug)
        .priority(priority);

    if let Some(due) = args.due {
        builder = builder.due(due);
    }

    for tag in &args.tag {
        builder = builder.tag(tag);
    }

    if let Some(notes) = &args.notes {
        builder = builder.notes(notes);
    }

    if let Some(estimate) = args.estimate {
        builder = builder.estimate(estimate);
    }

    let mut task = builder.build(next_id);

    // Set file path
    let _now = Local::now();
    task.file_path = task_storage
        .task_path(&task.id)
        .to_string_lossy()
        .to_string();

    // Create task
    task_storage.create(&task)?;

    println!("Created task: {}", task.id);
    println!("Project:    {}", project_slug);
    println!("Path:       {}", task.file_path);
    println!("Status:     {}", task.status_display());

    if let Some(ref due) = task.due {
        println!("Due:        {}", due);
    }

    if !task.tags.is_empty() {
        println!("Tags:       {}", task.tags.join(", "));
    }

    println!("Priority:   {}", task.priority_display().unwrap_or("P2"));

    // Print git suggestions
    let git_suggestion = GitSuggestion::for_task_add(&task, project_slug);
    git_suggestion.display();

    Ok(())
}

/// List tasks.
fn cmd_ls(args: LsArgs) -> Result<()> {
    let cwd = std::env::current_dir().map_err(TtError::IoError)?;
    let workspace = Workspace::load(cwd)?;

    // Get project
    let project_slug = args.project.as_deref().unwrap_or(workspace.default_project_slug());
    let project = workspace
        .get_project(project_slug)
        .ok_or_else(|| TtError::ProjectNotFound(project_slug.to_string()))?;

    // Create task storage
    let task_storage = TaskStorage::new(project.tasks_dir.clone());

    // Get tasks
    let all_tasks = task_storage.list()?;

    // Filter by status
    let filtered_tasks: Vec<_> = all_tasks
        .into_iter()
        .filter(|task| {
            if args.all {
                return true;
            }

            // By default, hide done and canceled
            !matches!(task.status, TaskStatus::Done | TaskStatus::Canceled)
        })
        .filter(|task| {
            if let Some(ref status_str) = args.status {
                let status = match status_str.to_lowercase().as_str() {
                    "todo" => TaskStatus::Todo,
                    "doing" => TaskStatus::Doing,
                    "done" => TaskStatus::Done,
                    "blocked" => TaskStatus::Blocked,
                    "canceled" => TaskStatus::Canceled,
                    _ => return true,
                };
                task.status == status
            } else {
                true
            }
        })
        .collect();

    println!("Project: {}", project_slug);
    println!();
    println!("{}", format_task_list(&filtered_tasks));

    Ok(())
}

/// Show task details.
fn cmd_show(id: String, project: Option<String>) -> Result<()> {
    let cwd = std::env::current_dir().map_err(TtError::IoError)?;
    let workspace = Workspace::load(cwd)?;

    // Get project
    let project_slug = project.as_deref().unwrap_or(workspace.default_project_slug());
    let project = workspace
        .get_project(project_slug)
        .ok_or_else(|| TtError::ProjectNotFound(project_slug.to_string()))?;

    // Create task storage
    let task_storage = TaskStorage::new(project.tasks_dir.clone());

    // Get task
    let task = task_storage.get(&id)?;

    println!("{}", format_task(&task));

    Ok(())
}

/// Start working on a task.
fn cmd_start(id: String, project: Option<String>) -> Result<()> {
    let cwd = std::env::current_dir().map_err(TtError::IoError)?;
    let workspace = Workspace::load(cwd)?;

    // Get project
    let project_slug = project.as_deref().unwrap_or(workspace.default_project_slug());
    let project = workspace
        .get_project(project_slug)
        .ok_or_else(|| TtError::ProjectNotFound(project_slug.to_string()))?;

    // Create task storage
    let task_storage = TaskStorage::new(project.tasks_dir.clone());

    // Get task
    let mut task = task_storage.get(&id)?;

    // Validate transition
    if !task.status.can_transition_to(TaskStatus::Doing) {
        return Err(TtError::InvalidStatusTransition {
            from: task.status.display().to_string(),
            to: "DOING".to_string(),
        });
    }

    // Update task
    task.status = TaskStatus::Doing;
    let today = Local::now().format("%Y-%m-%d").to_string();

    if task.started_at.is_none() {
        task.started_at = Some(today.clone());
    }

    task.updated_at = today;

    // Save task
    task_storage.update(&task)?;

    println!("Updated:  {}", task.id);
    println!("Status:   {}", task.status_display());
    println!("Started:  {}", task.started_at.as_ref().unwrap());
    println!("Path:     {}", task.file_path);

    // Print git suggestions
    let git_suggestion = GitSuggestion::for_task_start(&task, project_slug);
    git_suggestion.display();

    Ok(())
}

/// Mark a task as done.
fn cmd_done(id: String, project: Option<String>) -> Result<()> {
    let cwd = std::env::current_dir().map_err(TtError::IoError)?;
    let workspace = Workspace::load(cwd)?;

    // Get project
    let project_slug = project.as_deref().unwrap_or(workspace.default_project_slug());
    let project = workspace
        .get_project(project_slug)
        .ok_or_else(|| TtError::ProjectNotFound(project_slug.to_string()))?;

    // Create task storage
    let task_storage = TaskStorage::new(project.tasks_dir.clone());

    // Get task
    let mut task = task_storage.get(&id)?;

    // Validate transition
    if !task.status.can_transition_to(TaskStatus::Done) {
        return Err(TtError::InvalidStatusTransition {
            from: task.status.display().to_string(),
            to: "DONE".to_string(),
        });
    }

    // Update task
    task.status = TaskStatus::Done;
    let today = Local::now().format("%Y-%m-%d").to_string();
    task.done_at = Some(today.clone());
    task.updated_at = today;

    // Save task
    task_storage.update(&task)?;

    println!("Updated:  {}", task.id);
    println!("Status:   {}", task.status_display());
    println!("Done:     {}", task.done_at.as_ref().unwrap());
    println!("Path:     {}", task.file_path);

    // Print git suggestions
    let git_suggestion = GitSuggestion::for_task_done(&task, project_slug);
    git_suggestion.display();

    Ok(())
}

/// Append to daily log.
fn cmd_log(args: LogArgs) -> Result<()> {
    let cwd = std::env::current_dir().map_err(TtError::IoError)?;
    let workspace = Workspace::load(cwd)?;

    // Get project
    let project_slug = args.project.as_deref().unwrap_or(workspace.default_project_slug());
    let project = workspace
        .get_project(project_slug)
        .ok_or_else(|| TtError::ProjectNotFound(project_slug.to_string()))?;

    // Get date
    let date = args.date.unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());

    // Create log storage
    let log_storage = LogStorage::new(project.logs_dir.clone());

    // Append to log
    let log = log_storage.append(&date, project_slug, &args.text)?;

    println!("Updated log: {} ({})", date, project_slug);
    println!("Path:       {}", log.file_path.display());

    // Show detected task IDs
    if !log.task_ids.is_empty() {
        println!();
        println!("Detected task IDs in log entry:");
        for task_id in &log.task_ids {
            println!("  - {}", task_id);
        }
    }

    // Print git suggestions
    let git_suggestion = GitSuggestion::for_log(&date, project_slug, &log.file_path.to_string_lossy());
    git_suggestion.display();

    Ok(())
}

/// Generate weekly report.
fn cmd_report(week: Option<String>, project: Option<String>) -> Result<()> {
    let cwd = std::env::current_dir().map_err(TtError::IoError)?;
    let workspace = Workspace::load(cwd)?;

    // Get project
    let project_slug = project.as_deref().unwrap_or(workspace.default_project_slug());
    let project = workspace
        .get_project(project_slug)
        .ok_or_else(|| TtError::ProjectNotFound(project_slug.to_string()))?;

    // Parse week
    let week = if let Some(week_str) = week {
        WeekRange::from_iso_string(&week_str)
            .ok_or_else(|| TtError::InvalidWeekFormat(week_str))?
    } else {
        WeekRange::current()
    };

    // Create storages
    let task_storage = TaskStorage::new(project.tasks_dir.clone());
    let log_storage = LogStorage::new(project.logs_dir.clone());

    // Generate report
    let report = WeeklyReport::generate(&task_storage, &log_storage, project, &week)?;

    // Render report
    let report_content = report.render();

    // Write report to file
    let report_path = project.weekly_report_path(&week.iso_week);

    // Ensure parent directory exists
    if let Some(parent) = report_path.parent() {
        std::fs::create_dir_all(parent).map_err(TtError::IoError)?;
    }

    std::fs::write(&report_path, &report_content)
        .map_err(TtError::IoError)?;

    println!("Generated weekly report: {} ({})", week.iso_week, project_slug);
    println!("Range:      {} to {}", week.start, week.end);
    println!("Path:       {}", report_path.display());
    println!();
    println!("Included:");
    println!("- Done (by done_at): {}", report.done.len());
    println!("- In progress (current status): {}", report.in_progress.len());
    println!("- Blocked (current status): {}", report.blocked.len());
    println!("- Mentioned in logs: {}", report.mentioned.len());
    println!("- Missing tasks referenced in logs: {}", report.missing.len());
    println!("- Worklog highlights: {} day(s)", report.highlights.len());

    // Print git suggestions
    let git_suggestion = crate::cli::format::GitSuggestion::for_report(
        &week.iso_week,
        project_slug,
        &report_path.to_string_lossy(),
    );
    git_suggestion.display();

    Ok(())
}

// Search tasks and logs. (Disabled)
// fn cmd_search(args: SearchArgs) -> Result<()> {
//     use crate::search::{SearchIndex, SearchFilters, query::format_results};
//
//     let cwd = std::env::current_dir().map_err(|e| TtError::IoError(e))?;
//     let workspace = Workspace::load(cwd.clone())?;
//
//     // Index path
//     let index_path = cwd.join(".tt").join("index");
//
//     // Open or create index
//     let mut search_index = SearchIndex::new_or_open(&index_path)
//         .map_err(|e| TtError::IoError(std::io::Error::new(
//             std::io::ErrorKind::Other,
//             format!("Failed to open search index: {}", e)
//         )))?;
//
//     // Build filters
//     let filters = SearchFilters {
//         project: args.project,
//         status: if args.status.is_empty() { None } else { Some(args.status) },
//         tag: if args.tag.is_empty() { None } else { Some(args.tag) },
//         from: args.from,
//         to: args.to,
//     };
//
//     // Execute search
//     let results = search_index.search(&args.query, &filters, args.limit)
//         .map_err(|e| TtError::IoError(std::io::Error::new(
//             std::io::ErrorKind::Other,
//             format!("Search failed: {}", e)
//         )))?;
//
//     if results.is_empty() {
//         println!("No results found for '{}'", args.query);
//         return Ok(());
//     }
//
//     // Output results
//     let output = format_results(&results, args.json)
//         .map_err(|e| TtError::IoError(std::io::Error::new(
//             std::io::ErrorKind::Other,
//             format!("Failed to format results: {}", e)
//         )))?;
//
//     println!("{}", output);
//     println!("\nFound {} result(s)", results.len());
//
//     Ok(())
// }

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn setup_workspace() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        Workspace::init(temp_dir.path().to_path_buf()).unwrap();
        temp_dir
    }

    #[test]
    fn test_cmd_init() {
        let temp_dir = setup_workspace();

        assert!(temp_dir.path().join("tt.toml").exists());
        assert!(temp_dir.path().join("projects/work/project.toml").exists());
    }

    #[test]
    fn test_cmd_add_and_show() {
        let temp_dir = setup_workspace();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Add task
        let args = AddArgs {
            title: "Test task".to_string(),
            project: Some("work".to_string()),
            due: None,
            priority: "P2".to_string(),
            tag: vec![],
            notes: None,
            estimate: None,
        };
        cmd_add(args).unwrap();

        // Show task
        cmd_show("tt-000001".to_string(), Some("work".to_string())).unwrap();
    }
}

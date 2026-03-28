//! Workspace management - loading and discovering projects.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Result, StorageError, StorageResult, TtError};
use crate::models::{ProjectConfig, WorkspaceConfig};

/// A project in the workspace.
#[derive(Debug, Clone)]
pub struct Project {
    /// Project slug (filesystem name).
    pub slug: String,

    /// Project configuration.
    pub config: ProjectConfig,

    /// Path to project directory.
    pub path: PathBuf,

    /// Path to tasks directory.
    pub tasks_dir: PathBuf,

    /// Path to logs directory.
    pub logs_dir: PathBuf,

    /// Path to reports directory.
    pub reports_dir: PathBuf,
}

impl Project {
    /// Create a new project instance.
    pub fn new(slug: String, config: ProjectConfig, path: PathBuf) -> Self {
        let tasks_dir = path.join("tasks");
        let logs_dir = path.join("logs");
        let reports_dir = path.join("reports");

        Self {
            slug,
            config,
            path,
            tasks_dir,
            logs_dir,
            reports_dir,
        }
    }

    /// Get the path for a task file by year/month.
    pub fn task_path(&self, year: u32, month: u32, task_id: &str) -> PathBuf {
        self.tasks_dir.join(format!("{:04}", year)).join(format!("{:02}", month)).join(format!("{}.toml", task_id))
    }

    /// Get the path for a log file by date.
    pub fn log_path(&self, date: &str) -> PathBuf {
        // date format: YYYY-MM-DD
        if let Some(year) = date.get(0..4) {
            self.logs_dir.join(year).join(format!("{}.md", date))
        } else {
            self.logs_dir.join(format!("{}.md", date))
        }
    }

    /// Get the path for a weekly report.
    pub fn weekly_report_path(&self, week: &str) -> PathBuf {
        // week format: YYYY-Www
        self.reports_dir.join("weekly").join(format!("{}.md", week))
    }
}

/// The workspace root.
#[derive(Debug, Clone)]
pub struct Workspace {
    /// Root directory path.
    pub root: PathBuf,

    /// Workspace configuration.
    pub config: WorkspaceConfig,

    /// Loaded projects.
    pub projects: HashMap<String, Project>,
}

impl Workspace {
    /// Load workspace from a directory.
    ///
    /// Searches for tt.toml in the directory and loads all projects.
    pub fn load(root: PathBuf) -> Result<Self> {
        let config_path = root.join("tt.toml");

        if !config_path.exists() {
            return Err(TtError::WorkspaceNotFoundAtPath(
                root.display().to_string(),
            ));
        }

        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| TtError::IoError(e))?;

        let config = WorkspaceConfig::from_str(&config_content)
            .map_err(|e| TtError::TomlParseError(e))?;

        let projects_dir = root.join(
            config.storage.as_ref().map(|s| &s.projects_dir).unwrap_or(&"projects".to_string())
        );

        let projects = Self::discover_projects(&projects_dir)?;

        Ok(Self {
            root,
            config,
            projects,
        })
    }

    /// Discover all projects in the projects directory.
    fn discover_projects(projects_dir: &Path) -> StorageResult<HashMap<String, Project>> {
        let mut projects = HashMap::new();

        if !projects_dir.exists() {
            return Ok(projects);
        }

        let entries = fs::read_dir(projects_dir)
            .map_err(|e| StorageError::IoError(e))?;

        for entry in entries {
            let entry = entry.map_err(|e| StorageError::IoError(e))?;
            let project_path = entry.path();

            if !project_path.is_dir() {
                continue;
            }

            let config_path = project_path.join("project.toml");
            if !config_path.exists() {
                continue;
            }

            let config_content = fs::read_to_string(&config_path)
                .map_err(|e| StorageError::IoError(e))?;

            let config = ProjectConfig::from_str(&config_content)
                .map_err(|e| StorageError::TomlParseError(e))?;

            let slug = config.slug.clone();
            let project = Project::new(slug.clone(), config, project_path);

            projects.insert(slug, project);
        }

        Ok(projects)
    }

    /// Get a project by slug.
    pub fn get_project(&self, slug: &str) -> Option<&Project> {
        self.projects.get(slug)
    }

    /// Get the default project.
    pub fn get_default_project(&self) -> Result<&Project> {
        let default_slug = &self.config.workspace.default_project;
        self.projects.get(default_slug)
            .ok_or_else(|| TtError::ProjectNotFound(default_slug.clone()))
    }

    /// Get the default project slug.
    pub fn default_project_slug(&self) -> &str {
        &self.config.workspace.default_project
    }

    /// Initialize a new workspace at the given path.
    pub fn init(root: PathBuf) -> Result<Self> {
        // Create tt.toml
        let config_path = root.join("tt.toml");
        fs::write(&config_path, WorkspaceConfig::default_toml())
            .map_err(|e| TtError::IoError(e))?;

        // Create projects/work directory
        let projects_dir = root.join("projects");
        let work_dir = projects_dir.join("work");
        fs::create_dir_all(&work_dir)
            .map_err(|e| TtError::IoError(e))?;

        // Create project.toml
        let project_config_path = work_dir.join("project.toml");
        fs::write(&project_config_path, ProjectConfig::default_toml("Work", "work"))
            .map_err(|e| TtError::IoError(e))?;

        // Create tasks, logs, reports directories
        fs::create_dir_all(work_dir.join("tasks").join("2026").join("03"))
            .map_err(|e| TtError::IoError(e))?;
        fs::create_dir_all(work_dir.join("logs").join("2026"))
            .map_err(|e| TtError::IoError(e))?;
        fs::create_dir_all(work_dir.join("reports").join("weekly"))
            .map_err(|e| TtError::IoError(e))?;

        // Return workspace without calling load (avoid potential recursion)
        let config = WorkspaceConfig::default();
        let projects = HashMap::new();
        
        Ok(Self {
            root,
            config,
            projects,
        })
    }

    /// Check if workspace has any projects.
    pub fn has_projects(&self) -> bool {
        !self.projects.is_empty()
    }

    /// Get all project slugs.
    pub fn project_slugs(&self) -> Vec<&String> {
        self.projects.keys().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_workspace() -> (TempDir, Workspace) {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::init(temp_dir.path().to_path_buf()).unwrap();
        (temp_dir, workspace)
    }

    #[test]
    fn test_workspace_init_creates_structure() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::init(temp_dir.path().to_path_buf()).unwrap();

        assert!(temp_dir.path().join("tt.toml").exists());
        assert!(temp_dir.path().join("projects/work/project.toml").exists());
        assert!(temp_dir.path().join("projects/work/tasks").exists());
        assert!(temp_dir.path().join("projects/work/logs").exists());
        assert!(temp_dir.path().join("projects/work/reports").exists());
        assert_eq!(workspace.projects.len(), 1);
        assert!(workspace.get_project("work").is_some());
    }

    #[test]
    fn test_workspace_load_existing() {
        let (_temp_dir, workspace) = create_test_workspace();

        assert_eq!(workspace.config.workspace.default_project, "work");
        assert_eq!(workspace.config.workspace.week_starts_on, "monday");
        assert!(workspace.has_projects());
    }

    #[test]
    fn test_workspace_get_default_project() {
        let (_temp_dir, workspace) = create_test_workspace();

        let default_project = workspace.get_default_project().unwrap();
        assert_eq!(default_project.slug, "work");
        assert_eq!(default_project.config.name, "Work");
    }

    #[test]
    fn test_workspace_get_project() {
        let (_temp_dir, workspace) = create_test_workspace();

        let project = workspace.get_project("work");
        assert!(project.is_some());
        assert_eq!(project.unwrap().slug, "work");

        let nonexistent = workspace.get_project("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_workspace_load_missing_config() {
        let temp_dir = TempDir::new().unwrap();
        let result = Workspace::load(temp_dir.path().to_path_buf());

        assert!(result.is_err());
        match result {
            Err(TtError::WorkspaceNotFoundAtPath(path)) => {
                assert!(path.contains(temp_dir.path().to_str().unwrap()));
            }
            _ => panic!("Expected WorkspaceNotFoundAtPath error"),
        }
    }

    #[test]
    fn test_project_paths() {
        let (_temp_dir, workspace) = create_test_workspace();
        let project = workspace.get_default_project().unwrap();

        // Task path
        let task_path = project.task_path(2026, 3, "tt-000001");
        assert_eq!(
            task_path,
            project.path.join("tasks").join("2026").join("03").join("tt-000001.toml")
        );

        // Log path
        let log_path = project.log_path("2026-03-28");
        assert_eq!(
            log_path,
            project.path.join("logs").join("2026").join("2026-03-28.md")
        );

        // Weekly report path
        let report_path = project.weekly_report_path("2026-W13");
        assert_eq!(
            report_path,
            project.path.join("reports").join("weekly").join("2026-W13.md")
        );
    }

    #[test]
    fn test_workspace_project_slugs() {
        let (_temp_dir, workspace) = create_test_workspace();

        let slugs = workspace.project_slugs();
        assert_eq!(slugs.len(), 1);
        assert_eq!(slugs[0], &"work".to_string());
    }
}

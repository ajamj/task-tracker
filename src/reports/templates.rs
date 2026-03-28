//! Template loading system for reports and logs.
//!
//! This module provides template loading functionality with:
//! - Embedded default templates (compiled into binary)
//! - Filesystem override (custom templates in templates/ directory)
//! - Graceful fallback to embedded templates on errors

use include_dir::{include_dir, Dir};
use minijinja::{Environment, Error as JinjaError};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use crate::error::{Result, TtError};
use crate::models::config::WorkspaceConfig;

/// Embedded templates directory (compiled into binary).
static EMBEDDED_TEMPLATES: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/reports/templates");

/// Template type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateType {
    WeeklyReport,
    DailyLog,
}

impl TemplateType {
    /// Get the template file name.
    pub fn file_name(&self) -> &'static str {
        match self {
            TemplateType::WeeklyReport => "weekly_report.j2",
            TemplateType::DailyLog => "daily_log.j2",
        }
    }

    /// Get the embedded template content.
    pub fn embedded_content(&self) -> Option<String> {
        EMBEDDED_TEMPLATES
            .get_file(self.file_name())
            .map(|f| f.contents_utf8().unwrap().to_string())
    }
}

/// Get the default template path relative to workspace root.
pub fn get_default_template_dir() -> &'static str {
    "templates"
}

/// Get the template path from config or use default.
pub fn get_template_path(config: &WorkspaceConfig, template_type: TemplateType) -> PathBuf {
    // Check if custom template path is configured
    let custom_path = match template_type {
        TemplateType::WeeklyReport => {
            config.reports.as_ref().and_then(|r| r.template_path.as_ref())
        }
        TemplateType::DailyLog => {
            config.reports.as_ref().and_then(|r| r.log_template_path.as_ref())
        }
    };

    if let Some(path) = custom_path {
        PathBuf::from(path)
    } else {
        // Default: templates/<template_name>
        PathBuf::from(get_default_template_dir()).join(template_type.file_name())
    }
}

/// Load a template from filesystem or fall back to embedded.
///
/// # Arguments
/// * `workspace_root` - Root directory of the workspace
/// * `config` - Workspace configuration
/// * `template_type` - Type of template to load
///
/// # Returns
/// * `Ok(String)` - Template content
/// * `Err(TtError)` - Error loading template (but embedded always available as fallback)
pub fn load_template(
    workspace_root: &Path,
    config: &WorkspaceConfig,
    template_type: TemplateType,
) -> Result<String> {
    let template_path = get_template_path(config, template_type);
    let full_path = workspace_root.join(&template_path);

    // Try to load from filesystem
    if full_path.exists() {
        match std::fs::read_to_string(&full_path) {
            Ok(content) => {
                tracing::debug!("Loaded custom template from: {:?}", full_path);
                return Ok(content);
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to read custom template {:?}: {}. Falling back to embedded.",
                    full_path,
                    e
                );
            }
        }
    }

    // Fall back to embedded template
    if let Some(content) = template_type.embedded_content() {
        tracing::debug!("Using embedded template for: {:?}", template_type);
        Ok(content)
    } else {
        Err(TtError::TemplateError(format!(
            "Embedded template not found: {:?}",
            template_type
        )))
    }
}

/// Render a template with the given context.
///
/// # Arguments
/// * `template_content` - Template string
/// * `ctx` - Context data (must be Serialize)
///
/// # Returns
/// * `Ok(String)` - Rendered output
/// * `Err(TtError)` - Template rendering error
pub fn render_template<T: serde::Serialize>(
    template_content: &str,
    ctx: &T,
) -> Result<String> {
    let mut env = Environment::new();
    
    // Add custom filter for joining arrays
    env.add_filter("join", |strings: Vec<String>, sep: String| {
        strings.join(&sep)
    });

    match env.render_str(template_content, ctx) {
        Ok(rendered) => Ok(rendered),
        Err(e) => Err(TtError::TemplateError(format!(
            "Template rendering error: {}",
            e
        ))),
    }
}

/// Load and render a template with graceful fallback.
///
/// This is the main entry point for template rendering.
/// It will:
/// 1. Try to load custom template from filesystem
/// 2. Fall back to embedded template if file not found
/// 3. Return error only if both fail
///
/// # Arguments
/// * `workspace_root` - Root directory of the workspace
/// * `config` - Workspace configuration
/// * `template_type` - Type of template to use
/// * `ctx` - Context data (must be Serialize)
///
/// # Returns
/// * `Ok(String)` - Rendered output
/// * `Err(TtError)` - Both custom and embedded templates failed
pub fn load_and_render_template<T: serde::Serialize>(
    workspace_root: &Path,
    config: &WorkspaceConfig,
    template_type: TemplateType,
    ctx: &T,
) -> Result<String> {
    let template_content = load_template(workspace_root, config, template_type)?;
    render_template(&template_content, ctx)
}

/// Validate a template string.
///
/// Returns Ok(()) if the template is valid, Err otherwise.
pub fn validate_template(template_content: &str) -> std::result::Result<(), JinjaError> {
    let env = Environment::new();
    env.render_str(template_content, &serde_json::Value::Null)
        .map(|_| ())
        .unwrap_or_else(|e| Err(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_config() -> WorkspaceConfig {
        WorkspaceConfig::default()
    }

    #[test]
    fn test_template_type_file_name() {
        assert_eq!(TemplateType::WeeklyReport.file_name(), "weekly_report.j2");
        assert_eq!(TemplateType::DailyLog.file_name(), "daily_log.j2");
    }

    #[test]
    fn test_embedded_template_exists() {
        // Check that embedded templates exist
        assert!(TemplateType::WeeklyReport.embedded_content().is_some());
        assert!(TemplateType::DailyLog.embedded_content().is_some());
    }

    #[test]
    fn test_load_template_fallback_to_embedded() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config();

        // No custom template exists, should use embedded
        let content = load_template(temp_dir.path(), &config, TemplateType::WeeklyReport).unwrap();
        
        assert!(content.contains("Weekly Report"));
        assert!(content.contains("{{ week.iso_week }}"));
    }

    #[test]
    fn test_load_template_uses_custom() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config();

        // Create custom template
        let templates_dir = temp_dir.path().join("templates");
        fs::create_dir_all(&templates_dir).unwrap();
        fs::write(
            templates_dir.join("weekly_report.j2"),
            "Custom Template: {{ project }}",
        )
        .unwrap();

        let content = load_template(temp_dir.path(), &config, TemplateType::WeeklyReport).unwrap();
        
        assert_eq!(content, "Custom Template: {{ project }}");
    }

    #[test]
    fn test_render_template_basic() {
        let template = "Hello, {{ name }}!";
        let ctx = serde_json::json!({"name": "World"});

        let result = render_template(template, &ctx).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_render_template_with_filter() {
        let template = "{% for item in items %}{{ item }}{% if not loop.last %}, {% endif %}{% endfor %}";
        let ctx = serde_json::json!({"items": vec!["a", "b", "c"]});

        let result = render_template(template, &ctx).unwrap();
        assert_eq!(result, "a, b, c");
    }

    #[test]
    fn test_validate_template_valid() {
        let template = "Hello, {{ name }}!";
        assert!(validate_template(template).is_ok());
    }

    #[test]
    fn test_validate_template_invalid() {
        let template = "Hello, {{ name }!"; // Missing closing brace
        assert!(validate_template(template).is_err());
    }
}

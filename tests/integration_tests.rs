//! Integration tests for CLI commands.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn setup_workspace() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Initialize workspace using the CLI
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("init").current_dir(&temp_dir);
    cmd.assert().success();
    
    temp_dir
}

#[test]
fn test_init_creates_workspace() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("init").current_dir(&temp_dir);
    
    let assert = cmd.assert();
    assert.success();
    
    // Verify files created
    assert!(temp_dir.path().join("tt.toml").exists());
    assert!(temp_dir.path().join("projects/work/project.toml").exists());
}

#[test]
fn test_add_creates_task() {
    let temp_dir = setup_workspace();
    
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("add")
        .arg("Test task")
        .arg("--project")
        .arg("work")
        .current_dir(&temp_dir);
    
    let assert = cmd.assert();
    assert.success();
    assert.stdout(predicate::str::contains("tt-000001"));
    assert.stdout(predicate::str::contains("Test task"));
    
    // Verify task file created
    let task_file = temp_dir.path()
        .join("projects/work/tasks")
        .join(format!("{:04}", chrono::Local::now().year()))
        .join(format!("{:02}", chrono::Local::now().month()))
        .join("tt-000001.toml");
    
    assert!(task_file.exists());
}

#[test]
fn test_add_task_with_options() {
    let temp_dir = setup_workspace();
    
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("add")
        .arg("Important task")
        .arg("--due")
        .arg("2026-04-03")
        .arg("--priority")
        .arg("P1")
        .arg("--tag")
        .arg("rust")
        .arg("--tag")
        .arg("cli")
        .current_dir(&temp_dir);
    
    let assert = cmd.assert();
    assert.success();
    assert.stdout(predicate::str::contains("tt-000001"));
    assert.stdout(predicate::str::contains("P1"));
}

#[test]
fn test_ls_shows_tasks() {
    let temp_dir = setup_workspace();
    
    // Add a task first
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("add").arg("Test task").current_dir(&temp_dir);
    cmd.assert().success();
    
    // List tasks
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("ls").current_dir(&temp_dir);
    
    let assert = cmd.assert();
    assert.success();
    assert.stdout(predicate::str::contains("tt-000001"));
    assert.stdout(predicate::str::contains("Test task"));
    assert.stdout(predicate::str::contains("TODO"));
}

#[test]
fn test_show_displays_task() {
    let temp_dir = setup_workspace();
    
    // Add a task
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("add").arg("Test task").current_dir(&temp_dir);
    cmd.assert().success();
    
    // Show task
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("show").arg("tt-000001").current_dir(&temp_dir);
    
    let assert = cmd.assert();
    assert.success();
    assert.stdout(predicate::str::contains("tt-000001"));
    assert.stdout(predicate::str::contains("Test task"));
    assert.stdout(predicate::str::contains("Status:"));
}

#[test]
fn test_start_transitions_task() {
    let temp_dir = setup_workspace();
    
    // Add a task
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("add").arg("Test task").current_dir(&temp_dir);
    cmd.assert().success();
    
    // Start task
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("start").arg("tt-000001").current_dir(&temp_dir);
    
    let assert = cmd.assert();
    assert.success();
    assert.stdout(predicate::str::contains("DOING"));
    
    // Verify status changed
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("show").arg("tt-000001").current_dir(&temp_dir);
    let assert = cmd.assert();
    assert.stdout(predicate::str::contains("DOING"));
}

#[test]
fn test_done_completes_task() {
    let temp_dir = setup_workspace();
    
    // Add a task
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("add").arg("Test task").current_dir(&temp_dir);
    cmd.assert().success();
    
    // Start task
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("start").arg("tt-000001").current_dir(&temp_dir);
    cmd.assert().success();
    
    // Complete task
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("done").arg("tt-000001").current_dir(&temp_dir);
    
    let assert = cmd.assert();
    assert.success();
    assert.stdout(predicate::str::contains("DONE"));
    
    // Verify status changed
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("show").arg("tt-000001").current_dir(&temp_dir);
    let assert = cmd.assert();
    assert.stdout(predicate::str::contains("DONE"));
}

#[test]
fn test_log_appends_to_daily_log() {
    let temp_dir = setup_workspace();
    
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("log")
        .arg("Worked on tt-000001")
        .current_dir(&temp_dir);
    
    let assert = cmd.assert();
    assert.success();
    assert.stdout(predicate::str::contains(&today));
    assert.stdout(predicate::str::contains("tt-000001"));
    
    // Verify log file created
    let log_file = temp_dir.path()
        .join("projects/work/logs")
        .join(format!("{:04}", chrono::Local::now().year()))
        .join(format!("{}.md", today));
    
    assert!(log_file.exists());
}

#[test]
fn test_log_detects_task_ids() {
    let temp_dir = setup_workspace();
    
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("log")
        .arg("Worked on tt-000001 and tt-000002")
        .current_dir(&temp_dir);
    
    let assert = cmd.assert();
    assert.success();
    assert.stdout(predicate::str::contains("tt-000001"));
    assert.stdout(predicate::str::contains("tt-000002"));
}

#[test]
fn test_full_workflow() {
    let temp_dir = setup_workspace();
    
    // 1. Add task
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("add").arg("Refactor config").current_dir(&temp_dir);
    cmd.assert().success();
    
    // 2. List tasks
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("ls").current_dir(&temp_dir);
    cmd.assert().success();
    
    // 3. Start task
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("start").arg("tt-000001").current_dir(&temp_dir);
    cmd.assert().success();
    
    // 4. Log work
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("log").arg("Worked on tt-000001").current_dir(&temp_dir);
    cmd.assert().success();
    
    // 5. Complete task
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("done").arg("tt-000001").current_dir(&temp_dir);
    cmd.assert().success();
    
    // 6. Generate report
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("report").arg("week").current_dir(&temp_dir);
    cmd.assert().success();
}

#[test]
fn test_invalid_status_transition() {
    let temp_dir = setup_workspace();
    
    // Add a task
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("add").arg("Test task").current_dir(&temp_dir);
    cmd.assert().success();
    
    // Try to complete without starting (should fail or auto-start)
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("done").arg("tt-000001").current_dir(&temp_dir);
    
    // This should error because todo -> done is invalid
    let assert = cmd.assert();
    assert.failure();
}

#[test]
fn test_ls_filter_by_status() {
    let temp_dir = setup_workspace();
    
    // Add two tasks
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("add").arg("Task 1").current_dir(&temp_dir);
    cmd.assert().success();
    
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("add").arg("Task 2").current_dir(&temp_dir);
    cmd.assert().success();
    
    // Start one task
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("start").arg("tt-000001").current_dir(&temp_dir);
    cmd.assert().success();
    
    // Filter by status
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("ls").arg("--status").arg("doing").current_dir(&temp_dir);
    
    let assert = cmd.assert();
    assert.success();
    assert.stdout(predicate::str::contains("tt-000001"));
    assert.stdout(predicate::str::contains("DOING"));
}

#[test]
fn test_help_output() {
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.arg("--help");
    
    let assert = cmd.assert();
    assert.success();
    assert.stdout(predicate::str::contains("init"));
    assert.stdout(predicate::str::contains("add"));
    assert.stdout(predicate::str::contains("ls"));
    assert.stdout(predicate::str::contains("show"));
    assert.stdout(predicate::str::contains("start"));
    assert.stdout(predicate::str::contains("done"));
    assert.stdout(predicate::str::contains("log"));
    assert.stdout(predicate::str::contains("report"));
}

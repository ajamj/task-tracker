//! Integration tests for tt CLI commands.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

/// Helper to create a test workspace
fn setup_workspace() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    
    // Initialize workspace
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.current_dir(&temp_dir);
    cmd.arg("init");
    cmd.assert().success();
    
    temp_dir
}

#[test]
fn test_init_creates_workspace() {
    let temp_dir = setup_workspace();
    
    // Check tt.toml exists
    assert!(temp_dir.path().join("tt.toml").exists());
    
    // Check projects directory exists
    assert!(temp_dir.path().join("projects").exists());
    
    // Check work project exists
    assert!(temp_dir.path().join("projects/work/project.toml").exists());
}

#[test]
fn test_add_creates_task() {
    let temp_dir = setup_workspace();
    
    // Add a task
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.current_dir(&temp_dir);
    cmd.arg("add").arg("Test task");
    cmd.assert().success().stdout(predicate::str::contains("Created task:"));
    
    // Check task file exists
    let tasks_dir = temp_dir.path().join("projects/work/tasks");
    let task_files = fs::read_dir(tasks_dir).unwrap();
    let mut found_task = false;
    for entry in task_files.flatten() {
        if entry.path().extension().map_or(false, |ext| ext == "toml") {
            found_task = true;
            let content = fs::read_to_string(entry.path()).unwrap();
            assert!(content.contains("Test task"));
        }
    }
    assert!(found_task, "Task file should be created");
}

#[test]
fn test_ls_lists_tasks() {
    let temp_dir = setup_workspace();
    
    // Add a task first
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.current_dir(&temp_dir);
    cmd.arg("add").arg("Task to list");
    cmd.assert().success();
    
    // List tasks
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.current_dir(&temp_dir);
    cmd.arg("ls");
    cmd.assert().success().stdout(predicate::str::contains("Task to list"));
}

#[test]
fn test_start_changes_status() {
    let temp_dir = setup_workspace();
    
    // Add a task
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.current_dir(&temp_dir);
    cmd.arg("add").arg("Task to start");
    cmd.assert().success();
    
    // Start the task
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.current_dir(&temp_dir);
    cmd.arg("start").arg("tt-000001");
    cmd.assert().success().stdout(predicate::str::contains("DOING"));
}

#[test]
fn test_done_changes_status() {
    let temp_dir = setup_workspace();
    
    // Add and start a task
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.current_dir(&temp_dir);
    cmd.arg("add").arg("Task to complete");
    cmd.assert().success();
    
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.current_dir(&temp_dir);
    cmd.arg("start").arg("tt-000001");
    cmd.assert().success();
    
    // Complete the task
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.current_dir(&temp_dir);
    cmd.arg("done").arg("tt-000001");
    cmd.assert().success().stdout(predicate::str::contains("DONE"));
}

#[test]
fn test_log_creates_entry() {
    let temp_dir = setup_workspace();
    
    // Add a log entry
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.current_dir(&temp_dir);
    cmd.arg("log").arg("Test log entry");
    cmd.assert().success().stdout(predicate::str::contains("Updated log:"));
    
    // Check log file exists
    let logs_dir = temp_dir.path().join("projects/work/logs");
    let log_files = fs::read_dir(logs_dir).unwrap();
    let mut found_log = false;
    for entry in log_files.flatten() {
        if entry.path().extension().map_or(false, |ext| ext == "md") {
            found_log = true;
            let content = fs::read_to_string(entry.path()).unwrap();
            assert!(content.contains("Test log entry"));
        }
    }
    assert!(found_log, "Log file should be created");
}

#[test]
fn test_show_displays_task() {
    let temp_dir = setup_workspace();
    
    // Add a task
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.current_dir(&temp_dir);
    cmd.arg("add").arg("Task to show");
    cmd.assert().success();
    
    // Show the task
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.current_dir(&temp_dir);
    cmd.arg("show").arg("tt-000001");
    cmd.assert().success()
        .stdout(predicate::str::contains("Task to show"))
        .stdout(predicate::str::contains("tt-000001"));
}

#[test]
fn test_add_with_priority() {
    let temp_dir = setup_workspace();
    
    // Add a task with priority
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.current_dir(&temp_dir);
    cmd.arg("add").arg("High priority task").arg("--priority").arg("P1");
    cmd.assert().success();
    
    // Show the task
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.current_dir(&temp_dir);
    cmd.arg("show").arg("tt-000001");
    cmd.assert().success().stdout(predicate::str::contains("P1"));
}

#[test]
fn test_add_with_tags() {
    let temp_dir = setup_workspace();
    
    // Add a task with tags
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.current_dir(&temp_dir);
    cmd.arg("add").arg("Tagged task")
        .arg("--tag").arg("feature")
        .arg("--tag").arg("urgent");
    cmd.assert().success();
    
    // Show the task
    let mut cmd = Command::cargo_bin("tt").unwrap();
    cmd.current_dir(&temp_dir);
    cmd.arg("show").arg("tt-000001");
    cmd.assert().success()
        .stdout(predicate::str::contains("feature"))
        .stdout(predicate::str::contains("urgent"));
}

//! Performance benchmarks for tt CLI using criterion.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::path::PathBuf;
use tempfile::TempDir;

// Import from tt crate
use tt::models::{WeekRange, WorkspaceConfig};
use tt::storage::{Workspace, TaskStorage, LogStorage};
use tt::models::{NewTask, Priority};

/// Create a test workspace with N tasks
fn create_workspace_with_tasks(num_tasks: usize) -> (TempDir, Workspace) {
    let temp_dir = TempDir::new().unwrap();
    let workspace = Workspace::init(temp_dir.path().to_path_buf()).unwrap();
    
    let project = workspace.default_project().unwrap();
    let task_storage = TaskStorage::new(&project).unwrap();
    
    for i in 0..num_tasks {
        let new_task = NewTask {
            title: format!("Test task {}", i),
            due: None,
            priority: Priority::P2,
            tags: vec!["benchmark".to_string()],
            notes: Some(format!("Notes for task {}", i)),
            estimate: None,
        };
        task_storage.create(new_task, &project.slug).unwrap();
    }
    
    (temp_dir, workspace)
}

/// Benchmark: ls command (list tasks)
fn bench_ls_command(c: &mut Criterion) {
    let mut group = c.benchmark_group("ls_command");
    
    for num_tasks in [10, 100, 1000].iter() {
        let (temp_dir, workspace) = create_workspace_with_tasks(*num_tasks);
        std::env::set_current_dir(temp_dir.path()).unwrap();
        
        group.bench_with_input(
            BenchmarkId::from_parameter(num_tasks),
            num_tasks,
            |b, _| {
                let project = workspace.default_project().unwrap();
                let task_storage = TaskStorage::new(&project).unwrap();
                b.iter(|| {
                    let tasks = task_storage.list().unwrap();
                    black_box(tasks);
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark: add task command
fn bench_add_task(c: &mut Criterion) {
    let (temp_dir, workspace) = create_workspace_with_tasks(10);
    std::env::set_current_dir(temp_dir.path()).unwrap();
    
    let project = workspace.default_project().unwrap();
    
    c.bench_function("add_task", |b| {
        b.iter(|| {
            let task_storage = TaskStorage::new(&project).unwrap();
            let new_task = NewTask {
                title: "Benchmark task".to_string(),
                due: None,
                priority: Priority::P2,
                tags: vec![],
                notes: None,
                estimate: None,
            };
            let result = task_storage.create(new_task, &project.slug);
            black_box(result);
        })
    });
}

/// Benchmark: search command
fn bench_search(c: &mut Criterion) {
    let (temp_dir, workspace) = create_workspace_with_tasks(100);
    std::env::set_current_dir(temp_dir.path()).unwrap();
    
    // Create search index
    let index_path = temp_dir.path().join(".tt").join("index");
    let mut search_index = tt::search::SearchIndex::new_or_open(&index_path).unwrap();
    
    // Index all tasks
    let project = workspace.default_project().unwrap();
    let task_storage = TaskStorage::new(&project).unwrap();
    let tasks = task_storage.list().unwrap();
    for task in &tasks {
        search_index.add_task(task, &project.slug).unwrap();
    }
    
    c.bench_function("search_query", |b| {
        let filters = tt::search::SearchFilters::new()
            .with_project("work");
        
        b.iter(|| {
            let results = search_index.search("task", &filters, 20).unwrap();
            black_box(results);
        })
    });
}

/// Benchmark: weekly report generation
fn bench_report_generation(c: &mut Criterion) {
    let (temp_dir, workspace) = create_workspace_with_tasks(50);
    std::env::set_current_dir(temp_dir.path()).unwrap();
    
    let project = workspace.default_project().unwrap();
    let task_storage = TaskStorage::new(&project).unwrap();
    let log_storage = LogStorage::new(&project).unwrap();
    
    // Create some logs
    for i in 0..7 {
        let date = chrono::Local::now().naive_local().date() - chrono::Duration::days(i);
        let log = log_storage.get_or_create(&date.to_string(), &project.slug).unwrap();
        log_storage.append(&log.date, format!("Worked on task {}", i), &project.slug).unwrap();
    }
    
    let week = WeekRange::from_date(chrono::Local::now().naive_local().date());
    
    c.bench_function("generate_weekly_report", |b| {
        b.iter(|| {
            let report = tt::reports::WeeklyReport::generate(
                &task_storage,
                &log_storage,
                &project,
                &week,
            ).unwrap();
            black_box(report);
        })
    });
}

/// Benchmark: workspace initialization
fn bench_init(c: &mut Criterion) {
    c.bench_function("workspace_init", |b| {
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            let workspace = Workspace::init(temp_dir.path().to_path_buf());
            black_box(workspace);
        })
    });
}

criterion_group!(
    benches,
    bench_ls_command,
    bench_add_task,
    bench_search,
    bench_report_generation,
    bench_init,
);

criterion_main!(benches);

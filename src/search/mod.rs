//! Search module for full-text search across tasks and logs.
//!
//! This module provides simple regex-based search capabilities.

pub mod simple;

pub use simple::{search_tasks, search_tasks_regex, SearchResult};

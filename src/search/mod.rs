//! Search module for full-text search across tasks and logs.
//!
//! This module provides full-text search capabilities using the tantivy search engine.
//! It indexes tasks and logs, enabling fast search with filters for project, status, tags, and dates.

pub mod index;
pub mod query;

pub use index::SearchIndex;
pub use query::{SearchFilters, SearchResult};

//! Search index management using tantivy.

use std::path::Path;
use tantivy::{
    doc,
    schema::{Schema, TEXT, STORED, DATE},
    Index, IndexWriter, Document, Term, Searcher, ReloadPolicy,
    query::{QueryParser, Occur},
    collector::TopDocs,
    DateTime,
};
use crate::models::{Task, TaskStatus};
use crate::storage::Log;

/// Fields in the search index schema
pub struct SearchSchema {
    pub id: tantivy::schema::Field,
    pub title: tantivy::schema::Field,
    pub content: tantivy::schema::Field,
    pub doc_type: tantivy::schema::Field,  // "task" or "log"
    pub project: tantivy::schema::Field,
    pub status: tantivy::schema::Field,
    pub tags: tantivy::schema::Field,
    pub date: tantivy::schema::Field,
}

impl SearchSchema {
    pub fn build() -> (Schema, Self) {
        let mut schema_builder = Schema::builder();
        
        let id = schema_builder.add_text_field("id", STORED);
        let title = schema_builder.add_text_field("title", TEXT | STORED);
        let content = schema_builder.add_text_field("content", TEXT);
        let doc_type = schema_builder.add_text_field("type", STORED);
        let project = schema_builder.add_text_field("project", STORED);
        let status = schema_builder.add_text_field("status", STORED);
        let tags = schema_builder.add_text_field("tags", STORED);
        let date = schema_builder.add_date_field("date", STORED);

        let schema = schema_builder.build();
        
        (schema, Self { id, title, content, doc_type, project, status, tags, date })
    }
}

/// Search index for tasks and logs
pub struct SearchIndex {
    index: Index,
    schema: SearchSchema,
    index_writer: IndexWriter,
    searcher: Searcher,
}

impl SearchIndex {
    /// Create or open a search index at the given path
    pub fn new_or_open(path: &Path) -> Result<Self, tantivy::Error> {
        std::fs::create_dir_all(path)?;
        
        let (schema, search_schema) = SearchSchema::build();
        
        let index = Index::open_in_dir(path)
            .or_else(|_| Index::create_in_dir(path, schema.clone()))?;
        
        let index_writer = index.writer(50_000_000)?; // 50MB buffer
        let reader = index.reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_build()?;
        let searcher = reader.searcher();

        Ok(Self {
            index,
            schema: search_schema,
            index_writer,
            searcher,
        })
    }

    /// Add a task to the index
    pub fn add_task(&mut self, task: &Task, project: &str) -> Result<(), tantivy::Error> {
        let created_at_date = task.created_at
            .parse::<chrono::NaiveDate>()
            .map(|d| DateTime::from_timestamp_secs(d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp()))
            .unwrap_or_else(|_| DateTime::from_timestamp_secs(0));

        let mut doc = doc!(
            self.schema.id => task.id.to_string(),
            self.schema.title => task.title.clone(),
            self.schema.content => task.notes.clone().unwrap_or_default(),
            self.schema.doc_type => "task",
            self.schema.project => project.to_string(),
            self.schema.status => task.status.to_string(),
            self.schema.tags => task.tags.join(","),
            self.schema.date => created_at_date,
        );

        self.index_writer.add_document(doc)?;
        self.index_writer.commit()?;
        Ok(())
    }

    /// Update a task in the index
    pub fn update_task(&mut self, task: &Task, project: &str) -> Result<(), tantivy::Error> {
        // Delete existing document with same ID
        self.delete_task(&task.id)?;
        // Add updated document
        self.add_task(task, project)
    }

    /// Delete a task from the index
    pub fn delete_task(&mut self, task_id: &str) -> Result<(), tantivy::Error> {
        self.index_writer.delete_term(Term::from_field_text(self.schema.id, task_id));
        self.index_writer.commit()?;
        Ok(())
    }

    /// Add a log entry to the index
    pub fn add_log(&mut self, log: &Log, project: &str) -> Result<(), tantivy::Error> {
        let log_date = chrono::NaiveDate::parse_from_str(&log.date, "%Y-%m-%d")
            .map(|d| DateTime::from_timestamp_secs(d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp()))
            .unwrap_or_else(|_| DateTime::from_timestamp_secs(0));

        let doc = doc!(
            self.schema.id => format!("log:{}", log.date),
            self.schema.title => format!("Log: {}", log.date),
            self.schema.content => log.content.clone(),
            self.schema.doc_type => "log",
            self.schema.project => project.to_string(),
            self.schema.date => log_date,
        );

        self.index_writer.add_document(doc)?;
        self.index_writer.commit()?;
        Ok(())
    }

    /// Search the index with filters
    pub fn search(&self, query_text: &str, filters: &super::query::SearchFilters, limit: usize) -> Result<Vec<SearchResult>, tantivy::Error> {
        let query_parser = QueryParser::for_index(
            &self.index,
            vec![self.schema.title, self.schema.content],
        );

        let base_query = query_parser.parse_query(query_text)?;

        // Build filter queries
        let mut queries: Vec<(Occur, Box<dyn tantivy::query::Query>)> = vec![(Occur::Must, Box::new(base_query))];

        if let Some(ref project) = filters.project {
            let project_query = tantivy::query::TermQuery::new(
                Term::from_field_text(self.schema.project, project),
                tantivy::schema::IndexRecordOption::Basic,
            );
            queries.push((Occur::Must, Box::new(project_query)));
        }

        if let Some(ref statuses) = filters.status {
            if !statuses.is_empty() {
                let status_queries: Vec<Box<dyn tantivy::query::Query>> = statuses
                    .iter()
                    .map(|s| {
                        Box::new(tantivy::query::TermQuery::new(
                            Term::from_field_text(self.schema.status, s),
                            tantivy::schema::IndexRecordOption::Basic,
                        )) as Box<dyn tantivy::query::Query>
                    })
                    .collect();
                
                let or_query = tantivy::query::BooleanQuery::union(status_queries);
                queries.push((Occur::Must, Box::new(or_query)));
            }
        }

        let query = tantivy::query::BooleanQuery::new(queries);

        let top_docs = self.searcher.search(&query, &TopDocs::with_limit(limit))?;

        let results: Vec<SearchResult> = top_docs
            .iter()
            .filter_map(|(_, doc_address)| {
                let doc = self.searcher.doc(*doc_address).ok()?;
                self.doc_to_search_result(doc)
            })
            .collect();

        Ok(results)
    }

    /// Convert a Document to SearchResult
    fn doc_to_search_result(&self, doc: Document) -> Option<SearchResult> {
        let id = doc.get_first(self.schema.id)?.as_text()?.to_string();
        let title = doc.get_first(self.schema.title)?.as_text()?.to_string();
        let doc_type = doc.get_first(self.schema.doc_type)?.as_text()?.to_string();
        let project = doc.get_first(self.schema.project)?.as_text()?.to_string();
        
        let status = doc.get_first(self.schema.status)
            .and_then(|f| f.as_text())
            .map(|s| s.to_string());
        
        let tags = doc.get_first(self.schema.tags)
            .and_then(|f| f.as_text())
            .map(|t| t.split(',').map(|s| s.to_string()).collect())
            .unwrap_or_default();

        Some(SearchResult {
            id,
            title,
            doc_type,
            project,
            status,
            tags,
            score: 0.0, // Score not easily accessible
        })
    }

    /// Rebuild the index from scratch
    pub fn rebuild(&mut self) -> Result<(), tantivy::Error> {
        // Drop current index and recreate
        let index_path = self.index.directory().get_filepath();
        drop(self.index_writer);
        drop(self.searcher);
        drop(&self.index);

        // Delete index directory
        if index_path.exists() {
            std::fs::remove_dir_all(&index_path)?;
        }

        // Recreate
        let new = Self::new_or_open(&index_path)?;
        self.index = new.index;
        self.schema = new.schema;
        self.index_writer = new.index_writer;
        self.searcher = new.searcher;

        Ok(())
    }
}

/// Search result item
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub doc_type: String,  // "task" or "log"
    pub project: String,
    pub status: Option<String>,
    pub tags: Vec<String>,
    pub score: f64,
}

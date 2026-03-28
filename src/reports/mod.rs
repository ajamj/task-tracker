//! Report generation for tt.

pub mod templates;
pub mod weekly;

pub use weekly::{WeeklyReport, HighlightDay, WeeklyReportContext, LogContext};
pub use templates::{TemplateType, load_template, render_template, load_and_render_template};

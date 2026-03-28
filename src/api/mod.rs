//! API module for tt dashboard.

pub mod handlers;
pub mod state;
pub mod routes;

pub use state::AppState;
pub use routes::create_router;

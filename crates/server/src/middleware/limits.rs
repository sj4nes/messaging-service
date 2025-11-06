use axum::extract::DefaultBodyLimit;

/// Create a body size limit layer using Axum's DefaultBodyLimit.
pub fn body_limit(max_bytes: usize) -> DefaultBodyLimit {
    DefaultBodyLimit::max(max_bytes)
}

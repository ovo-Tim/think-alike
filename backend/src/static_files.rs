use std::path::Path;
use tower_http::services::{ServeDir, ServeFile};

pub fn index_file(path: &Path) -> ServeFile {
    ServeFile::new(path.join("index.html"))
}

pub fn assets_service(path: &Path) -> ServeDir {
    ServeDir::new(path.join("assets"))
}

use axum::{
    Router, extract,
    http::{header, status::StatusCode},
    response::IntoResponse,
    routing::get,
};

use searchinator::{get_icon_for_file, get_icon_for_folder};
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::fs;
use tokio::net::TcpListener;

// Hardcoded port and icon directory for simplicity
const PORT: u16 = 7878;
// The directory where the SVG icons are stored
// Could be defined in a configuration file or environment variable for more flexibility
const ICON_DIR: &str = "icons";

// Check if the filename is valid (only contains alphanumeric characters, underscores, hyphens, and dots)
// This is a basic check to prevent directory traversal attacks and other invalid filenames.
fn check_filename(filename: &String) -> bool {
    filename
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
}

#[tokio::main]
pub async fn main() {
    // initialize async tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Create the Axum router and define the route for serving icons
    let app = Router::new().route("/icon/{kind}/{name}", get(get_icon));

    // Bind the server to the specified address and port
    // Add some error handling to log if the server fails to start
    let addr = SocketAddr::from(([127, 0, 0, 1], PORT));

    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!("Failed to bind to address {}: {}", addr, e);
            return;
        }
    };

    tracing::info!("Server listening on http://{}", addr);

    // Start the Axum server and handle any errors that occur during serving
    match axum::serve(listener, app).await {
        Ok(_) => tracing::info!("Server stopped gracefully"),
        Err(e) => tracing::error!("Server error: {}", e),
    }
}

async fn get_icon(
    extract::Path((kind, filename)): extract::Path<(String, String)>,
) -> Result<impl IntoResponse, StatusCode> {
    if !check_filename(&filename) {
        tracing::error!("Invalid filename: {}", filename);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Retrieve the icon number based on the kind (file or folder) and the filename
    // If no icon is found, log an error and return a 404 Not Found status code
    // If the kind is invalid, log an error and return a 400 Bad Request status code
    let svg_num: u64 = match kind.as_str() {
        "file" => get_icon_for_file(&filename).ok_or_else(|| {
            tracing::error!("No icon found for file: {}", filename);
            StatusCode::NOT_FOUND
        })?,
        "folder" => get_icon_for_folder(&filename).ok_or_else(|| {
            tracing::error!("No icon found for folder: {}", filename);
            StatusCode::NOT_FOUND
        })?,
        _ => {
            tracing::error!("Invalid kind: {}", kind);
            Err(StatusCode::BAD_REQUEST)
        }?,
    };

    // Construct the path to the SVG file
    let path = PathBuf::from(ICON_DIR).join(format!("{svg_num}.svg"));

    tracing::debug!(
        "Serving icon {} for file {}",
        path.to_string_lossy(),
        filename
    );

    // Read the SVG file asynchronously and handle any errors that occur during reading
    // If the file is read successfully, return the SVG content with the appropriate Content-Type header
    let svg = match fs::read(&path).await {
        Ok(svg) => svg,
        Err(e) => {
            tracing::error!("Failed to read SVG file {}: {}", path.to_string_lossy(), e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    Ok(([(header::CONTENT_TYPE, "image/svg+xml")], svg))
}

//! Serves the built UI (a Next.js static export) under `[serve] ui_path`.
//!
//! Not `ServeDir`: that redirects `/chaos/runs` to `/runs/` (it does not
//! know the nest prefix) and a static export wants `runs/index.html` served
//! in place. Rules: a directory serves its `index.html`, a file serves
//! itself, anything else serves `404.html` with status 404.

use std::path::{Component, Path, PathBuf};

use axum::{
    Router,
    body::Body,
    extract::{Request, State},
    http::{StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
};
use tower::ServiceExt;
use tower_http::services::ServeFile;

#[derive(Clone)]
struct Ui {
    dir: PathBuf,
    prefix: String,
}

/// A router that serves `dir` for every path under `ui_path`.
pub fn router(ui_path: &str, dir: &Path) -> Router {
    let state = Ui {
        dir: dir.to_path_buf(),
        prefix: ui_path.to_owned(),
    };
    Router::new()
        .route(ui_path, get(serve))
        .route(&format!("{ui_path}/"), get(serve))
        .route(&format!("{ui_path}/{{*rest}}"), get(serve))
        .with_state(state)
}

async fn serve(State(ui): State<Ui>, uri: Uri, req: Request) -> Response {
    let Some(rel) = relative(uri.path(), &ui.prefix, &ui.dir) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let target = if rel.is_dir() {
        rel.join("index.html")
    } else if rel.is_file() {
        rel
    } else {
        let mut response = file(ui.dir.join("404.html"), req).await;
        *response.status_mut() = StatusCode::NOT_FOUND;
        return response;
    };
    file(target, req).await
}

async fn file(path: PathBuf, req: Request) -> Response {
    let (parts, _) = req.into_parts();
    let req = Request::from_parts(parts, Body::empty());
    match ServeFile::new(path).oneshot(req).await {
        Ok(r) => r.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Map a request path under `prefix` to a path inside `dir`, refusing any
/// segment that could step out of it.
fn relative(request_path: &str, prefix: &str, dir: &Path) -> Option<PathBuf> {
    let rest = request_path.strip_prefix(prefix)?;
    let decoded = percent_decode(rest);
    let mut out = dir.to_path_buf();
    for seg in decoded.split('/').filter(|s| !s.is_empty()) {
        match Path::new(seg).components().next() {
            Some(Component::Normal(_)) if !seg.contains('\\') => out.push(seg),
            _ => return None,
        }
    }
    Some(out)
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Some(v) = std::str::from_utf8(&bytes[i + 1..i + 3])
                .ok()
                .and_then(|h| u8::from_str_radix(h, 16).ok())
        {
            out.push(v);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_stay_inside_the_directory() {
        let dir = Path::new("/srv/ui");
        assert_eq!(
            relative("/chaos/runs/", "/chaos", dir),
            Some(dir.join("runs"))
        );
        assert_eq!(relative("/chaos", "/chaos", dir), Some(dir.to_path_buf()));
        assert_eq!(
            relative("/chaos/a%20b", "/chaos", dir),
            Some(dir.join("a b"))
        );
        assert_eq!(relative("/chaos/../etc", "/chaos", dir), None);
        assert_eq!(relative("/chaos/%2e%2e/etc", "/chaos", dir), None);
        assert_eq!(relative("/other", "/chaos", dir), None);
    }
}

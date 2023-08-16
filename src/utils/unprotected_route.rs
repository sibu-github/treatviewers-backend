use axum::http::Uri;

const UNPROTECTED_PATHS: [&str; 1] = ["/user/login"];

const ADMIN_ONLY_PATHS: [&str; 1] = ["/admin/login"];

pub fn is_unprotected_path(uri: &Uri) -> bool {
    UNPROTECTED_PATHS.contains(&uri.path())
}

pub fn is_admin_only_path(uri: &Uri) -> bool {
    ADMIN_ONLY_PATHS.contains(&uri.path())
}

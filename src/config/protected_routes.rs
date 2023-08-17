use axum::http::Uri;

const UNPROTECTED_PATHS: [&str; 4] = ["/ping", "/tempApiGetOtp", "/tempApiGetToken", "/user/login"];

const ADMIN_ONLY_PATHS: [&str; 1] = ["/admin/login"];

pub fn is_unprotected_path(uri: &Uri) -> bool {
    UNPROTECTED_PATHS.contains(&uri.path())
}

pub fn is_admin_only_path(uri: &Uri) -> bool {
    ADMIN_ONLY_PATHS.contains(&uri.path())
}

#[cfg(test)]
pub fn get_an_unprotected_path() -> &'static str {
    UNPROTECTED_PATHS[0]
}

#[cfg(test)]
pub fn get_an_admin_path() -> &'static str {
    ADMIN_ONLY_PATHS[0]
}

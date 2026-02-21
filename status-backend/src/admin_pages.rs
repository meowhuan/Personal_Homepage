use axum::{
    http::header,
    response::{Html, IntoResponse},
};

const ADMIN_COMMON_CSS: &str = include_str!("../templates/admin/common.css");

pub async fn schedule_admin_page() -> impl IntoResponse {
    Html(include_str!("../templates/admin/schedule_admin.html"))
}

pub async fn status_admin_page() -> impl IntoResponse {
    Html(include_str!("../templates/admin/status_admin.html"))
}

pub async fn blog_admin_page() -> impl IntoResponse {
    Html(include_str!("../templates/admin/blog_admin.html"))
}

pub async fn admin_common_css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        ADMIN_COMMON_CSS,
    )
}

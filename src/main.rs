mod bbs;
mod now;
mod web;

use std::{net::SocketAddr, path::PathBuf, time::Duration};

use anyhow::{Context, Result};
use axum::{
    Router,
    body::Body,
    extract::Path,
    http::{Request, StatusCode, Uri},
    response::{Html, IntoResponse, Redirect, Response},
};
use clap::Parser;
use tower::ServiceExt;
use tower_http::services::ServeDir;

const USER_LIST: &str = include_str!("../templates/users.html");

#[derive(Debug, Parser)]
#[command(version, about = "The salyut.one website")]
struct Arguments {
    #[arg(long, default_value = "127.0.0.1:8082")]
    listen: SocketAddr,

    #[arg(long, default_value_os_t = bbs_socket())]
    bbs_socket: PathBuf,

    #[arg(long, default_value = "/usr/bin/pinky")]
    pinky: PathBuf,

    #[arg(long, default_value_t = 3)]
    pinky_timeout_seconds: u64,
}

#[cfg(target_os = "macos")]
fn bbs_socket() -> PathBuf {
    std::env::var_os("TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("salyut-bbs/users.sock")
}

#[cfg(not(target_os = "macos"))]
fn bbs_socket() -> PathBuf {
    PathBuf::from("/run/salyut-bbs/users/salyut.sock")
}

#[tokio::main]
async fn main() -> Result<()> {
    let arguments = Arguments::parse();
    let static_ = ServeDir::new("static");

    let app = Router::new()
        .route("/~{username}", axum::routing::get(redirect_user_site))
        .route("/~{username}/{*path}", axum::routing::get(render_user_site))
        .route("/users", axum::routing::get(get_users))
        .merge(bbs::router(arguments.bbs_socket))
        .merge(now::router(
            arguments.pinky,
            Duration::from_secs(arguments.pinky_timeout_seconds),
        ))
        .fallback_service(static_);

    let listener = tokio::net::TcpListener::bind(arguments.listen)
        .await
        .with_context(|| format!("listen on {}", arguments.listen))?;

    eprintln!("salyut-site listening on http://{}", arguments.listen);

    axum::serve(listener, app).await.context("serve HTTP")
}

async fn get_users() -> Html<String> {
    let all_users = unsafe { users::all_users() };

    let mut user_list_html = String::new();

    for user in all_users {
        // Ignore system users and root (UID 0) and users with UID > 60,000
        if (1_000..60_000).contains(&user.uid()) {
            let username = user.name().to_string_lossy();
            if web::valid_username(&username) {
                let username = web::escape(&username);
                user_list_html
                    .push_str(&format!("<li><a href=\"/~{username}\">{username}</a></li>"));
            }
        }
    }

    Html(USER_LIST.replace("{{USER_LIST}}", &user_list_html))
}

async fn redirect_user_site(Path(username): Path<String>, uri: Uri) -> Response {
    if !web::valid_username(&username) {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let mut location = format!("/~{username}/");
    if let Some(query) = uri.query() {
        location.push('?');
        location.push_str(query);
    }
    Redirect::permanent(&location).into_response()
}

async fn render_user_site(
    Path((username, path)): Path<(String, String)>,
    mut request: Request<Body>,
) -> Response {
    if !web::valid_username(&username) {
        return StatusCode::BAD_REQUEST.into_response();
    }

    *request.uri_mut() = format!("/{path}")
        .parse::<Uri>()
        .expect("relative URI is derived from a valid request URI");

    ServeDir::new(format!("/srv/user_sites/{username}"))
        .oneshot(request)
        .await
        .expect("ServeDir is infallible")
        .map(Body::new)
}

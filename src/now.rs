use std::{path::PathBuf, time::Duration};

use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use serde::Deserialize;
use tokio::process::Command;

use crate::web;

const PAGE: &str = include_str!("../templates/now.html");
const MAX_OUTPUT: usize = 64 * 1024;

#[derive(Clone)]
struct Config {
    pinky: PathBuf,
    timeout: Duration,
}

#[derive(Default, Deserialize)]
struct Lookup {
    username: Option<String>,
}

pub fn router(pinky: PathBuf, timeout: Duration) -> Router {
    Router::new()
        .route("/now", get(index))
        .route("/now/", get(|| async { Redirect::permanent("/now") }))
        .route("/now/~{username}", get(user))
        .route(
            "/now/~{username}/",
            get(|Path(username): Path<String>| async move {
                Redirect::permanent(&format!("/now/~{username}"))
            }),
        )
        .with_state(Config { pinky, timeout })
}

async fn index(Query(lookup): Query<Lookup>) -> Response {
    if let Some(username) = lookup.username.filter(|value| web::valid_username(value)) {
        return Redirect::to(&format!("/now/~{username}")).into_response();
    }
    Html(page(
        "Pinky",
        "<h1>Pinky</h1>\
         <p>Look up a salyut.one Unix user.</p><hr>\
         <form action=\"/now\" method=\"get\">\
         <label for=\"username\">~</label><input id=\"username\" name=\"username\" \
         autocomplete=\"username\" pattern=\"[a-z_][a-z0-9_-]{0,31}\" required>\
         <button type=\"submit\">[Look Up]</button></form>",
    ))
    .into_response()
}

async fn user(State(config): State<Config>, Path(username): Path<String>) -> Response {
    if !web::valid_username(&username) || !account_exists(username.clone()).await {
        return (
            StatusCode::NOT_FOUND,
            Html(page("User not found", "<h1>User not found</h1>")),
        )
            .into_response();
    }

    let mut command = Command::new(&config.pinky);
    command
        .args(["-lb", "--"])
        .arg(&username)
        .kill_on_drop(true);
    let output = match tokio::time::timeout(config.timeout, command.output()).await {
        Ok(Ok(output)) if output.status.success() => output.stdout,
        Ok(Ok(output)) => {
            eprintln!(
                "pinky {username:?} exited with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            );
            return unavailable();
        }
        Ok(Err(error)) => {
            eprintln!("start {}: {error}", config.pinky.display());
            return unavailable();
        }
        Err(_) => {
            eprintln!("pinky {username:?} timed out after {:?}", config.timeout);
            return unavailable();
        }
    };

    let output = String::from_utf8_lossy(&output[..output.len().min(MAX_OUTPUT)]);
    Html(page(
        &format!("~{username}"),
        &format!(
            "<h1>~{username}</h1><p class=\"command\">$ pinky -lb -- {username}</p>\
             <hr><pre>{output}</pre>",
            username = web::escape(&username),
            output = web::escape(&output),
        ),
    ))
    .into_response()
}

async fn account_exists(username: String) -> bool {
    tokio::task::spawn_blocking(move || users::get_user_by_name(&username).is_some())
        .await
        .unwrap_or(false)
}

fn unavailable() -> Response {
    (
        StatusCode::BAD_GATEWAY,
        Html(page(
            "Pinky unavailable",
            "<h1>Pinky unavailable</h1><p>Try again later.</p>",
        )),
    )
        .into_response()
}

fn page(title: &str, content: &str) -> String {
    web::page(PAGE, title, content)
}

#[cfg(test)]
mod tests {
    use crate::web;

    #[test]
    fn escapes_pinky_output() {
        let html = super::page(
            "~alice",
            &format!("<pre>{}</pre>", web::escape("<script>&")),
        );
        assert!(html.contains("&lt;script&gt;&amp;"));
        assert!(!html.contains("<script>"));
    }
}

mod view;

use std::path::PathBuf;

use anyhow::Result;
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use salyut_bbs::client::Client;

pub fn router(socket: PathBuf) -> Router {
    Router::new()
        .route("/healthz", get(health))
        .route("/bbs", get(index))
        .route("/bbs/", get(|| async { Redirect::permanent("/bbs") }))
        .route("/bbs/boards/{slug}", get(board))
        .route("/bbs/posts/{id}", get(post))
        .with_state(Client::new(socket))
}

async fn health(State(client): State<Client>) -> Response {
    match tokio::task::spawn_blocking(move || client.boards()).await {
        Ok(Ok(_)) => (StatusCode::OK, "ok\n").into_response(),
        Ok(Err(error)) => {
            eprintln!("BBS health check failed: {error:#}");
            (StatusCode::SERVICE_UNAVAILABLE, "unavailable\n").into_response()
        }
        Err(error) => {
            eprintln!("BBS health task failed: {error}");
            (StatusCode::INTERNAL_SERVER_ERROR, "unavailable\n").into_response()
        }
    }
}

async fn index(State(client): State<Client>) -> Response {
    render(move || Ok(Some(view::index(&client.boards()?)))).await
}

async fn board(State(client): State<Client>, Path(slug): Path<String>) -> Response {
    render(move || {
        let Some(board) = client
            .boards()?
            .into_iter()
            .find(|board| board.slug == slug)
        else {
            return Ok(None);
        };
        Ok(Some(view::board(
            &board,
            &client.posts(&board.slug, 200, 0)?,
        )))
    })
    .await
}

async fn post(State(client): State<Client>, Path(id): Path<i64>) -> Response {
    render(move || Ok(client.post(id)?.as_ref().map(view::post))).await
}

async fn render(operation: impl FnOnce() -> Result<Option<String>> + Send + 'static) -> Response {
    match tokio::task::spawn_blocking(operation).await {
        Ok(Ok(Some(body))) => Html(body).into_response(),
        Ok(Ok(None)) => (
            StatusCode::NOT_FOUND,
            Html(view::page("Not found", "<h1>Not found</h1>")),
        )
            .into_response(),
        Ok(Err(error)) => {
            eprintln!("BBS request failed: {error:#}");
            (
                StatusCode::BAD_GATEWAY,
                Html(view::page(
                    "BBS unavailable",
                    "<h1>BBS unavailable</h1><p>Try again later.</p>",
                )),
            )
                .into_response()
        }
        Err(error) => {
            eprintln!("BBS task failed: {error}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(view::page(
                    "BBS unavailable",
                    "<h1>BBS unavailable</h1><p>Try again later.</p>",
                )),
            )
                .into_response()
        }
    }
}

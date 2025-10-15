use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{delete, get},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

use crate::{
    config::Config,
    models::{apartment::Apartment, watchlist::Watchlist},
    services::watchlists,
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Deserialize)]
pub struct QueryChat {
    pub chat_id: i64,
}

#[derive(Deserialize)]
pub struct SubscribeRequest {
    pub chat_id: i64,
    pub location: String,
    pub min_size: f64,
    pub max_size: f64,
    pub target_yield: f64,
}

#[derive(Serialize)]
pub struct WatchlistsResponse {
    pub watchlists: Vec<Watchlist>,
}

#[derive(Serialize)]
pub struct ApartmentsResponse {
    pub apartments: Vec<Apartment>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/api/watchlists",
            get(list_watchlists).post(subscribe_watchlist),
        )
        .route("/api/watchlists/:id", delete(delete_watchlist))
        .route("/api/watchlists/:id/apartments", get(get_all_apartments))
        .route("/api/watchlists/:id/matching", get(get_matching_apartments))
        .layer(middleware::from_fn(cors_layer))
        .with_state(state)
}

pub async fn start_http_server(
    state: AppState,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) {
    let bind_addr = state
        .config
        .http_bind_address
        .clone()
        .unwrap_or_else(|| "0.0.0.0:8080".to_string());

    let listener = TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|err| panic!("failed to bind http listener on {}: {}", bind_addr, err));
    let app = router(state);

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.recv().await;
        })
        .await
        .expect("HTTP server crashed");
}

async fn cors_layer(req: axum::http::Request<axum::body::Body>, next: Next) -> Response {
    if req.method() == Method::OPTIONS {
        let mut response = Response::new(axum::body::Body::empty());
        apply_cors_headers(response.headers_mut());
        *response.status_mut() = StatusCode::NO_CONTENT;
        response
    } else {
        let mut response = next.run(req).await;
        apply_cors_headers(response.headers_mut());
        response
    }
}

fn apply_cors_headers(headers: &mut axum::http::HeaderMap) {
    headers.insert(
        axum::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    headers.insert(
        axum::http::header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("content-type"),
    );
    headers.insert(
        axum::http::header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET, POST, DELETE, OPTIONS"),
    );
}

async fn list_watchlists(
    State(state): State<AppState>,
    axum::extract::Query(QueryChat { chat_id }): axum::extract::Query<QueryChat>,
) -> Result<Json<ApiResponse<WatchlistsResponse>>, StatusCode> {
    let watchlists = watchlists::list(&state.config, chat_id);
    Ok(Json(ApiResponse {
        data: WatchlistsResponse { watchlists },
    }))
}

async fn subscribe_watchlist(
    State(state): State<AppState>,
    Json(body): Json<SubscribeRequest>,
) -> Result<Json<ApiResponse<Watchlist>>, StatusCode> {
    watchlists::subscribe(
        state.config.clone(),
        body.chat_id,
        body.location,
        (body.min_size, body.max_size),
        body.target_yield,
    )
    .await
    .map(|watchlist| Json(ApiResponse { data: watchlist }))
    .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn delete_watchlist(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i32>,
    axum::extract::Query(QueryChat { chat_id }): axum::extract::Query<QueryChat>,
) -> StatusCode {
    match watchlists::delete(&state.config, chat_id, id) {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::NOT_FOUND,
    }
}

async fn get_all_apartments(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i32>,
    axum::extract::Query(QueryChat { chat_id }): axum::extract::Query<QueryChat>,
) -> Result<Json<ApiResponse<ApartmentsResponse>>, StatusCode> {
    watchlists::get_all_apartments(&state.config, chat_id, id)
        .map(|apartments| {
            Json(ApiResponse {
                data: ApartmentsResponse { apartments },
            })
        })
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn get_matching_apartments(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<i32>,
    axum::extract::Query(QueryChat { chat_id }): axum::extract::Query<QueryChat>,
) -> Result<Json<ApiResponse<ApartmentsResponse>>, StatusCode> {
    watchlists::get_matching_apartments(&state.config, chat_id, id)
        .map(|apartments| {
            Json(ApiResponse {
                data: ApartmentsResponse { apartments },
            })
        })
        .map_err(|_| StatusCode::BAD_REQUEST)
}

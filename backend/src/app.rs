use crate::{auth::AuthService, config::Config, embedding::EmbeddingClient, routes, static_files};
use axum::{routing::get_service, Router};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub pool: PgPool,
    pub auth: AuthService,
    pub embeddings: EmbeddingClient,
}

pub async fn build_router(config: Arc<Config>, pool: PgPool) -> Router {
    let auth = AuthService::new(config.clone());
    let embeddings = EmbeddingClient::new(config.clone());
    let state = AppState {
        config: config.clone(),
        pool,
        auth,
        embeddings,
    };

    Router::new()
        .nest("/api", routes::api_router())
        .route_service("/", get_service(static_files::index_file(&config.static_dir)))
        .route_service("/kanban", get_service(static_files::index_file(&config.static_dir)))
        .nest_service("/assets", static_files::assets_service(&config.static_dir))
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(state))
}

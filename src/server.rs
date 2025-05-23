use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use zeroize::Zeroizing;

use crate::api::State;

#[derive(Debug, Deserialize)]
struct AddRequest {
    service: String,
    username: String,
    password: String,
    master_password: String,
}

#[derive(Debug, Deserialize)]
struct GetRequest {
    service: String,
    master_password: String,
}

#[derive(Debug, Serialize)]
struct GetResponse {
    password: String,
}

async fn add_handler(Json(payload): Json<AddRequest>) -> Result<(), (StatusCode, String)> {
    let result = (|| {
        let mut state = State::load()?;
        state.add(
            payload.service,
            payload.username,
            Zeroizing::new(payload.password),
            Zeroizing::new(payload.master_password),
        )?;
        state.save()
    })();

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn get_handler(
    Json(payload): Json<GetRequest>,
) -> Result<Json<GetResponse>, (StatusCode, String)> {
    let result = (|| {
        let state = State::load()?;
        state.get(
            payload.service.clone(),
            Zeroizing::new(payload.master_password),
        )
    })();

    match result {
        Ok(password) => Ok(Json(GetResponse {
            password: password.to_string(),
        })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn list_handler() -> Result<Json<Vec<String>>, (StatusCode, String)> {
    let result = (|| {
        let state = State::load()?;
        state.list()
    })();

    match result {
        Ok(services) => Ok(Json(services)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

pub async fn start_server(port: u16) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/add", post(add_handler))
        .route("/get", post(get_handler))
        .route("/list", get(list_handler));

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    println!("Server started on http://0.0.0.0:{}", port);

    axum::serve(listener, app).await?;
    Ok(())
}

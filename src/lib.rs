mod db_fns;
mod types;

use crate::db_fns::{db_conn, make_table};
use crate::types::*;
use anyhow;
use axum::{
    extract,
    extract::State,
    response::{ErrorResponse, IntoResponse, Redirect, Result},
    routing::get,
    Router, Server,
};
use axum_macros::debug_handler;
use axum_template::{engine::Engine, RenderHtml};
use handlebars::Handlebars;
use http::StatusCode;
use local_ip_address::local_ip;
use rusqlite::Connection;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

type AppEngine = Engine<Handlebars<'static>>;

#[derive(Clone, Debug, extract::FromRef)]
struct AppState {
    addr: IpAddr,
    conn: Conn,
    engine: AppEngine,
}

impl AppState {
    fn new(conn: Connection) -> anyhow::Result<Self> {
        let conn = Arc::new(Mutex::new(conn));
        let mut hbs = Handlebars::new();
        hbs.register_templates_directory(".hbs", "../templates")?;
        let addr = local_ip()?;
        Ok(Self {
            addr,
            conn,
            engine: Engine::from(hbs),
        })
    }
}

fn entries_url(id: usize) -> String {
    format!("/entries/{id}")
}

fn db_404(e: anyhow::Error) -> ErrorResponse {
    match e.downcast_ref() {
        Some(re) => match re {
            rusqlite::Error::QueryReturnedNoRows => {
                (StatusCode::NOT_FOUND, "entry not found").into()
            }
            e => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("internal server error: {e}"),
            )
                .into(),
        },
        None => (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into(),
    }
}

fn db_400(e: anyhow::Error) -> ErrorResponse {
    match e.downcast_ref() {
        Some(
            err @ rusqlite::Error::InvalidParameterName(_)
            | err @ rusqlite::Error::ToSqlConversionFailure(_),
        ) => (StatusCode::BAD_REQUEST, format!("invalid request: {err}")).into(),
        Some(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("internal server error: {err}"),
        )
            .into(),
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("internal server error: {e}"),
        )
            .into(),
    }
}

#[debug_handler(state = AppState)]
async fn get_index(
    State(engine): State<AppEngine>,
    State(conn): State<Conn>,
    State(addr): State<IpAddr>,
) -> Result<impl IntoResponse> {
    match db_fns::get_all_entries(conn).await {
        Ok(entries) => Ok(RenderHtml("index", engine, ManyEntries { entries, addr })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("internal server error: {e}"),
        )
            .into()),
    }
}

#[debug_handler(state = AppState)]
async fn get_entry(
    State(engine): State<AppEngine>,
    State(conn): State<Conn>,
    State(addr): State<IpAddr>,
    extract::Path(entry_id): extract::Path<usize>,
) -> Result<impl IntoResponse> {
    match db_fns::get_entry(conn, entry_id).await {
        Ok(entry) => Ok(RenderHtml("entry", engine, SingleEntry { entry, addr })),
        Err(e) => Err(db_404(e)),
    }
}

#[debug_handler(state = AppState)]
async fn delete_entry(
    State(conn): State<Conn>,
    extract::Path(entry_id): extract::Path<usize>,
) -> Result<impl IntoResponse> {
    match db_fns::delete_entry(conn, entry_id).await {
        Ok(num_affected) if num_affected == 1 => Ok(Redirect::to("/")),
        Ok(num_affected) if num_affected == 0 => {
            Err((StatusCode::NOT_FOUND, "entry not found").into())
        }
        Ok(_) => Err((StatusCode::BAD_REQUEST, "deleted >1 entry").into()),
        Err(e) => {
            let msg = format!("internal server error: {e}");
            Err((StatusCode::INTERNAL_SERVER_ERROR, msg).into())
        }
    }
}

#[debug_handler(state = AppState)]
async fn create_entry(
    State(conn): State<Conn>,
    extract::Form(data): extract::Form<Entry>,
) -> Result<impl IntoResponse> {
    match db_fns::create_entry(conn, data).await {
        Ok(new_id) => Ok(Redirect::to(entries_url(new_id).as_str())),
        Err(e) => Err(db_400(e)),
    }
}

#[debug_handler(state = AppState)]
async fn new_entry(State(engine): State<AppEngine>) -> Result<impl IntoResponse> {
    Ok(RenderHtml("new_entry", engine, ()))
}

#[debug_handler(state = AppState)]
async fn update_entry(
    State(conn): State<Conn>,
    extract::Path(entry_id): extract::Path<usize>,
    extract::Form(data): extract::Form<Entry>,
) -> Result<impl IntoResponse> {
    match db_fns::update_entry(conn, entry_id, data).await {
        Ok(num_affected) if num_affected == 1 => Ok(Redirect::to(entries_url(entry_id).as_str())),
        Ok(num_affected) if num_affected == 0 => {
            Err((StatusCode::NOT_FOUND, "entry not found").into())
        }
        Ok(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "multiple entries updated",
        )
            .into()),
        Err(e) => Err(db_400(e)),
    }
}

#[tokio::main]
pub async fn main() {
    let conn = make_table(db_conn().unwrap()).unwrap();
    let state = AppState::new(conn).unwrap();
    let app = Router::new()
        .route("/", get(get_index))
        .route("/entry", get(new_entry).post(create_entry))
        .route(
            "/entries/:entry_id",
            get(get_entry).delete(delete_entry).put(update_entry),
        )
        .with_state(state);
    Server::bind(&"0.0.0.0:9999".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap()
}

mod db_fns;
mod types;

use crate::db_fns::{db_conn, make_table};
use crate::types::*;
use anyhow;
use axum::{
    extract,
    extract::State,
    response::{ErrorResponse, IntoResponse, Redirect, Result},
    routing::{get, post},
    Router, Server,
};
use axum_template::{engine::Engine, RenderHtml};
use handlebars::Handlebars;
use http::StatusCode;
use local_ip_address::local_ip;
use std::net::IpAddr;
use std::sync::Arc;

type AppEngine = Engine<Handlebars<'static>>;

#[derive(Debug)]
struct Client(Arc<tokio_postgres::Client>);

impl Clone for Client {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

#[derive(Clone, Debug, extract::FromRef)]
struct AppState {
    addr: IpAddr,
    client: Client,
    engine: AppEngine,
}

impl AppState {
    fn new(client: tokio_postgres::Client) -> anyhow::Result<Self> {
        let mut pth = folder_path();
        pth.push("templates");
        let mut hbs = Handlebars::new();
        hbs.register_templates_directory(".hbs", pth.to_str().unwrap())?;
        let addr = local_ip()?;
        Ok(Self {
            addr,
            client: Client(Arc::new(client)),
            engine: Engine::from(hbs),
        })
    }
}

fn entries_url(id: u32) -> String {
    format!("/entries/{id}")
}

fn db_404() -> ErrorResponse {
    (StatusCode::NOT_FOUND, "entry not found").into()
}

fn db_400(e: anyhow::Error) -> ErrorResponse {
    (StatusCode::BAD_REQUEST, format!("invalid request: {e}")).into()
}

fn db_500(e: anyhow::Error) -> ErrorResponse {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("internal server error: {e})"),
    )
        .into()
}

async fn get_index(
    State(engine): State<AppEngine>,
    State(conn): State<Client>,
    State(addr): State<IpAddr>,
) -> Result<impl IntoResponse> {
    match db_fns::get_all_entries(&conn.0).await {
        Ok(entries) => Ok(RenderHtml("index", engine, ManyEntries { entries, addr })),
        Err(e) => Err(db_500(e)),
    }
}

async fn get_entry(
    State(engine): State<AppEngine>,
    State(conn): State<Client>,
    State(addr): State<IpAddr>,
    extract::Path(entry_id): extract::Path<u32>,
) -> Result<impl IntoResponse> {
    match db_fns::get_entry(&conn.0, entry_id).await {
        Ok(Some(entry)) => Ok(RenderHtml("entry", engine, SingleEntry { entry, addr })),
        Ok(_) => Err(db_404()),
        Err(e) => Err(db_500(e)),
    }
}

async fn delete_entry(
    State(conn): State<Client>,
    extract::Path(entry_id): extract::Path<u32>,
) -> Result<impl IntoResponse> {
    match db_fns::delete_entry(&conn.0, entry_id).await {
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

async fn create_entry(
    State(conn): State<Client>,
    extract::Form(data): extract::Form<Entry>,
) -> Result<impl IntoResponse> {
    match db_fns::create_entry(&conn.0, data).await {
        Ok(new_id) => Ok(Redirect::to(entries_url(new_id).as_str())),
        Err(e) => Err(db_400(e)),
    }
}

async fn new_entry(
    State(engine): State<AppEngine>,
    State(addr): State<IpAddr>,
) -> Result<impl IntoResponse> {
    Ok(RenderHtml("new_entry", engine, Addr { addr }))
}

async fn update_entry(
    State(conn): State<Client>,
    extract::Path(entry_id): extract::Path<u32>,
    extract::Form(data): extract::Form<Entry>,
) -> Result<impl IntoResponse> {
    match db_fns::update_entry(&conn.0, entry_id, data).await {
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
    let conn = db_conn().await.expect("db connection");
    make_table(&conn).await.expect("make table");
    let state = AppState::new(conn).unwrap();
    let app = Router::new()
        .route("/", get(get_index))
        .route("/entry", get(new_entry))
        .route("/entries", post(create_entry))
        .route("/entries/:entry_id", get(get_entry).post(update_entry))
        .route("/entries/:entry_id/delete", post(delete_entry))
        .with_state(state);
    Server::bind(&"0.0.0.0:9999".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap()
}

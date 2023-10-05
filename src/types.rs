use anyhow::Result;
use chrono::prelude::*;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::PathBuf;
use tokio_postgres::Row;

pub(crate) fn folder_path() -> PathBuf {
    let mut home = home_dir().unwrap();
    home.push(".tmt");
    home
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Addr {
    pub addr: IpAddr,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Entry {
    pub url: String,
    pub title: String,
    pub notes: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct DbEntry {
    pub id: u32,
    pub url: String,
    pub title: String,
    pub notes: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct SingleEntry {
    pub entry: DbEntry,
    pub addr: IpAddr,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ManyEntries {
    pub entries: Vec<DbEntry>,
    pub addr: IpAddr,
}

impl DbEntry {
    pub(crate) fn from_row(row: &Row) -> Result<Self> {
        let id = row.get::<&str, u32>("id");
        let url = row.get::<&str, String>("url");
        let title = row.get::<&str, String>("title");
        let notes = row.get::<&str, String>("notes");
        let created_at = row.get::<&str, DateTime<Utc>>("created_at");
        let updated_at = row.get::<&str, DateTime<Utc>>("updated_at");
        Ok(Self {
            id,
            url,
            title,
            notes,
            created_at,
            updated_at,
        })
    }
}

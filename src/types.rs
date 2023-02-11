use anyhow::Result;
use chrono::prelude::*;
use rusqlite::{Connection, Row};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) type Conn = Arc<Mutex<Connection>>;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Entry {
    pub url: String,
    pub title: String,
    pub notes: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct DbEntry {
    pub id: usize,
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
    pub(crate) fn from_row(row: &Row<'_>) -> Result<Self> {
        let id = row.get("id")?;
        let url = row.get("url")?;
        let title = row.get("title")?;
        let notes = row.get("notes")?;
        let created_at = row.get("created_at")?;
        let updated_at = row.get("updated_at")?;
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

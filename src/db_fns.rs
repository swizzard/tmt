use crate::types::*;
use anyhow::Result;
use tokio_postgres::{Client, NoTls};

pub(crate) async fn db_conn() -> Result<Client> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=swizzard port=5433 database=swizzard_toomanytabs",
        NoTls,
    )
    .await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    Ok(client)
}

pub(crate) async fn make_table(client: &Client) -> Result<()> {
    client
        .execute(
            r#"
         CREATE TABLE IF NOT EXISTS entries (
             id INTEGER PRIMARY KEY,
             url TEXT DEFAULT "" NOT NULL,
             title TEXT DEFAULT "" NOT NULL,
             notes TEXT DEFAULT "" NOT NULL,
             created_at TEXT DEFAULT (datetime('now', 'utc')),
             updated_at TEXT DEFAULT (datetime('now', 'utc'))
         );
         CREATE TRIGGER IF NOT EXISTS update_updated_trigger
         AFTER INSERT ON entries
         BEGIN
            UPDATE entries SET updated_at = (datetime('now', 'utc')) WHERE id = NEW.id;
         END;
         "#,
            &[],
        )
        .await?;
    Ok(())
}

pub(crate) async fn get_all_entries(client: &Client) -> Result<Vec<DbEntry>> {
    let entries = client
        .query("SELECT * FROM entries ORDER BY created_at DESC", &[])
        .await?
        .iter()
        .map(|row| DbEntry::from_row(row))
        .collect::<Result<Vec<DbEntry>>>()?;
    Ok(entries)
}

pub(crate) async fn get_entry(client: &Client, entry_id: u32) -> Result<Option<DbEntry>> {
    client
        .query_opt("SELECT * FROM entries WHERE id = ?", &[&entry_id])
        .await?
        .map(|row| DbEntry::from_row(&row))
        .transpose()
}

pub(crate) async fn delete_entry(client: &Client, entry_id: u32) -> Result<u64> {
    let num_affected = client
        .execute("DELETE FROM entries WHERE id = ?", &[&entry_id])
        .await?;
    Ok(num_affected)
}

pub(crate) async fn create_entry(client: &Client, data: Entry) -> Result<u32> {
    let new_id = client
        .query_one(
            "INSERT INTO entries (url, title, notes) VALUES ($1, $2, $3) RETURNING id",
            &[&data.url, &data.title, &data.notes],
        )
        .await?
        .get(0);
    Ok(new_id)
}

pub(crate) async fn update_entry(client: &Client, entry_id: u32, data: Entry) -> Result<u64> {
    let stmt = client
        .prepare("UPDATE entries SET url = $2, title = $3, notes = $4 WHERE id = $1")
        .await?;
    let num_affected = client
        .execute(&stmt, &[&entry_id, &data.url, &data.title, &data.notes])
        .await?;
    Ok(num_affected)
}

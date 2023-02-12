use crate::types::*;
use anyhow::{anyhow, Result};
use rusqlite::{named_params, Connection};

pub(crate) fn db_conn() -> Result<Connection> {
    let mut pth = folder_path();
    pth.push("db.db3");
    Connection::open(pth).map_err(|_| anyhow!("Error opening database"))
}

pub(crate) fn make_table(conn: Connection) -> Result<Connection> {
    conn.execute(
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
        (),
    )?;
    Ok(conn)
}

pub(crate) async fn get_all_entries(conn: Conn) -> Result<Vec<DbEntry>> {
    let c = conn.lock().await;
    let mut stmt = c.prepare("SELECT * FROM entries")?;
    let entries = stmt
        .query_and_then((), |row| DbEntry::from_row(row))?
        .collect::<Result<Vec<DbEntry>>>()?;
    Ok(entries)
}

pub(crate) async fn get_entry(conn: Conn, entry_id: usize) -> Result<DbEntry> {
    let c = conn.lock().await;
    let entry = c.query_row_and_then("SELECT * FROM entries WHERE id = ?", (entry_id,), |row| {
        DbEntry::from_row(row)
    })?;
    Ok(entry)
}

pub(crate) async fn delete_entry(conn: Conn, entry_id: usize) -> Result<usize> {
    let c = conn.lock().await;
    let num_affected = c.execute("DELETE FROM entries WHERE id = ?", (entry_id,))?;
    Ok(num_affected)
}

pub(crate) async fn create_entry(conn: Conn, data: Entry) -> Result<usize> {
    let c = conn.lock().await;
    let mut stmt =
        c.prepare("INSERT INTO entries (url, title, notes) VALUES (:url, :title, :notes)")?;
    let new_id = stmt
        .insert(named_params! {":url": data.url, ":title": data.title, ":notes": data.notes})?
        .try_into()?;
    Ok(new_id)
}

pub(crate) async fn update_entry(conn: Conn, entry_id: usize, data: Entry) -> Result<usize> {
    let c = conn.lock().await;
    let mut stmt =
        c.prepare("UPDATE entries SET url = :url, title = :title, notes = :notes WHERE id = :id")?;
    let num_affected = stmt.execute(named_params! {":url": data.url, ":title": data.title, ":notes": data.notes, ":id": entry_id})?;
    Ok(num_affected)
}

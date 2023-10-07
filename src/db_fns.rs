use crate::types::*;
use anyhow::Result;
use tokio_postgres::{Client, NoTls};

pub(crate) async fn db_conn() -> Result<Client> {
    match tokio_postgres::connect(
        "user=swizzard password=postgres dbname=swizzard_toomanytabs host=localhost port=5433",
        NoTls,
    )
    .await
    {
        Ok((client, connection)) => {
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("connection error: {}", e);
                }
            });
            Ok(client)
        }
        Err(e) => {
            eprintln!("db error: {}", e);
            Err(e.into())
        }
    }
}

pub(crate) async fn make_table(client: &Client) -> Result<()> {
    client
        .execute(
            r#"
         CREATE TABLE IF NOT EXISTS entries (
             id SERIAL PRIMARY KEY,
             url TEXT NOT NULL DEFAULT '',
             title TEXT NOT NULL DEFAULT '',
             notes TEXT NOT NULL DEFAULT '',
             created_at TIMESTAMP WITH TIME ZONE DEFAULT timezone('gmt', localtimestamp),
             updated_at TIMESTAMP WITH TIME ZONE DEFAULT timezone('gmt', localtimestamp)
         );
         "#,
            &[],
        )
        .await?;

    client
        .execute(
            r#"
         CREATE OR REPLACE FUNCTION uut() RETURNS TRIGGER AS $$
            
             BEGIN
                 NEW.updated_at = timezone('gmt', localtimestamp);
                 RETURN NEW;
             END;
         $$ LANGUAGE plpgsql;
         "#,
            &[],
        )
        .await?;
    client
        .execute(
            r#"
         DROP TRIGGER IF EXISTS update_updated_trigger ON entries;
         "#,
            &[],
        )
        .await?;
    client
        .execute(
            r#"
         CREATE TRIGGER update_updated_trigger
             AFTER INSERT ON entries
             FOR EACH ROW
             EXECUTE PROCEDURE uut();
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

pub(crate) async fn get_entry(client: &Client, entry_id: i32) -> Result<Option<DbEntry>> {
    client
        .query_opt("SELECT * FROM entries WHERE id = $1", &[&entry_id])
        .await?
        .map(|row| DbEntry::from_row(&row))
        .transpose()
}

pub(crate) async fn delete_entry(client: &Client, entry_id: i32) -> Result<u64> {
    let num_affected = client
        .execute("DELETE FROM entries WHERE id = $1", &[&entry_id])
        .await?;
    Ok(num_affected)
}

pub(crate) async fn create_entry(client: &Client, data: Entry) -> Result<i32> {
    let new_id = client
        .query_one(
            "INSERT INTO entries (url, title, notes) VALUES ($1, $2, $3) RETURNING id",
            &[&data.url, &data.title, &data.notes],
        )
        .await?
        .get(0);
    Ok(new_id)
}

pub(crate) async fn update_entry(client: &Client, entry_id: i32, data: Entry) -> Result<u64> {
    let num_affected = client
        .execute(
            "UPDATE entries SET url = $2, title = $3, notes = $4 WHERE id = $1",
            &[&entry_id, &data.url, &data.title, &data.notes],
        )
        .await?;
    Ok(num_affected)
}

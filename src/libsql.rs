mod constants;

use crate::constants::*;
use libsql::{Builder, Connection, Database};
use std::time::Instant;

#[tokio::main]
async fn main() {
    let db: Database = Builder::new_local("libsql.sqlite").build().await.unwrap();
    let conn: Connection = db.connect().unwrap();

    for line in PRAGMAS.to_string().split("\n") {
        if !line.is_empty() {
            let mut rows = conn.query(line, ()).await.unwrap();
            // while let Ok(Some(row)) = rows.next().await {
            //     println!("Row: {row:?}");
            // }
        }
    }
    conn.execute_batch(CREATE_TABLE_QUERY).await.unwrap();

    conn.query(INSERT_QUERY, (0, 5)).await.unwrap();
    conn.query(INSERT_QUERY, (1, 512)).await.unwrap();

    let row_count: i64 = conn
        .prepare(COUNT_QUERY)
        .await
        .unwrap()
        .query_row(())
        .await
        .unwrap()
        .get(0)
        .unwrap();

    println!("Done. row_count={row_count}");
}

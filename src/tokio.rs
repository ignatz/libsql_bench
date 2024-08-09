mod constants;

use crate::constants::*;
use std::time::Instant;
use tokio_rusqlite::Connection;

#[tokio::main]
async fn main() {
    let conn = Connection::open("tokio.sqlite").await.unwrap();

    conn.call(|c| Ok(c.execute_batch(&format!("{PRAGMAS}\n{CREATE_TABLE_QUERY}"))?))
        .await
        .unwrap();

    conn.call(move |c| Ok(c.execute(INSERT_QUERY, (0, 5))?))
        .await
        .unwrap();

    conn.call(move |c| Ok(c.execute(INSERT_QUERY, (1, 512))?))
        .await
        .unwrap();

    println!("Done");
}

mod constants;

use crate::constants::*;
use libsql::{Builder, Connection, Database};
use std::time::Instant;

#[tokio::main]
async fn main() {
    let db: Database = Builder::new_local("libsql.sqlite").build().await.unwrap();
    let conn: Connection = db.connect().unwrap();

    if let Err(err) = conn.execute_batch(PRAGMAS).await {
        println!(
            "Executing pragmas leads to: \"{err}\". Have to use query. Am I holding it wrong?"
        );
    }
    for line in PRAGMAS.to_string().split("\n") {
        if !line.is_empty() {
            let mut rows = conn.query(line, ()).await.unwrap();
            while let Ok(Some(row)) = rows.next().await {
                println!("Row: {row:?}");
            }
        }
    }
    conn.execute_batch(CREATE_TABLE_QUERY).await.unwrap();

    let start = Instant::now();
    let tasks: Vec<_> = (0..TASKS)
        .into_iter()
        .map(|task| {
            let conn = conn.clone();
            tokio::spawn(async move {
                let mut stmt = conn.prepare(BENCHMARK_QUERY).await.unwrap();
                for i in 0..N {
                    let id = task * N + i;
                    // WARN: Not sure I'm holding this right. Not resetting the prepared statement
                    // leads to parameters not being bound correctly and thus to UNIQUE constraint
                    // violations for person.id.
                    // I did run both and there were basically the same +/- peanuts.
                    stmt.reset();
                    stmt.execute((id, format!("{id}")))
                        .await
                        .expect(&format!("expected to pass: {id}"));

                    // ALTERNATIVELY: non-prepared statement works too.
                    // conn.execute(BENCHMARK_QUERY, (id, format!("{id}")))
                    //     .await
                    //     .unwrap();
                }
                println!("finished {task}");
            })
        })
        .collect();

    for t in tasks {
        t.await.unwrap();
    }

    let mut stmt = conn.prepare(COUNT_QUERY).await.unwrap();
    let count: i64 = stmt.query_row(()).await.unwrap().get(0).unwrap();

    assert_eq!(count, TASKS * N);
    println!(
        "Inserted {count} rows in {elapsed:?}",
        elapsed = Instant::now() - start
    );
}

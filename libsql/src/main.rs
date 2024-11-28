mod constants;

use crate::constants::*;
use libsql::{Builder, Connection, Database};
use std::time::Instant;

fn main() {
  let tmp_dir = tempfile::TempDir::new().unwrap();

  let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();

  rt.block_on(async {
    let fname = tmp_dir.path().join("libsql.sqlite");
    println!("DB file: {fname:?}");

    let db: Database = Builder::new_local(fname).build().await.unwrap();
    let conn: Connection = db.connect().unwrap();

    if let Err(err) = conn.execute_batch(PRAGMAS).await {
      println!("Executing pragmas leads to: \"{err}\". Have to use query. Am I holding it wrong?");
    }
    for line in PRAGMAS.to_string().split("\n") {
      if !line.is_empty() {
        let _rows = conn.query(line, ()).await.unwrap();
      }
    }
    conn.execute_batch(CREATE_TABLE_QUERY).await.unwrap();

    let start = Instant::now();
    let tasks: Vec<_> = (0..TASKS)
      .into_iter()
      .map(|task| {
        let conn = conn.clone();
        tokio::spawn(async move {
          // let mut stmt = conn.prepare(BENCHMARK_QUERY).await.unwrap();
          for i in 0..N {
            let id = task * N + i;
            // stmt.reset();
            // stmt.execute((id, format!("{id}")))
            //     .await
            //     .expect(&format!("expected to pass: {id}"));

            conn
              .execute(BENCHMARK_QUERY, (id, format!("{id}")))
              .await
              .unwrap();
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
  });
}

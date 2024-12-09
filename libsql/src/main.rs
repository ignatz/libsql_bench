use constants::*;
use libsql::{Builder, Connection, Database};
use std::time::Instant;

const NAME: &str = "LIBSQL";

fn main() {
  let tmp_dir = tempfile::TempDir::new().unwrap();

  let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();

  rt.block_on(async {
    let fname = tmp_dir.path().join(format!("{NAME}.sqlite"));
    println!("DB file: {fname:?}");

    let db: Database = Builder::new_local(fname.clone()).build().await.unwrap();
    let conn: Connection = db.connect().unwrap();

    let version: String = conn
      .prepare("SELECT sqlite_version()")
      .await
      .unwrap()
      .query_row(())
      .await
      .unwrap()
      .get(0)
      .unwrap();
    println!("Sqlite v{version:?}");

    if let Err(err) = conn.execute_batch(PRAGMAS).await {
      println!("Executing pragmas leads to: \"{err}\". Have to use query. Am I holding it wrong?");
    }
    for line in PRAGMAS.to_string().split("\n") {
      if !line.is_empty() {
        let _rows = conn.query(line, ()).await.unwrap();
      }
    }
    conn.execute_batch(CREATE_TABLE_QUERY).await.unwrap();

    {
      // Insert
      let start = Instant::now();
      let tasks: Vec<_> = (0..num_tasks())
        .into_iter()
        .map(|task| {
          let conn = conn.clone();
          rt.spawn(async move {
            // let mut stmt = conn.prepare(BENCHMARK_QUERY).await.unwrap();
            for i in 0..N {
              let id = (task * N + i) as i64;
              // stmt.reset();
              // stmt.execute((id, format!("{id}")))
              //     .await
              //     .unwrap();
              conn
                .execute(BENCHMARK_QUERY, (id, format!("{id}")))
                .await
                .unwrap();
            }
          })
        })
        .collect();

      for t in tasks {
        t.await.unwrap();
      }

      println!(
        "[{NAME}]\n\tInserted {count} rows in {elapsed:?}",
        count = num_tasks() * N,
        elapsed = Instant::now() - start,
      );
    }

    let mut stmt = conn.prepare(COUNT_QUERY).await.unwrap();
    let count: i64 = stmt.query_row(()).await.unwrap().get(0).unwrap();
    assert_eq!(count, (num_tasks() * N) as i64);

    {
      // Read
      let start = Instant::now();
      let tasks: Vec<_> = (0..num_tasks())
        .into_iter()
        .map(|task| {
          let conn = conn.clone();
          rt.spawn(async move {
            for i in 0..N {
              let id = (task * N + i) as i64;

              let mut rows = conn
                .query("SELECT * FROM person WHERE id = $1", [id])
                .await
                .unwrap();
              rows.next().await.unwrap();
            }
          })
        })
        .collect();

      for t in tasks {
        t.await.unwrap();
      }

      println!(
        "[{NAME}]\n\tRead {count} rows in {elapsed:?}",
        count = num_tasks() * N,
        elapsed = Instant::now() - start,
      );
    }

    std::fs::remove_file(fname).unwrap();
  });
}

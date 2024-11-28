use constants::*;
use std::time::Instant;
use tokio_rusqlite::Connection;

fn main() {
  let tmp_dir = tempfile::TempDir::new().unwrap();

  let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();

  rt.block_on(async {
    let fname = tmp_dir.path().join("tokio_rusqlite.sqlite");
    println!("DB file: {fname:?}");

    let conn = Connection::open(fname.clone()).await.unwrap();

    conn
      .call(move |c| {
        let version: String = c
          .query_row("SELECT sqlite_version()", (), |row| row.get(0))
          .unwrap();
        println!("Sqlite v{version:?}");
        Ok(())
      })
      .await
      .unwrap();

    conn
      .call(|c| Ok(c.execute_batch(&format!("{PRAGMAS}\n{CREATE_TABLE_QUERY}"))?))
      .await
      .unwrap();

    let start = Instant::now();
    let tasks: Vec<_> = (0..num_tasks())
      .into_iter()
      .map(|task| {
        let conn = conn.clone();

        rt.spawn(async move {
          for i in 0..N {
            let id = task * N + i;
            conn
              .call(move |c| {
                let mut stmt = c.prepare_cached(BENCHMARK_QUERY).unwrap();
                Ok(stmt.execute((id, format!("{id}")))?)
              })
              .await
              .unwrap();
          }
        })
      })
      .collect();

    for t in tasks {
      t.await.unwrap();
    }

    let count = conn
      .call(|c| Ok(c.query_row(COUNT_QUERY, (), |row| row.get::<_, i64>(0))?))
      .await
      .unwrap();

    assert_eq!(count, (num_tasks() * N) as i64);
    println!(
      "Inserted {count} rows in {elapsed:?}",
      elapsed = Instant::now() - start
    );

    std::fs::remove_file(fname).unwrap();
  });
}

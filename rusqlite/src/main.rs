use constants::*;
use rusqlite::Connection;
use std::time::Instant;

fn main() {
  let tmp_dir = tempfile::TempDir::new().unwrap();

  let rt = tokio::runtime::Builder::new_multi_thread().build().unwrap();

  let fname = tmp_dir.path().join("rusqlite.sqlite");
  println!("DB file: {fname:?}");

  let conn = Connection::open(fname.clone()).unwrap();

  let version: String = conn
    .query_row("SELECT sqlite_version()", (), |row| row.get(0))
    .unwrap();
  println!("Sqlite v{version:?}");

  conn
    .execute_batch(&format!("{PRAGMAS}\n{CREATE_TABLE_QUERY}"))
    .unwrap();

  let start = Instant::now();
  let tasks: Vec<_> = (0..num_tasks())
    .into_iter()
    .map(|task| {
      let conn = Connection::open(fname.clone()).unwrap();

      rt.spawn(async move {
        for i in 0..N {
          let id = task * N + i;

          let mut stmt = conn.prepare_cached(BENCHMARK_QUERY).unwrap();
          stmt.execute((id, format!("{id}"))).unwrap();
        }
      })
    })
    .collect();

  rt.block_on(async {
    for t in tasks {
      t.await.unwrap();
    }
  });

  let count = conn
    .query_row(COUNT_QUERY, (), |row| row.get::<_, i64>(0))
    .unwrap();

  assert_eq!(count, (num_tasks() * N) as i64);
  println!(
    "Inserted {count} rows in {elapsed:?}",
    elapsed = Instant::now() - start
  );

  std::fs::remove_file(fname).unwrap();
}

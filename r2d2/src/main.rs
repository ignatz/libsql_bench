use constants::*;
use std::time::Instant;

use r2d2_sqlite::SqliteConnectionManager;

const NAME: &str = "R2D2";

fn new_conn(path: &std::path::PathBuf) -> r2d2::Pool<SqliteConnectionManager> {
  let manager = SqliteConnectionManager::file(path);
  let pool = r2d2::Pool::new(manager).unwrap();

  return pool;
}

fn main() {
  let tmp_dir = tempfile::TempDir::new().unwrap();

  let fname = tmp_dir.path().join(format!("{NAME}.sqlite"));
  println!("DB file: {fname:?}");

  let conn = new_conn(&fname);

  let version: String = conn
    .get()
    .unwrap()
    .query_row("SELECT sqlite_version()", (), |row| row.get(0))
    .unwrap();
  println!("Sqlite v{version:?}");

  conn
    .get()
    .unwrap()
    .execute_batch(&format!("{PRAGMAS}\n{CREATE_TABLE_QUERY}"))
    .unwrap();

  {
    // Insert
    let start = Instant::now();
    let tasks: Vec<_> = (0..num_tasks())
      .into_iter()
      .map(|task| {
        let conn = conn.clone();

        std::thread::spawn(move || {
          for i in 0..N {
            let id = task * N + i;

            let c = conn.get().unwrap();
            let mut stmt = c.prepare_cached(BENCHMARK_QUERY).unwrap();
            if let Err(err) = stmt.execute((id, format!("{id}"))) {
              println!("Execute error: {err}");
            }
          }
        })
      })
      .collect();

    for t in tasks {
      t.join().unwrap();
    }

    println!(
      "[{NAME}]\n\tInserted {count} rows in {elapsed:?}",
      count = num_tasks() * N,
      elapsed = Instant::now() - start,
    );
  }

  let count: usize = conn
    .get()
    .unwrap()
    .query_row(COUNT_QUERY, (), |row| row.get(0))
    .unwrap();
  assert_eq!(count, num_tasks() * N);

  {
    // Read
    let start = Instant::now();
    let tasks: Vec<_> = (0..num_tasks())
      .into_iter()
      .map(|task| {
        let conn = conn.clone();

        std::thread::spawn(move || {
          for i in 0..N {
            let id = task * N + i;

            let c = conn.get().unwrap();
            let mut stmt = c
              .prepare_cached("SELECT * FROM person WHERE id = $1")
              .unwrap();
            let mut rows = stmt.query([id]).unwrap();
            rows.next().unwrap();
          }
        })
      })
      .collect();

    for t in tasks {
      t.join().unwrap();
    }

    println!(
      "[{NAME}]\n\tRead {count} rows in {elapsed:?}",
      count = num_tasks() * N,
      elapsed = Instant::now() - start,
    );
  }

  std::fs::remove_file(fname).unwrap();
}

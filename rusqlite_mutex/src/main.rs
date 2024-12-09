use constants::*;
use parking_lot::Mutex;
use rusqlite::Connection;
use std::sync::Arc;
use std::time::{Duration, Instant};

const NAME: &str = "RUSQLITE_MUTEX";

fn main() {
  let tmp_dir = tempfile::TempDir::new().unwrap();

  let fname = tmp_dir.path().join(format!("{NAME}.sqlite"));
  println!("rusqlite_mutex DB file: {fname:?}");

  let conn = Connection::open(fname.clone()).unwrap();
  conn.busy_timeout(Duration::from_secs(10)).unwrap();

  let version: String = conn
    .query_row("SELECT sqlite_version()", (), |row| row.get(0))
    .unwrap();
  println!("Sqlite v{version:?}");

  conn
    .execute_batch(&format!("{PRAGMAS}\n{CREATE_TABLE_QUERY}"))
    .unwrap();

  let c = Arc::new(Mutex::new(conn));

  {
    // Insert
    let start = Instant::now();
    let tasks: Vec<_> = (0..num_tasks())
      .into_iter()
      .map(|task| {
        let c = c.clone();
        std::thread::spawn(move || {
          for i in 0..N {
            let id = task * N + i;

            {
              let conn = c.lock();
              let mut stmt = conn.prepare_cached(BENCHMARK_QUERY).unwrap();
              stmt.execute((id, format!("{id}"))).unwrap();
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

  let count: usize = c
    .lock()
    .query_row(COUNT_QUERY, (), |row| row.get(0))
    .unwrap();
  assert_eq!(count, num_tasks() * N);

  {
    // Read
    let start = Instant::now();
    let tasks: Vec<_> = (0..num_tasks())
      .into_iter()
      .map(|task| {
        let c = c.clone();
        std::thread::spawn(move || {
          for i in 0..N {
            let id = task * N + i;

            {
              let conn = c.lock();
              let mut stmt = conn
                .prepare_cached("SELECT * FROM person WHERE id = $1")
                .unwrap();
              stmt.query_row([id], |_row| Ok(())).unwrap();
            }
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

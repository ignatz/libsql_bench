use constants::*;
use rusqlite::Connection;
use std::time::{Duration, Instant};

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

  {
    // Insert
    let start = Instant::now();
    let tasks: Vec<_> = (0..num_tasks())
      .into_iter()
      .map(|task| {
        let conn = Connection::open(fname.clone()).unwrap();
        conn.busy_timeout(Duration::from_secs(10)).unwrap();
        conn
          .busy_handler(Some(|_attempts| {
            std::thread::sleep(Duration::from_millis(50));
            return true;
          }))
          .unwrap();

        rt.spawn(async move {
          for i in 0..N {
            let id = task * N + i;

            let mut stmt = conn.prepare_cached(BENCHMARK_QUERY).unwrap();
            if let Err(err) = stmt.execute((id, format!("{id}"))) {
              println!("Execute error: {err}");
            }
          }
        })
      })
      .collect();

    rt.block_on(async {
      for t in tasks {
        t.await.unwrap();
      }
    });

    println!(
      "Inserted {count} rows in {elapsed:?}",
      count = num_tasks() * N,
      elapsed = Instant::now() - start,
    );
  }

  let count: usize = conn.query_row(COUNT_QUERY, (), |row| row.get(0)).unwrap();
  assert_eq!(count, num_tasks() * N);

  {
    // Read
    let start = Instant::now();
    let tasks: Vec<_> = (0..num_tasks())
      .into_iter()
      .map(|task| {
        let conn = Connection::open(fname.clone()).unwrap();

        rt.spawn(async move {
          for i in 0..N {
            let id = task * N + i;

            let mut stmt = conn
              .prepare_cached("SELECT * FROM person WHERE id = $1")
              .unwrap();
            let _ = stmt.query([id]).unwrap();
          }
        })
      })
      .collect();

    rt.block_on(async {
      for t in tasks {
        t.await.unwrap();
      }
    });

    println!(
      "Read {count} rows in {elapsed:?}",
      count = num_tasks() * N,
      elapsed = Instant::now() - start,
    );
  }

  std::fs::remove_file(fname).unwrap();
}

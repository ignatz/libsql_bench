use constants::*;
use rusqlite::Connection;
use std::time::Instant;

const NAME: &str = "RUSQLITE";

fn new_conn(path: &std::path::PathBuf) -> Connection {
  let conn = Connection::open(path).unwrap();
  conn.busy_timeout(constants::BUSY_TIMEOUT).unwrap();
  conn
    .busy_handler(Some(|_attempts| {
      std::thread::sleep(constants::BUSY_SLEEP);
      return true;
    }))
    .unwrap();
  return conn;
}

fn main() {
  let tmp_dir = tempfile::TempDir::new().unwrap();

  let fname = tmp_dir.path().join(format!("{NAME}.sqlite"));
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
        let conn = new_conn(&fname);

        std::thread::spawn(move || {
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

    for t in tasks {
      t.join().unwrap();
    }

    println!(
      "[{NAME}]\n\tInserted {count} rows in {elapsed:?}",
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

        std::thread::spawn(move || {
          for i in 0..N {
            let id = task * N + i;

            let mut stmt = conn
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

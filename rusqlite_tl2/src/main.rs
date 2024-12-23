use constants::*;
use rusqlite::Connection;
use std::sync::Arc;
use std::time::Instant;
use thread_local::ThreadLocal;

const NAME: &str = "RUSQLITE_TL2";

struct State {
  factory: Box<dyn Fn() -> rusqlite::Connection + Send + Sync>,
  conn: ThreadLocal<rusqlite::Connection>,
}

#[derive(Clone)]
struct Test {
  state: Arc<State>,
}

impl Test {
  pub fn new(f: impl Fn() -> rusqlite::Connection + Send + Sync + 'static) -> Self {
    return Self {
      state: Arc::new(State {
        factory: Box::new(f),
        conn: ThreadLocal::new(),
      }),
    };
  }

  fn conn(&self) -> &rusqlite::Connection {
    return self.state.conn.get_or(|| (self.state.factory)());
  }
}

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

  let fname_clone = fname.clone();
  let test = Test::new(move || new_conn(&fname_clone));

  {
    // Insert
    let start = Instant::now();
    let tasks: Vec<_> = (0..num_tasks())
      .into_iter()
      .map(|task| {
        let test = test.clone();

        std::thread::spawn(move || {
          for i in 0..N {
            let id = task * N + i;

            test
              .conn()
              .execute(BENCHMARK_QUERY, (id, format!("{id}")))
              .unwrap();
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
        let test = test.clone();

        std::thread::spawn(move || {
          for i in 0..N {
            let id = task * N + i;

            let g: usize = test
              .conn()
              .query_row("SELECT * FROM person WHERE id = $1", [id], |row| row.get(0))
              .unwrap();

            assert_eq!(id, g);
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

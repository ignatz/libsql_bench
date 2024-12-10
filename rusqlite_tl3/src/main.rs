use constants::*;
use parking_lot::Mutex;
use rusqlite::Connection;
use std::cell::{OnceCell, RefCell};
use std::sync::Arc;
use std::time::Instant;

const NAME: &str = "RUSQLITE_TL3";

const MAX_ID: usize = 4;
static ID: Mutex<usize> = Mutex::new(0);
thread_local! {
  static CELLS : Vec<OnceCell<RefCell<rusqlite::Connection>>> = std::iter::repeat_with(|| OnceCell::new()).take(MAX_ID).collect();
}

#[derive(Clone)]
struct Test {
  id: usize,
  fun: Arc<dyn Fn() -> rusqlite::Connection + Send + Sync>,
}

impl Test {
  pub fn new(f: impl Fn() -> rusqlite::Connection + Send + Sync + 'static) -> Self {
    let id = {
      let mut lock = ID.lock();
      let id = *lock;
      *lock += 1;
      id
    };
    if id >= MAX_ID {
      panic!("");
    }
    return Self {
      id,
      fun: Arc::new(f),
    };
  }

  #[inline]
  fn _call<T, F>(&self, f: F) -> rusqlite::Result<T>
  where
    F: FnOnce(&mut rusqlite::Connection) -> rusqlite::Result<T>,
  {
    return CELLS.with(|cells| {
      let c = cells[self.id].get_or_init(|| RefCell::new((self.fun)()));
      let conn: &mut rusqlite::Connection = &mut c.borrow_mut();
      return f(conn);
    });
  }

  // NOTE: A `query` would require an owned Rows type to avoid holding a ref.
  #[inline]
  pub fn query_row<T, P, F>(&self, sql: &str, params: P, f: F) -> rusqlite::Result<T>
  where
    P: rusqlite::Params,
    F: FnOnce(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
  {
    return self._call(move |conn| {
      let mut stmt = conn.prepare_cached(sql)?;
      return stmt.query_row(params, f);
    });
  }

  #[inline]
  pub fn execute<P>(&self, sql: &str, params: P) -> rusqlite::Result<usize>
  where
    P: rusqlite::Params,
  {
    return self._call(move |conn| {
      let mut stmt = conn.prepare_cached(sql)?;
      return stmt.execute(params);
    });
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

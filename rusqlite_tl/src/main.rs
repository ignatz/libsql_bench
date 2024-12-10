use constants::*;
use rusqlite::Connection;
use std::sync::Arc;
use std::time::{Duration, Instant};

const NAME: &str = "RUSQLITE_TL";

#[derive(Clone)]
struct Test {
  fun: Arc<dyn Fn() -> rusqlite::Connection + Send + Sync>,

  #[cfg(debug_assertions)]
  conn: std::sync::OnceLock<Arc<parking_lot::Mutex<rusqlite::Connection>>>,
}

/// NOTE: This API allows to switch between different execution modes, e.g. multi-conn in release
/// and shared conn for in-memory dbs in tests.
/// NOTE: We could even try to work with RwLock + stmt.readonly().
impl Test {
  pub fn new(f: Arc<dyn Fn() -> rusqlite::Connection + Send + Sync>) -> Self {
    return Self {
      fun: f,
      #[cfg(debug_assertions)]
      conn: std::sync::OnceLock::new(),
    };
  }

  #[cfg(not(debug_assertions))]
  #[inline]
  fn _call<T, F>(&self, f: F) -> rusqlite::Result<T>
  where
    F: FnOnce(&mut rusqlite::Connection) -> rusqlite::Result<T>,
  {
    use std::cell::{OnceCell, RefCell};
    thread_local! {
      static CELL : OnceCell<RefCell<rusqlite::Connection>> = OnceCell::new();
    }

    return CELL.with(|cell| {
      let c = cell.get_or_init(|| RefCell::new((self.fun)()));
      let conn: &mut rusqlite::Connection = &mut c.borrow_mut();
      return f(conn);
    });
  }

  #[cfg(debug_assertions)]
  #[inline]
  fn _call<T, F>(&self, f: F) -> rusqlite::Result<T>
  where
    F: FnOnce(&mut rusqlite::Connection) -> rusqlite::Result<T>,
  {
    let conn = self
      .conn
      .get_or_init(|| Arc::new(parking_lot::Mutex::new((self.fun)())));
    return f(&mut conn.lock());
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

fn new_conn(path: &std::path::PathBuf) -> rusqlite::Connection {
  let conn = Connection::open(path).unwrap();
  conn.busy_timeout(Duration::from_secs(10)).unwrap();
  conn
    .busy_handler(Some(|_attempts| {
      std::thread::sleep(Duration::from_millis(10));
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
        let fname = fname.clone();

        std::thread::spawn(move || {
          let test = Test::new(Arc::new(move || new_conn(&fname)));
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
        let fname = fname.clone();

        std::thread::spawn(move || {
          let test = Test::new(Arc::new(move || new_conn(&fname)));
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

use std::time::Duration;

pub const N: usize = 100000;

pub const BUSY_TIMEOUT: Duration = Duration::from_secs(10);
pub const BUSY_SLEEP: Duration = Duration::from_millis(10);

pub fn num_tasks() -> usize {
  std::thread::available_parallelism().unwrap().into()
}

// DO NOT DEPEND ON RUSQLITE TO AVOID LINKING BOTH: RUSQLITE & LIBSQL.
// pub fn new_conn(path: &std::path::PathBuf) -> rusqlite::Connection {
//   let conn = rusqlite::Connection::open(path).unwrap();
//   conn.busy_timeout(BUSY_TIMEOUT).unwrap();
//   conn
//     .busy_handler(Some(|_attempts| {
//       std::thread::sleep(BUSY_SLEEP);
//       return true;
//     }))
//     .unwrap();
//   return conn;
// }

pub const PRAGMAS: &str = r#"
    PRAGMA busy_timeout       = 10000;
    PRAGMA journal_mode       = WAL;
    PRAGMA journal_size_limit = 200000000;
    PRAGMA synchronous        = NORMAL;
    PRAGMA foreign_keys       = ON;
    PRAGMA temp_store         = MEMORY;
    PRAGMA cache_size         = -16000;
"#;

pub const CREATE_TABLE_QUERY: &str = r#"
    DROP TABLE IF EXISTS person;
    CREATE TABLE person (
      id         INTEGER PRIMARY KEY NOT NULL,
      name       TEXT NOT NULL,
      created    DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
    );
"#;

pub const BENCHMARK_QUERY: &str = "INSERT INTO person (id, name) VALUES ($1, $2)";

pub const COUNT_QUERY: &str = "SELECT COUNT(*) FROM person";

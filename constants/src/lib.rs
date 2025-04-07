use std::time::Duration;

pub const N: usize = 100000;

pub const BUSY_TIMEOUT: Duration = Duration::from_secs(10);
pub const BUSY_SLEEP: Duration = Duration::from_millis(10);

pub fn num_tasks() -> usize {
  std::thread::available_parallelism().unwrap().into()
}

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

pub trait SyncConnection {
  fn run_query(&self, sql: &str);
}

impl SyncConnection for rusqlite::Connection {
  fn run_query(&self, sql: &str) {
    let mut stmt = self.prepare_cached(sql).unwrap();
  }
}

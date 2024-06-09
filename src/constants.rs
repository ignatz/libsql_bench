pub const N: i64 = 1000;
pub const TASKS: i64 = 16;

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

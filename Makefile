all: rusqlite rusqlite_mutex libsql tokio_rusqlite

rusqlite:
	cargo run --release -p rusqlite_bench

rusqlite_mutex:
	cargo run --release -p rusqlite_mutex_bench

libsql:
	cargo run --release -p libsql_bench 

tokio_rusqlite:
	cargo run --release -p tokio_rusqlite_bench

.PHONY: all rusqlite rusqlite_mutex libsql tokio_rusqlite

all: rusqlite rusqlite_mutex libsql tokio_rusqlite

rusqlite:
	cargo run -p rusqlite_bench

rusqlite_mutex:
	cargo run -p rusqlite_mutex_bench

libsql:
	cargo run -p libsql_bench 

tokio_rusqlite:
	cargo run -p tokio_rusqlite_bench

.PHONY: all rusqlite rusqlite_mutex libsql tokio_rusqlite

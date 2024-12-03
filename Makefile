all: rusqlite rusqlite_mutex rusqlite_tl libsql libsql_rusqlite tokio_rusqlite

rusqlite:
	cargo run --release -p rusqlite_bench

rusqlite_mutex:
	cargo run --release -p rusqlite_mutex_bench

rusqlite_tl:
	cargo run --release -p rusqlite_thread_local_bench

libsql:
	cargo run --release -p libsql_bench 

libsql_rusqlite:
	cargo run --release -p libsql_rusqlite_bench 

tokio_rusqlite:
	cargo run --release -p tokio_rusqlite_bench

.PHONY: all rusqlite rusqlite_mutex rusqlite_tl libsql libsql_rusqlite tokio_rusqlite

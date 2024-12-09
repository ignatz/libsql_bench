all: rusqlite rusqlite_mutex rusqlite_tl rusqlite_tl2 libsql libsql_rusqlite tokio_rusqlite

rusqlite:
	cargo run --release -p rusqlite_bench 2> /dev/null

rusqlite_mutex:
	cargo run --release -p rusqlite_mutex_bench 2> /dev/null

rusqlite_tl:
	cargo run --release -p rusqlite_thread_local_bench 2> /dev/null

rusqlite_tl2:
	cargo run --release -p rusqlite_thread_local_bench2 2> /dev/null

libsql:
	cargo run --release -p libsql_bench 2> /dev/null

libsql_rusqlite:
	cargo run --release -p libsql_rusqlite_bench 2> /dev/null

tokio_rusqlite:
	cargo run --release -p tokio_rusqlite_bench 2> /dev/null

.PHONY: all rusqlite rusqlite_mutex rusqlite_tl rusqlite_tl2 libsql libsql_rusqlite tokio_rusqlite

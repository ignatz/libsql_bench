# Libsql Bench

I'm very excited about libsql both in terms of runtime capabilities as well as
properly async rust APIs.

Trying it out, I noticed:

* It doesn't support extensions, which is a known issue:
  https://github.com/tursodatabase/libsql/issues/402
  * Note that trying to load a sqlite database that was created by rusqlite and
    extensions loaded, crashed libsql. This one is probably on me?
* Reusing prepared statements requires explicit resets or otherwise the
  parameters don't bind correctly, see benchmark.
* Some SQL statements that do not typically return rows like PRAGMA need to be
  queried. Execution leads to "Execute returned rows".
* Even when running in local mode, i.e. plain sqlite3 mode w/o replication
  capabilities, it's appreciably slower:  *~270 times* or two orders of
  magnitude, not percent, at least based on this hopefully flawed benchmark.
  That said, I'm seeing only 80-90 insertions per second compared to 23k for
  rusqlite.

FWIW, building and running the benchmarks in debug vs release mode doesn't
change much in terms of relative performance.

I haven't (yet) looked into the details of the fork or the execution model of
the async client library. That said, I'll share some observations/speculations.

* I suspected lock contention at first, both on the DB as well as the
  library-level. However inserting sequentially basically leads to the same
  outcome.
* CPU utilization during execution is incredibly low ~15% of a single core,
  making me think that most of the time is spent waiting for I/O.

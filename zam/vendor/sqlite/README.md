# SQLite

This folder holds the [SQLite](https://sqlite.org) amalgamation, a whole SQL database engine in one C file. `std/sqlite` uses it.

- Version: 3.54.1.
- Source: <https://sqlite.org/2026/sqlite-amalgamation-3540100.zip>
- SHA3-256 of that archive: `92281c86e2957f9b9305890f9ac04204caa7b677cc868dc326d942689df28e30`
- License: public domain. See <https://sqlite.org/copyright.html>.

Only `sqlite3.c` and `sqlite3.h` are kept, unchanged. `zam` compiles them through a small wrapper file that turns off loadable extensions (`SQLITE_OMIT_LOAD_EXTENSION`), so programs don't need `libdl`, and keeps SQLite's default thread safety.

To update: replace both files with those of a newer amalgamation, update this file, and bump `native-c-N` in `zam/src/project.rs`.

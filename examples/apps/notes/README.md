# notes

`notes` is a REST service that stores notes in a SQLite database. Several threads handle requests at the same time, and each request opens its own database connection.

```bash
zam run examples/apps/notes
```

```text
notes: listening on http://127.0.0.1:8080 (database notes.db)
```

Then, from another terminal:

```bash
curl -X POST http://127.0.0.1:8080/notes -d '{"title": "milk", "body": "2 L"}'
```

```text
{"id": 1, "title": "milk", "body": "2 L", "created": "2026-10-24T13:44:11.678745+08:00", "updated": "2026-10-24T13:44:11.678745+08:00"}
```

| Request | What it does |
| --- | --- |
| `GET /notes` | Lists the notes. `?q=TEXT` keeps the notes whose title or body contains `TEXT`. |
| `POST /notes` | Adds a note from `{"title": "...", "body": "..."}`, and returns it with status 201. |
| `GET /notes/ID` | Returns one note. |
| `PUT /notes/ID` | Changes a note's title and body. |
| `DELETE /notes/ID` | Deletes a note, and returns status 204. |
| `GET /health` | Returns `ok`. |

Errors come back as `{"error": "..."}` with status 400 for a bad body, 404 for a missing note or route, 405 for a method a route doesn't take, and 500 for anything else.

Settings:

| Environment variable | Default | What it sets |
| --- | --- | --- |
| `NOTES_DB` | `notes.db` | The database file. It's created on first start. |
| `PORT` | `8080` | The port to listen on, on `127.0.0.1`. |

The program uses `std/http` for the server, `std/sqlite` to store the notes, `std/json` to read request bodies, and `std/datetime` for the times.

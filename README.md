# reindexer-rs
Rust bindings for Reindexer 5.9 with embedded (builtin) engine and optional cproto client.

## Requirements (Windows)
- MSVC toolchain (VS Build Tools / VS 2022)
- CMake and Ninja in `PATH`
- Rust toolchain `x86_64-pc-windows-msvc`
All native deps (Reindexer, LevelDB, Snappy) собираются из вендора автоматически.

## Подключение в свой проект
Через Git:
```
[dependencies]
reindexer = { git = "https://github.com/coinrust/reindexer-rs", branch = "master" }
```
Локально (если репо рядом):
```
[dependencies]
reindexer = { path = "../reindexer-rs" }
```

## Сборка этого репо
```
cargo build --release
cargo run --release --example hello
```
Бинарь примера: `target/release/examples/hello.exe`.

## Встраиваемый режим (embedded)
Работает без внешнего сервера, хранит данные по DSN `builtin://<путь>`.
```rust
use reindexer_rs::reindexer::Reindexer;

fn main() {
    let db = Reindexer::new();

    // каталог для данных
    let _ = std::fs::create_dir_all("./data/reindex");
    let ok = db.connet("builtin://./data/reindex");
    assert!(ok);

    let ns = "items";
    let ok = db.open_namespace(ns);
    assert!(ok);

    // одиночные индексы требуют явных json_paths
    let _ = db.add_index(ns, "id", "id", "hash", "int", true);   // PK
    let _ = db.add_index(ns, "fk_id", "fk_id", "hash", "int", false);
    let _ = db.add_index(ns, "id+fk_id", "id,fk_id", "tree", "composite", false);

    let ok = db.upsert(ns, r#"{"id":1234,"value":"value"}"#);
    assert!(ok);

    let (qr, ok) = db.select("SELECT * FROM items");
    assert!(ok);
    for s in qr.iter() {
        println!("{}", s);
    }
}
```

## Клиентский режим (cproto)
Требуется запущенный reindexer-server и DSN вида `cproto://host:port/db`.
```rust
use reindexer_rs::creindexer::CReindexer;

fn main() {
    let db = CReindexer::new();
    let ok = db.connect("cproto://127.0.0.1:6534/test_db");
    assert!(ok);

    let ns = "items";
    let _ = db.open_namespace(ns, true);
    let _ = db.add_index(ns, "id", "hash", "int", true);

    let _ = db.upsert(ns, r#"{"id":1,"value":"hello"}"#);

    let (qr, ok) = db.select("SELECT * FROM items");
    assert!(ok);
    for s in qr.iter() {
        println!("{}", s);
    }
}
```

## Быстрый тест
```
cargo run --release --example hello
```
Embedded-путь в примере: `builtin://./target/reindex_data`. Если хотите протестировать cproto, задайте `REINDEXER_CPROTO_DSN` или поправьте DSN в коде и поднимите сервер.
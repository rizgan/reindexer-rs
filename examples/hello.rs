
use reindexer_rs::creindexer::*;
use reindexer_rs::reindexer::*;

fn main() {
    // builtin
    let db = Reindexer::new();
    let data_dir = std::env::current_dir()
        .unwrap()
        .join("target")
        .join("reindex_data");
    let _ = std::fs::remove_dir_all(&data_dir);
    let _ = std::fs::create_dir_all(&data_dir);
    let dsn = "builtin://./target/reindex_data";
    let ok = db.connet(&dsn);
    println!("connect builtin: {} -> {}", dsn, ok);
    assert!(ok, "connect failed: {}", dsn);

    let ns = "items";
    let ok = db.open_namespace(ns);
        println!("open_namespace {} -> {}", ns, ok);
    assert!(ok);

    let ok = db.add_index(ns, "id", "id", "hash", "int", true);
    if !ok {
        println!("add_index id: already exists?");
    }

    let ok = db.add_index(ns, "fk_id", "fk_id", "hash", "int", false);
    if !ok {
        println!("add_index fk_id: already exists?");
    }

    let ok = db.add_index(ns, "id+fk_id", "id,fk_id", "tree", "composite", false);
    if !ok {
        println!("add_index id+fk_id: already exists?");
    }

    let item = r#"{"id":1234, "value" : "value"}"#;
    let ok = db.upsert(ns, item);
    assert!(ok);

    let item = r#"{"id":1235, "value" : "value"}"#;
    let ok = db.upsert(ns, item);
    assert!(ok);

    let (_, ok) = db.update_sql("UPDATE items SET ext = 'hello' WHERE id = 1235");
    assert!(ok);

    let (qr, ok) = db.select("SELECT * FROM items WHERE id = 1235");
    assert!(ok);

    let mut n = 0;
    for s in qr.iter() {
        println!("item: {}", s);
        n += 1;
        if n > 10 {
            break;
        }
    }

    // cproto (optional: set REINDEXER_CPROTO_DSN to enable)
    if let Ok(cproto_dsn) = std::env::var("REINDEXER_CPROTO_DSN") {
        let db = CReindexer::new();
        let ok = db.connect(&cproto_dsn);
        assert!(ok);

        let ns = "items";
        let ok = db.open_namespace(ns, true);
        assert!(ok);

        let ok = db.add_index(ns, "id", "hash", "int", true);
        if !ok {
            println!("cproto add_index id: already exists?");
        }

        let item = r#"{"id":1234, "value" : "value"}"#;
        let ok = db.upsert(ns, item);
        assert!(ok);

        let item = r#"{"id":1235, "value" : "value"}"#;
        let ok = db.upsert(ns, item);
        assert!(ok);

        let (qr, ok) = db.select("SELECT * FROM items");
        assert!(ok);

        for s in qr.iter() {
            println!("item: {}", s);
        }
    } else {
        println!("Skipping cproto example; set REINDEXER_CPROTO_DSN to run it");
    }
}

//fn test_unsafe() {
//    unsafe {
//        let db = ffi::re_client_new();
//
//        let dsn = CString::new(format!("{}", "cproto://127.0.0.1:6534/test_db")).unwrap();
//        let ok = ffi::re_client_connect(db, dsn.as_ptr());
//        println!("re_client_connect: {}", ok);
//
//        let ns = CString::new("items").unwrap();
//        let ok = ffi::re_client_open_namespace(db, ns.as_ptr(), true);
//        println!("re_client_open_namespace: {}", ok);
//
//        let name = CString::new("id").unwrap();
//        let index_type = CString::new("hash").unwrap();
//        let field_type = CString::new("int").unwrap();
//        let index_opts = ffi::index_opts_new();
//        ffi::index_opts_pk(index_opts);
//        let ok = ffi::re_client_add_index(db, ns.as_ptr(), name.as_ptr(), index_type.as_ptr(), field_type.as_ptr(), index_opts);
//        println!("re_client_add_index: {}", ok);
//        ffi::index_opts_destroy(index_opts);
//
//        let data = CString::new(r#"{"id":1234, "value" : "value"}"#).unwrap();
//        let ok = ffi::re_client_insert(db, ns.as_ptr(), data.as_ptr());
//        println!("re_client_insert: {}", ok);
//
//        let data = CString::new(r#"{"id":1235, "value" : "value"}"#).unwrap();
//        let ok = ffi::re_client_insert(db, ns.as_ptr(), data.as_ptr());
//        println!("re_client_insert: {}", ok);
//
//        let qr = ffi::re_client_query_results_new();
//        let query = CString::new("SELECT * FROM test_namespace").unwrap();
//        let ok = ffi::re_client_select(db, qr, query.as_ptr());
//        println!("re_client_select: {}", ok);
//
//        let count = ffi::re_client_query_results_count(qr);
//        println!("re_client_query_results_count: {}", count);
//
//        let it = ffi::re_client_query_results_iter(qr);
//
//        loop {
//            let ok = ffi::re_client_query_results_iter_next(it);
//            if !ok {
//                break;
//            }
//
//            let json = ffi::re_client_query_results_iter_get_json(it);
//            println!("re_client_query_results_iter_get_json {}", ok);
//            let cstr = CStr::from_ptr(json);
//            let s = cstr.to_string_lossy().into_owned();
//            println!("re_client_query_results_iter_get_json: {:?}", s);
//        }
//
//        ffi::re_client_query_results_iter_destroy(it);
//        println!("re_client_query_results_iter_destroy");
//
//        ffi::re_client_query_results_destroy(qr);
//        println!("re_client_query_results_destroy");
//
//        ffi::re_client_destroy(db);
//        println!("re_client_destroy");
//
//        //ffi::re_test();
//        //ffi::re_client_test();
//    }
//}

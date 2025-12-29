#include "client/queryresults.h"
#include "client/reindexer.h"
#include "core/item.h"
#include "core/reindexer.h"
#include "core/type_consts.h"
#include "tools/errors.h"
#include "tools/stringstools.h"
#include <cstring>
#include <iostream>
#include <string_view>
#include <vector>

using namespace reindexer;
using namespace std;

class CIterator {
public:
  CIterator(reindexer::client::QueryResults::Iterator start,
            reindexer::client::QueryResults::Iterator end)
      : current(start), end(end), iter(false) {}
  reindexer::client::QueryResults::Iterator current;
  reindexer::client::QueryResults::Iterator end;
  bool iter;
};

class Iterator {
public:
  Iterator(reindexer::QueryResults::Iterator start,
           reindexer::QueryResults::Iterator end)
      : current(start), end(end), iter(false) {}
  reindexer::QueryResults::Iterator current;
  reindexer::QueryResults::Iterator end;
  bool iter;
};

extern "C" {

void re_test() {}
void re_client_test() {}

reindexer::client::Reindexer *re_client_new() {
  reindexer::client::ReindexerConfig config;
  return new reindexer::client::Reindexer(config);
}

void re_client_destroy(reindexer::client::Reindexer *db) {
  delete db;
}

bool re_client_connect(reindexer::client::Reindexer *db, const char *dsn) {
  auto err = db->Connect(string(dsn));
  return err.ok();
}

bool re_client_open_namespace(reindexer::client::Reindexer *db, const char *ns, bool enabledStorage) {
  Error err = db->OpenNamespace(string(ns), StorageOpts().Enabled(enabledStorage));
  return err.ok();
}

IndexOpts *index_opts_new() { return new IndexOpts(); }
void index_opts_destroy(IndexOpts *indexOpts) { delete indexOpts; }
void index_opts_pk(IndexOpts *indexOpts) { indexOpts->PK(); }

bool re_client_add_index(reindexer::client::Reindexer *db, const char *ns, const char *name,
                         const char *indexType, const char *fieldType, IndexOpts *indexOpts) {
  auto err = db->AddIndex(ns, {name, indexType, fieldType, *indexOpts});
  return err.ok();
}

bool re_client_insert(reindexer::client::Reindexer *db, const char *ns, const char *data) {
  reindexer::client::Item item(db->NewItem(ns));
  Error err = item.FromJSON(data);
  if (!err.ok()) return false;
  err = db->Insert(ns, item);
  return err.ok();
}

bool re_client_update(reindexer::client::Reindexer *db, const char *ns, const char *data) {
  reindexer::client::Item item(db->NewItem(ns));
  Error err = item.FromJSON(data);
  if (!err.ok()) return false;
  err = db->Update(ns, item);
  return err.ok();
}

bool re_client_upsert(reindexer::client::Reindexer *db, const char *ns, const char *data) {
  reindexer::client::Item item(db->NewItem(ns));
  Error err = item.FromJSON(data);
  if (!err.ok()) return false;
  err = db->Upsert(ns, item);
  return err.ok();
}

bool re_client_delete(reindexer::client::Reindexer *db, const char *ns, const char *data) {
  reindexer::client::Item item(db->NewItem(ns));
  Error err = item.FromJSON(data);
  if (!err.ok()) return false;
  err = db->Delete(ns, item);
  return err.ok();
}

bool re_client_select(reindexer::client::Reindexer *db, reindexer::client::QueryResults *qr, const char *query) {
  Error err = db->ExecSQL(query, *qr);
  return err.ok();
}

reindexer::client::QueryResults *re_client_query_results_new() { return new reindexer::client::QueryResults(); }
void re_client_query_results_destroy(reindexer::client::QueryResults *qr) { delete qr; }
int re_client_query_results_count(reindexer::client::QueryResults *qr) { return qr->Count(); }
CIterator *re_client_query_results_iter(reindexer::client::QueryResults *qr) { return new CIterator(qr->begin(), qr->end()); }
bool re_client_query_results_iter_next(CIterator *it) {
  if (it->iter) {
    if (it->current == it->end) return false;
    ++(*it->current);
  } else {
    it->iter = true;
  }
  if (it->current == it->end) return false;
  return it->current.Status().ok();
}
char *re_client_query_results_iter_get_json(CIterator *it) {
  WrSerializer ser;
  it->current.GetJSON(ser, false);
  return strdup(ser.c_str());
}
void re_client_query_results_iter_destroy(CIterator *it) { delete it; }

reindexer::Reindexer *re_new() { return new reindexer::Reindexer(); }
void re_destroy(reindexer::Reindexer *db) { delete db; }

bool re_connect(reindexer::Reindexer *db, const char *dsn) {
  ConnectOpts opts;
  opts.AllowNamespaceErrors(true).WithStorageType(kStorageTypeOptLevelDB);
  auto err = db->Connect(dsn, opts);
  if (!err.ok()) {
    std::cerr << "re_connect failed: " << err.what() << std::endl;
  }
  return err.ok();
}

bool re_open_namespace(reindexer::Reindexer *db, const char *ns) {
  Error err = db->OpenNamespace(string(ns), StorageOpts().Enabled(true).CreateIfMissing(true));
  if (!err.ok()) {
    std::cerr << "re_open_namespace failed: " << err.what() << std::endl;
  }
  return err.ok();
}

bool re_add_index(reindexer::Reindexer *db, const char *ns, const char *name, const char *jsonPaths,
                  const char *indexType, const char *fieldType, IndexOpts *indexOpts) {
  std::vector<std::string> hashParts;
  split(jsonPaths, ",", false, hashParts);
  if (!hashParts.empty()) {
    JsonPaths jPaths(hashParts.begin(), hashParts.end());
    IndexDef indexDef(name, jPaths, indexType, fieldType, *indexOpts);
    auto err = db->AddIndex(ns, indexDef);
    if (!err.ok()) {
      std::cerr << "re_add_index failed: " << err.what() << std::endl;
    }
    return err.ok();
  } else {
    IndexDef indexDef(name, indexType, fieldType, *indexOpts);
    auto err = db->AddIndex(ns, indexDef);
    if (!err.ok()) {
      std::cerr << "re_add_index failed: " << err.what() << std::endl;
    }
    return err.ok();
  }
}

bool re_add_index_from_json(reindexer::Reindexer *db, const char *ns, const char *indexDefJson) {
  auto maybeDef = IndexDef::FromJSON(std::string_view(indexDefJson));
  if (!maybeDef) return false;
  return db->AddIndex(ns, *maybeDef).ok();
}

bool re_insert(reindexer::Reindexer *db, const char *ns, const char *data) {
  reindexer::Item item(db->NewItem(ns));
  Error err = item.FromJSON(data);
  if (!err.ok()) return false;
  err = db->Insert(ns, item);
  return err.ok();
}

bool re_update(reindexer::Reindexer *db, const char *ns, const char *data) {
  reindexer::Item item(db->NewItem(ns));
  Error err = item.FromJSON(data);
  if (!err.ok()) return false;
  err = db->Update(ns, item);
  return err.ok();
}

bool re_upsert(reindexer::Reindexer *db, const char *ns, const char *data) {
  reindexer::Item item(db->NewItem(ns));
  Error err = item.FromJSON(data);
  if (!err.ok()) return false;
  err = db->Upsert(ns, item);
  return err.ok();
}

bool re_delete(reindexer::Reindexer *db, const char *ns, const char *data) {
  reindexer::Item item(db->NewItem(ns));
  Error err = item.FromJSON(data);
  if (!err.ok()) return false;
  err = db->Delete(ns, item);
  return err.ok();
}

bool re_select(reindexer::Reindexer *db, reindexer::QueryResults *qr, const char *query) {
  Error err = db->Select(Query::FromSQL(query), *qr);
  return err.ok();
}

bool re_update_sql(reindexer::Reindexer *db, reindexer::QueryResults *qr, const char *query) {
  try {
    Query q = Query::FromSQL(query);
    return db->Update(q, *qr).ok();
  } catch (const Error &) {
    return false;
  }
}

reindexer::QueryResults *re_query_results_new() { return new reindexer::QueryResults(); }
void re_query_results_destroy(reindexer::QueryResults *qr) { delete qr; }
int re_query_results_count(reindexer::QueryResults *qr) { return qr->Count(); }
Iterator *re_query_results_iter(reindexer::QueryResults *qr) { return new Iterator(qr->begin(), qr->end()); }
bool re_query_results_iter_next(Iterator *it) {
  if (it->iter) {
    if (it->current == it->end) return false;
    ++(*it->current);
  } else {
    it->iter = true;
  }
  if (it->current == it->end) return false;
  return it->current.Status().ok();
}
char *re_query_results_iter_get_json(Iterator *it) {
  WrSerializer ser;
  it->current.GetJSON(ser, false);
  return strdup(ser.c_str());
}
void re_query_results_iter_destroy(Iterator *it) { delete it; }

} // extern "C"
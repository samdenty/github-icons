mod schema;
pub use diesel::prelude::*;
pub use schema::*;

use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel_migrations::embed_migrations;
use lru::LruCache;
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
  error::Error,
  sync::{Arc, Mutex},
};
use thread_local::ThreadLocal;

static CONNECTION_MANAGER: Lazy<Pool<ConnectionManager<SqliteConnection>>> =
  Lazy::new(|| establish_connection().unwrap());

static DB: Lazy<ThreadLocal<PooledConnection<ConnectionManager<SqliteConnection>>>> =
  Lazy::new(|| ThreadLocal::new());

static REGEX_CACHE: Lazy<Arc<Mutex<LruCache<String, Arc<Regex>>>>> =
  Lazy::new(|| Arc::new(Mutex::new(LruCache::new(16))));

embed_migrations!("./migrations");

pub mod functions {
  use diesel::sql_types::*;

  sql_function!(fn regexp(regex: Text, text: Text) -> Bool);
}

fn establish_connection() -> Result<Pool<ConnectionManager<SqliteConnection>>, Box<dyn Error>> {
  let database_url = "file:test.db";
  let manager = ConnectionManager::<SqliteConnection>::new(database_url);
  let pool = Pool::builder().build(manager)?;
  let conn = pool.get()?;

  embedded_migrations::run(&conn)?;

  functions::regexp::register_impl(&conn, move |regex: String, text: String| {
    let mut cache = REGEX_CACHE.lock().unwrap();
    let re = cache.get(&regex).cloned().unwrap_or_else(|| {
      let re = Arc::new(Regex::new(&regex).unwrap());
      cache.put(regex, re.clone());
      re
    });

    re.is_match(&text)
  })
  .unwrap();

  Ok(pool)
}

pub fn db() -> &'static PooledConnection<ConnectionManager<SqliteConnection>> {
  DB.get_or(|| CONNECTION_MANAGER.get().unwrap())
}

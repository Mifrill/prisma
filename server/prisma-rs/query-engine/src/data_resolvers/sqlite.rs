mod data_resolver;
mod service;

use chrono::{DateTime, Utc};
use prisma_common::PrismaResult;
use prisma_models::prelude::*;
use r2d2_sqlite3::SqliteConnectionManager;
use std::collections::HashSet;
use sqlite::{Statement, Type, Value};

use crate::SERVER_ROOT;

type Connection = r2d2::PooledConnection<SqliteConnectionManager>;
type Pool = r2d2::Pool<SqliteConnectionManager>;

pub struct Sqlite {
    pool: Pool,
    test_mode: bool,
}

impl Sqlite {
    /// Creates a new SQLite pool connected into local memory. By querying from
    /// different databases, it will try to create them to
    /// `$SERVER_ROOT/db/db_name` if they do not exists yet.
    pub fn new(connection_limit: u32, test_mode: bool) -> PrismaResult<Sqlite> {
        let pool = r2d2::Pool::builder()
            .max_size(connection_limit)
            .build(SqliteConnectionManager::memory())?;

        Ok(Sqlite { pool, test_mode })
    }

    /// Will create a new file if it doesn't exist. Otherwise loads db/db_name
    /// from the SERVER_ROOT.
    fn create_database(conn: &mut Connection, db_name: &str) -> PrismaResult<()> {
        let stmt = conn.prepare("PRAGMA database_list")?;
        let mut cursor = stmt.cursor();

        let mut databases: HashSet<String> = HashSet::new();

        while let Ok(Some(row)) = cursor.next() {
            databases.insert(row[0].as_string().unwrap().into());
        }

        if !databases.contains(db_name) {
            let path = format!("{}/db/{}.db", *SERVER_ROOT, db_name);
            let mut stmt = conn.prepare("ATTACH DATABASE ? AS ?")?;
            stmt.bind(1, path.as_str())?;
            stmt.bind(2, db_name)?;

            // TODO(katharina) This should be replaced with a better `execute` call in
            //                 the sqlite crate!
            stmt.next().unwrap();
        }

        Ok(())
    }

    /// Take a new connection from the pool and create the database if it
    /// doesn't exist yet.
    fn with_connection<F, T>(&self, db_name: &str, f: F) -> PrismaResult<T>
    where
        F: FnOnce(&Connection) -> PrismaResult<T>,
    {
        let mut conn = self.pool.get()?;
        Self::create_database(&mut conn, db_name)?;

        let result = f(&conn);

        if self.test_mode {
            let mut stmt = conn.prepare("ATTACH DATABASE ? AS ?")?;
            stmt.bind(1, db_name);

            // TODO(katharina) This should be replaced with a better `execute` call in
            //                 the sqlite crate!
            stmt.next().unwrap();
        }

        result
    }

    /// Converter function to wrap the limited set of types in SQLite to a
    /// richer PrismaValue.
    fn fetch_value(typ: TypeIdentifier, value: &Statement, i: usize) -> PrismaValue {
        // let result = match typ {
        //     TypeIdentifier::String => row.get_checked(i).map(|val| PrismaValue::String(val)),
        //     TypeIdentifier::GraphQLID => row.get_checked(i).map(|val| PrismaValue::GraphqlId(val)),
        //     TypeIdentifier::UUID => row.get_checked(i).map(|val| PrismaValue::Uuid(val)),
        //     TypeIdentifier::Int => row.get_checked(i).map(|val| PrismaValue::Int(val)),
        //     TypeIdentifier::Boolean => row.get_checked(i).map(|val| PrismaValue::Boolean(val)),
        //     TypeIdentifier::Enum => row.get_checked(i).map(|val| PrismaValue::Enum(val)),
        //     TypeIdentifier::Json => row.get_checked(i).map(|val| PrismaValue::Json(val)),
        //     TypeIdentifier::DateTime => row.get_checked(i).map(|ts: i64| {
        //         let nsecs = ((ts % 1000) * 1_000_000) as u32;
        //         let secs = (ts / 1000) as i64;
        //         let naive = chrono::NaiveDateTime::from_timestamp(secs, nsecs);
        //         let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);

        //         PrismaValue::DateTime(datetime.to_rfc3339())
        //     }),
        //     TypeIdentifier::Relation => panic!("We should not have a Relation here!"),
        //     TypeIdentifier::Float => row.get_checked(i).map(|val: f64| PrismaValue::Float(val as f32)),
        // };

        // result.unwrap_or_else(|e| match e {
        //     rusqlite::Error::InvalidColumnType(_, rusqlite::types::Type::Null) => PrismaValue::Null,
        //     _ => panic!(e),
        // })

        unimplemented!()
    }
}

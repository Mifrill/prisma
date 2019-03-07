use crate::data_resolvers::{DataResolver, SelectQuery, Sqlite};

use prisma_common::PrismaResult;
use prisma_models::prelude::*;
use prisma_query::visitor::{self, Visitor};

use sqlite::State;

impl DataResolver for Sqlite {
    fn select_nodes(&self, query: SelectQuery) -> PrismaResult<(Vec<Vec<PrismaValue>>, Vec<String>)> {
        let db_name = query.db_name;
        let query_ast = query.query_ast;
        let names = query.selected_fields.names_of_scalar_non_list_fields();
        let fields = query.selected_fields.fields;

        self.with_connection(&db_name, |conn| {
            let (query_sql, params) = dbg!(visitor::Sqlite::build(query_ast));
            let mut stmt = conn.prepare(&query_sql)?;
            params
                .into_iter()
                .zip(1..)
                .map(|(p, i)| stmt.bind(i, p))
                .fold(Ok(()), |acc, x| match (acc, x) {
                    (Ok(_), x) => x,
                    (x, _) => x,
                })?;

            let nodes_iter = while let State::Row = stmt.next().unwrap() {
                fields
                    .iter()
                    .enumerate()
                    .map(|(i, field)| Self::fetch_value(field.type_identifier, &stmt, i))
                    .collect()
            };

            // let nodes_iter = stmt.query_map(&params, |row| {
            //     fields
            //         .iter()
            //         .enumerate()
            //         .map(|(i, field)| Self::fetch_value(field.type_identifier, &row, i))
            //         .collect()
            // })?;

            let mut nodes = Vec::new();
            for node in nodes_iter {
                nodes.push(node?);
            }

            Ok(dbg!((nodes, names)))
        })
    }
}

use std::marker::PhantomData;

use sqlx::{AnyPool, FromRow};

use crate::{error::OrmError, model::Model, value::OrmValue};

/// Fluent query builder — mirrors the Lucid ORM API.
///
/// Created via `Model::query()`.  Consumed on execution.
pub struct QueryBuilder<M: Model> {
    table:      &'static str,
    conditions: Vec<String>,
    bindings:   Vec<OrmValue>,
    order:      Option<String>,
    limit_val:  Option<i64>,
    offset_val: Option<i64>,
    _m:         PhantomData<M>,
}

impl<M: Model> QueryBuilder<M> {
    pub(crate) fn new(table: &'static str) -> Self {
        Self {
            table,
            conditions: Vec::new(),
            bindings:   Vec::new(),
            order:      None,
            limit_val:  None,
            offset_val: None,
            _m:         PhantomData,
        }
    }

    // ── FILTERING ────────────────────────────────────────────────────────────

    /// Raw WHERE fragment.  Use `?` as the placeholder.
    ///
    /// Multiple calls are joined with `AND`.
    ///
    /// ```ignore
    /// User::query().where_raw("status = ?").bind("active")
    /// ```
    pub fn where_raw(mut self, condition: impl Into<String>) -> Self {
        self.conditions.push(condition.into());
        self
    }

    /// Bind a value to the next `?` placeholder.
    pub fn bind(mut self, value: impl Into<OrmValue>) -> Self {
        self.bindings.push(value.into());
        self
    }

    /// Shorthand: `column = ?` with the given value.
    pub fn where_eq(self, column: &str, value: impl Into<OrmValue>) -> Self {
        self.where_raw(format!("{} = ?", column)).bind(value)
    }

    /// Shorthand: `column != ?` with the given value.
    pub fn where_not_eq(self, column: &str, value: impl Into<OrmValue>) -> Self {
        self.where_raw(format!("{} != ?", column)).bind(value)
    }

    /// Shorthand: `column > ?`.
    pub fn where_gt(self, column: &str, value: impl Into<OrmValue>) -> Self {
        self.where_raw(format!("{} > ?", column)).bind(value)
    }

    /// Shorthand: `column < ?`.
    pub fn where_lt(self, column: &str, value: impl Into<OrmValue>) -> Self {
        self.where_raw(format!("{} < ?", column)).bind(value)
    }

    /// Shorthand: `column IS NULL`.
    pub fn where_null(self, column: &str) -> Self {
        self.where_raw(format!("{} IS NULL", column))
    }

    /// Shorthand: `column IS NOT NULL`.
    pub fn where_not_null(self, column: &str) -> Self {
        self.where_raw(format!("{} IS NOT NULL", column))
    }

    // ── ORDERING / PAGING ────────────────────────────────────────────────────

    /// ORDER BY `column direction`.
    pub fn order_by(mut self, column: &str, direction: &str) -> Self {
        self.order = Some(format!("{} {}", column, direction));
        self
    }

    /// LIMIT n.
    pub fn limit(mut self, n: i64) -> Self {
        self.limit_val = Some(n);
        self
    }

    /// OFFSET n.
    pub fn offset(mut self, n: i64) -> Self {
        self.offset_val = Some(n);
        self
    }

    /// Convenience: set LIMIT + OFFSET from 1-based `page` and `per_page`.
    pub fn paginate(self, page: i64, per_page: i64) -> Self {
        let offset = (page.max(1) - 1) * per_page;
        self.limit(per_page).offset(offset)
    }

    // ── EXECUTION ────────────────────────────────────────────────────────────

    fn build_select_sql(&self) -> String {
        let mut sql = format!("SELECT * FROM {}", self.table);

        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.conditions.join(" AND "));
        }
        if let Some(ord) = &self.order {
            sql.push_str(&format!(" ORDER BY {}", ord));
        }
        if let Some(n) = self.limit_val {
            sql.push_str(&format!(" LIMIT {}", n));
        }
        if let Some(n) = self.offset_val {
            sql.push_str(&format!(" OFFSET {}", n));
        }
        sql
    }

    fn build_count_sql(&self) -> String {
        let mut sql = format!("SELECT COUNT(*) FROM {}", self.table);
        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.conditions.join(" AND "));
        }
        sql
    }

    /// Fetch all matching rows.
    pub async fn all(self, pool: &AnyPool) -> Result<Vec<M>, OrmError> {
        let sql = self.build_select_sql();
        let mut q = sqlx::query_as::<sqlx::Any, M>(&sql);
        for v in &self.bindings {
            q = bind_orm_value(q, v);
        }
        Ok(q.fetch_all(pool).await?)
    }

    /// Fetch the first matching row.
    pub async fn first(self, pool: &AnyPool) -> Result<Option<M>, OrmError> {
        let sql = {
            let mut s = self.build_select_sql();
            // Avoid double LIMIT
            if self.limit_val.is_none() {
                s.push_str(" LIMIT 1");
            }
            s
        };
        let mut q = sqlx::query_as::<sqlx::Any, M>(&sql);
        for v in &self.bindings {
            q = bind_orm_value(q, v);
        }
        Ok(q.fetch_optional(pool).await?)
    }

    /// Count matching rows.
    pub async fn count(self, pool: &AnyPool) -> Result<i64, OrmError> {
        let sql = self.build_count_sql();
        let mut q = sqlx::query_as::<sqlx::Any, (i64,)>(&sql);
        for v in &self.bindings {
            q = bind_orm_value_tuple(q, v);
        }
        let (n,) = q.fetch_one(pool).await?;
        Ok(n)
    }

    /// Delete matching rows.
    pub async fn delete(self, pool: &AnyPool) -> Result<u64, OrmError> {
        let mut sql = format!("DELETE FROM {}", self.table);
        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.conditions.join(" AND "));
        }
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for v in &self.bindings {
            q = bind_orm_value_query(q, v);
        }
        let result = q.execute(pool).await?;
        Ok(result.rows_affected())
    }
}

// ── Helpers to bind OrmValue to different query types ────────────────────────

pub(crate) fn bind_orm_value<'q, M>(
    q: sqlx::query::QueryAs<'q, sqlx::Any, M, sqlx::any::AnyArguments<'q>>,
    v: &'q OrmValue,
) -> sqlx::query::QueryAs<'q, sqlx::Any, M, sqlx::any::AnyArguments<'q>>
where
    M: for<'r> FromRow<'r, sqlx::any::AnyRow> + Send + Unpin,
{
    match v {
        OrmValue::Int(n)   => q.bind(*n),
        OrmValue::Float(f) => q.bind(*f),
        OrmValue::Text(s)  => q.bind(s.as_str()),
        OrmValue::Bool(b)  => q.bind(*b),
        OrmValue::Null     => q.bind(Option::<String>::None),
    }
}

pub(crate) fn bind_orm_value_tuple<'q>(
    q: sqlx::query::QueryAs<'q, sqlx::Any, (i64,), sqlx::any::AnyArguments<'q>>,
    v: &'q OrmValue,
) -> sqlx::query::QueryAs<'q, sqlx::Any, (i64,), sqlx::any::AnyArguments<'q>> {
    match v {
        OrmValue::Int(n)   => q.bind(*n),
        OrmValue::Float(f) => q.bind(*f),
        OrmValue::Text(s)  => q.bind(s.as_str()),
        OrmValue::Bool(b)  => q.bind(*b),
        OrmValue::Null     => q.bind(Option::<String>::None),
    }
}

pub(crate) fn bind_orm_value_query<'q>(
    q: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>,
    v: &'q OrmValue,
) -> sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>> {
    match v {
        OrmValue::Int(n)   => q.bind(*n),
        OrmValue::Float(f) => q.bind(*f),
        OrmValue::Text(s)  => q.bind(s.as_str()),
        OrmValue::Bool(b)  => q.bind(*b),
        OrmValue::Null     => q.bind(Option::<String>::None),
    }
}

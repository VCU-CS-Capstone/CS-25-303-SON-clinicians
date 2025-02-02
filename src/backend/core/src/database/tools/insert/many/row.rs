use std::{borrow::Cow, fmt::Debug};

use sqlx::{Encode, Postgres, Type};

use crate::database::tools::{ColumnType, FormatSql, HasArguments};

use super::{InsertManyBuilder, InsertManyValue};

pub struct InsertRowBuilder<'query, 'args, C: ColumnType> {
    /// If None then it will use DEFAULT
    columns_to_insert: Vec<(C, Option<usize>)>,
    query: &'query mut InsertManyBuilder<'args, C>,
}
impl<'query, 'args, C: ColumnType + PartialEq + Clone> InsertRowBuilder<'query, 'args, C> {
    pub(super) fn new(query: &'query mut InsertManyBuilder<'args, C>) -> Self {
        Self {
            columns_to_insert: Vec::with_capacity(query.columns_to_insert.len()),
            query,
        }
    }
    /// Insert a value into the query
    pub fn insert<T>(&mut self, column: C, value: T) -> &mut Self
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        let index = self.query.push_argument(value);
        self.columns_to_insert.push((column, Some(index)));
        self
    }

    pub fn insert_value(&mut self, column: C, value: InsertManyValue) -> &mut Self {
        if !self.query.shared_values.contains(&value) {
            panic!("Value is not registered to be used in the query");
        }
        self.columns_to_insert.push((column, Some(value.0)));
        self
    }
    /// Will check if option is Some and insert the value if it is
    ///
    /// This will allow for the database to just use its default value if the option is None
    ///
    /// If you want to insert a NULL value use `insert` with `None`
    pub fn insert_option<T>(&mut self, column: C, value: Option<T>) -> &mut Self
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        if let Some(value) = value {
            let index = self.query.push_argument(value);

            self.columns_to_insert.push((column, Some(index)));
        } else {
            self.columns_to_insert.push((column, None));
        }
        self
    }
    pub(super) fn finish(self) -> InsertRow<C> {
        let mut values = Vec::with_capacity(self.query.columns_to_insert.len());
        for column in self.query.columns_to_insert.iter() {
            let mut value = None;
            for (c, has_value) in &self.columns_to_insert {
                if c == column {
                    value = *has_value;
                    break;
                }
            }
            // Any values that do not have a value will be set to DEFAULT
            values.push((column.clone(), value));
        }
        InsertRow(values)
    }
}

pub struct InsertRow<C>(Vec<(C, Option<usize>)>);
impl<C> Debug for InsertRow<C>
where
    C: ColumnType + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("InsertRow").field(&self.0).finish()
    }
}
impl<C: ColumnType> FormatSql for InsertRow<C> {
    fn format_sql(&self) -> Cow<'_, str> {
        let mut string_builder = "(".to_owned();
        let mut iter = self.0.iter().peekable();
        while let Some((_, value)) = iter.next() {
            if let Some(value) = value {
                string_builder.push_str(&format!("${}", value));
            } else {
                string_builder.push_str("DEFAULT");
            }
            if iter.peek().is_some() {
                string_builder.push_str(", ");
            }
        }
        string_builder.push(')');
        Cow::Owned(string_builder)
    }
}

pub struct InsertRowOrderedBuilder<'query, 'args, C: ColumnType> {
    /// If None then it will use DEFAULT
    columns_to_insert: Vec<Option<usize>>,
    query: &'query mut InsertManyBuilder<'args, C>,
}
impl<'query, 'args, C: ColumnType + PartialEq + Clone> InsertRowOrderedBuilder<'query, 'args, C> {
    pub(super) fn new(query: &'query mut InsertManyBuilder<'args, C>) -> Self {
        Self {
            columns_to_insert: Vec::with_capacity(query.columns_to_insert.len()),
            query,
        }
    }
    /// Insert a value into the query
    pub fn insert<T>(&mut self, value: T) -> &mut Self
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        let index = self.query.push_argument(value);
        self.columns_to_insert.push(Some(index));
        self
    }
    /// Inserts a previously registered value into the row
    pub fn insert_value(&mut self, value: InsertManyValue) -> &mut Self {
        if !self.query.shared_values.contains(&value) {
            panic!("Value is not registered to be used in the query");
        }
        self.columns_to_insert.push(Some(value.0));
        self
    }
    /// Will check if option is Some and insert the value if it is
    ///
    /// This will allow for the database to just use its default value if the option is None
    ///
    /// If you want to insert a NULL value use `insert` with `None`
    pub fn insert_option<T>(&mut self, value: Option<T>) -> &mut Self
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        if let Some(value) = value {
            let index = self.query.push_argument(value);

            self.columns_to_insert.push(Some(index));
        } else {
            self.columns_to_insert.push(None);
        }
        self
    }
    pub(super) fn finish(self) -> InsertRow<C> {
        let mut column_values = Vec::with_capacity(self.query.columns_to_insert.len());
        let mut values = self.columns_to_insert.into_iter();
        for column in self.query.columns_to_insert.iter() {
            let value = values.next().flatten();
            // Any values that do not have a value will be set to DEFAULT
            column_values.push((column.clone(), value));
        }
        InsertRow(column_values)
    }
}

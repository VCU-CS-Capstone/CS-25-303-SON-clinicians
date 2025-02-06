use std::fmt::Display;

use derive_more::derive::From;
use sqlx::{Encode, Postgres, Type};

use super::{
    AndOr, ColumnType, DynColumn, FormatSql, HasArguments, QueryBuilderValue,
    QueryBuilderValueType, SQLComparison,
};

pub trait WhereableTool<'args>: HasArguments<'args> + Sized {
    fn where_column<SC, F>(&mut self, column: SC, where_: F) -> &mut Self
    where
        SC: ColumnType + 'static,
        F: FnOnce(WhereBuilder<'_, 'args, Self>) -> WhereComparison,
    {
        let builder = WhereBuilder::new(self, column);
        let where_comparison = where_(builder);

        self.push_where_comparison(where_comparison);

        self
    }
    /// Adds a where clause to check if the column is equal to the value
    fn where_equals<SC, T>(&mut self, column: SC, value: T) -> &mut Self
    where
        SC: ColumnType + 'static,
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        self.where_column(column, |builder| builder.equals(value).build())
    }

    /// Adds a where clause to check if the column is like the value
    fn where_like<SC, T>(&mut self, column: SC, value: T) -> &mut Self
    where
        SC: ColumnType + 'static,
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        self.where_column(column, |builder| builder.like(value).build())
    }
    /// Required to push the where comparison to the query
    fn where_is_not_null<SC>(&mut self, column: SC) -> &mut Self
    where
        SC: ColumnType + 'static,
    {
        self.where_column(column, |builder| builder.is_not_null().build())
    }
    /// Adds a where clause to check if the column is null
    fn where_is_null<SC>(&mut self, column: SC) -> &mut Self
    where
        SC: ColumnType + 'static,
    {
        self.where_column(column, |builder| builder.is_null().build())
    }

    /// Required to push the where comparison to the query
    ///
    /// The internal structure will be a Vec<WhereComparison>
    ///
    /// Each are concatenated with an AND
    fn push_where_comparison(&mut self, comparison: WhereComparison);
}
#[derive(Debug)]
pub enum WhereValue {
    CompareValue {
        comparison: SQLComparison,
        value: QueryBuilderValue,
    },
    NotNull,
    Null,
}

pub struct WhereBuilder<'query, 'args, A>
where
    A: HasArguments<'args>,
{
    args: &'query mut A,
    column: DynColumn,
    value: Option<WhereValue>,
    phantoms: std::marker::PhantomData<&'args ()>,
}
impl<'query, 'args, A> WhereBuilder<'query, 'args, A>
where
    A: HasArguments<'args>,
{
    pub fn new<SC>(args: &'query mut A, column: SC) -> Self
    where
        SC: ColumnType + 'static,
    {
        Self {
            args,
            column: column.dyn_column(),
            value: None,
            phantoms: std::marker::PhantomData,
        }
    }
    pub fn is_not_null(self) -> Self {
        Self {
            value: Some(WhereValue::NotNull),
            ..self
        }
    }
    pub fn is_null(self) -> Self {
        Self {
            value: Some(WhereValue::Null),
            ..self
        }
    }

    pub fn compare<V>(mut self, comparison: SQLComparison, value: V) -> Self
    where
        V: QueryBuilderValueType<'args> + 'args,
    {
        let value = value.process(self.args);
        self.value = Some(WhereValue::CompareValue { comparison, value });
        self
    }
    pub fn equals<V>(self, value: V) -> Self
    where
        V: QueryBuilderValueType<'args> + 'args,
    {
        self.compare(SQLComparison::Equals, value)
    }

    pub fn like<T>(self, value: T) -> Self
    where
        T: 'args + Encode<'args, Postgres> + Type<Postgres>,
    {
        self.compare(SQLComparison::Like, value)
    }

    pub fn build(self) -> WhereComparison {
        self.into()
    }
    pub fn then<'s: 'query, F, SC>(self, and_or: AndOr, then_column: SC, f: F) -> WhereComparison
    where
        SC: ColumnType + 'static,
        F: FnOnce(WhereBuilder<'_, 'args, A>) -> WhereComparison,
    {
        let Self {
            args,
            column,
            mut value,
            ..
        } = self;
        let builder = WhereBuilder {
            args,
            column: then_column.dyn_column(),
            value: None,
            phantoms: std::marker::PhantomData,
        };
        let then = f(builder);

        WhereComparison {
            column,
            value: value.take().expect("Value not set"),
            then: Some((and_or, Box::new(then))),
        }
    }
    pub fn and<F, SC>(self, column: SC, f: F) -> WhereComparison
    where
        SC: ColumnType + 'static,
        F: FnOnce(WhereBuilder<'_, 'args, A>) -> WhereComparison,
    {
        self.then(AndOr::And, column, f)
    }
    pub fn or<F, SC>(self, column: SC, f: F) -> WhereComparison
    where
        SC: ColumnType + 'static,
        F: FnOnce(WhereBuilder<'_, 'args, A>) -> WhereComparison,
    {
        self.then(AndOr::Or, column, f)
    }
}
#[derive(Debug)]
pub struct WhereComparison {
    column: DynColumn,
    value: WhereValue,
    then: Option<(AndOr, Box<WhereComparison>)>,
}

impl<'query, 'args, A> From<WhereBuilder<'query, 'args, A>> for WhereComparison
where
    A: HasArguments<'args>,
{
    fn from(builder: WhereBuilder<'query, 'args, A>) -> Self {
        WhereComparison {
            column: builder.column,
            value: builder.value.expect("Value not set"),
            then: None,
        }
    }
}
#[derive(Debug, From)]
pub struct MultipleWhereFormatter<'w>(&'w WhereComparison);
impl Display for MultipleWhereFormatter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.0)
    }
}
impl Display for WhereComparison {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            WhereValue::CompareValue { comparison, value } => write!(
                f,
                "{} {} {}",
                self.column.formatted_column(),
                comparison,
                value.format_sql()
            )?,
            WhereValue::NotNull => {
                write!(f, "{} IS NOT NULL", self.column.formatted_column())?;
            }

            WhereValue::Null => {
                write!(f, "{} IS NULL", self.column.formatted_column())?;
            }
        }
        if let Some((and_or, then)) = &self.then {
            write!(f, " {} {}", and_or.as_ref(), then)?;
        }
        Ok(())
    }
}
pub fn format_where(comparison: &[WhereComparison]) -> String {
    if comparison.is_empty() {
        return String::new();
    }
    if comparison.len() == 1 {
        // TODO: Remove the extra parentheses
        return comparison[0].to_string();
    }
    comparison
        .iter()
        .map(|comparison| MultipleWhereFormatter(comparison).to_string())
        .collect::<Vec<_>>()
        .join(" AND ")
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]

    use crate::database::{
        prelude::*,
        tools::{
            testing::TestTableColumn,
            where_sql::{format_where, WhereBuilder},
        },
    };

    use super::WhereComparison;

    pub struct TestParentQuery<'args> {
        arguments: Option<<Postgres as sqlx::Database>::Arguments<'args>>,
    }
    impl<'args> HasArguments<'args> for TestParentQuery<'args> {
        fn take_arguments_or_error(&mut self) -> <Postgres as sqlx::Database>::Arguments<'args> {
            self.arguments.take().expect("Arguments already taken")
        }

        fn borrow_arguments_or_error(
            &mut self,
        ) -> &mut <Postgres as sqlx::Database>::Arguments<'args> {
            self.arguments.as_mut().expect("Arguments already taken")
        }
    }
    #[test]
    pub fn where_format() {
        let part_one = {
            let then = Box::new(WhereComparison {
                column: TestTableColumn::Name.dyn_column(),
                value: WhereValue::CompareValue {
                    comparison: SQLComparison::Equals,
                    value: 2.into(),
                },
                then: None,
            });
            WhereComparison {
                column: TestTableColumn::Id.dyn_column(),
                value: WhereValue::CompareValue {
                    comparison: SQLComparison::Equals,
                    value: 1.into(),
                },
                then: Some((AndOr::And, then)),
            }
        };

        let part_two = {
            let then = Box::new(WhereComparison {
                column: TestTableColumn::Age.dyn_column(),
                value: WhereValue::CompareValue {
                    comparison: SQLComparison::Equals,
                    value: 3.into(),
                },
                then: None,
            });
            WhereComparison {
                column: TestTableColumn::Email.dyn_column(),
                value: WhereValue::CompareValue {
                    comparison: SQLComparison::Equals,
                    value: QueryBuilderFunction::now().into(),
                },
                then: Some((AndOr::Or, then)),
            }
        };
        let where_part = vec![part_one, part_two];
        let result = format_where(&where_part);
        println!("{}", result);
    }
    #[test]
    pub fn test_builder() {
        let mut query = TestParentQuery {
            arguments: Some(Default::default()),
        };
        let where_part = WhereBuilder::new(&mut query, TestTableColumn::Id)
            .equals(1)
            .and(TestTableColumn::Name, |builder| builder.equals(2).build());
        let result = format_where(&[where_part]);
        println!("{}", result);
    }
}

use std::borrow::Cow;

use derive_more::From;
use sqlx::Postgres;

use super::{ColumnType, DynColumn, FormatSql, HasArguments};
pub trait QueryBuilderValueType<'args> {
    fn process<A>(self, args: &mut A) -> QueryBuilderValue
    where
        Self: 'args,
        A: HasArguments<'args>;
}
impl<'args, T> QueryBuilderValueType<'args> for T
where
    T: 'args + sqlx::Encode<'args, Postgres> + sqlx::Type<Postgres>,
{
    fn process<A>(self, args: &mut A) -> QueryBuilderValue
    where
        Self: 'args,
        A: HasArguments<'args>,
    {
        let index = args.push_argument(self);
        QueryBuilderValue::ArgumentIndex(index)
    }
}
impl<'args> QueryBuilderValueType<'args> for QueryBuilderFunction {
    fn process<A>(self, _args: &mut A) -> QueryBuilderValue
    where
        Self: 'args,
        A: HasArguments<'args>,
    {
        QueryBuilderValue::Function(self)
    }
}
impl<'args> QueryBuilderValueType<'args> for DynColumn {
    fn process<A>(self, _args: &mut A) -> QueryBuilderValue
    where
        Self: 'args,
        A: HasArguments<'args>,
    {
        QueryBuilderValue::Column(self)
    }
}

#[derive(Debug, From)]
pub enum QueryBuilderValue {
    ArgumentIndex(usize),
    Function(QueryBuilderFunction),
    Column(DynColumn),
}
impl FormatSql for QueryBuilderValue {
    fn format_sql(&self) -> Cow<'_, str> {
        match self {
            QueryBuilderValue::ArgumentIndex(index) => Cow::Owned(format!("${}", index)),
            QueryBuilderValue::Function(function) => function.format_sql(),
            QueryBuilderValue::Column(column) => column.formatted_column(),
        }
    }
}
#[derive(Debug)]
pub struct QueryBuilderFunction {
    function_name: &'static str,
}
impl QueryBuilderFunction {
    pub fn now() -> Self {
        Self {
            function_name: "NOW",
        }
    }
}

impl FormatSql for QueryBuilderFunction {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        Cow::Owned(format!("{}()", self.function_name))
    }
}

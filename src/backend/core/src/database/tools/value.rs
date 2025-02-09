use std::borrow::Cow;

use derive_more::From;
use sqlx::Postgres;

use super::{ColumnType, DynColumn, FormatSql, HasArguments};
pub trait QueryBuilderValueType<'args> {
    fn process<A>(self, args: &mut A) -> QueryBuilderValue
    where
        Self: 'args,
        A: HasArguments<'args>;

    fn with_function(self, function_name: &'static str) -> FunctionCall<'args, Self>
    where
        Self: Sized,
    {
        FunctionCall {
            function_name,
            query_builder_value: self,
            phantom: std::marker::PhantomData,
        }
    }
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
pub struct FunctionCall<'args, V: QueryBuilderValueType<'args>> {
    function_name: &'static str,
    query_builder_value: V,
    phantom: std::marker::PhantomData<&'args V>,
}
impl<'args, V> QueryBuilderValueType<'args> for FunctionCall<'args, V>
where
    V: QueryBuilderValueType<'args>,
{
    fn process<A>(self, args: &mut A) -> QueryBuilderValue
    where
        Self: 'args,
        A: HasArguments<'args>,
    {
        let value = self.query_builder_value.process(args);
        QueryBuilderValue::Function(QueryBuilderFunction {
            function_name: self.function_name,
            params: vec![value],
        })
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
#[derive(Debug, Default)]
pub struct QueryBuilderFunction {
    function_name: &'static str,
    params: Vec<QueryBuilderValue>,
}
impl QueryBuilderFunction {
    pub fn now() -> Self {
        Self {
            function_name: "NOW",
            params: Vec::new(),
        }
    }
}

impl FormatSql for QueryBuilderFunction {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        let params = self
            .params
            .iter()
            .map(|param| param.format_sql())
            .collect::<Vec<_>>()
            .join(", ");
        Cow::Owned(format!("{}({params})", self.function_name))
    }
}

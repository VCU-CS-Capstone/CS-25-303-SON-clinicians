use std::borrow::Cow;

use crate::database::tools::{
    AndOr, ColumnType, DynColumn, FormatSql, HasArguments, QueryBuilderValue,
    QueryBuilderValueType, SQLComparison, SQLCondition,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}
impl FormatSql for JoinType {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        match self {
            JoinType::Inner => "INNER JOIN".into(),
            JoinType::Left => "LEFT JOIN".into(),
            JoinType::Right => "RIGHT JOIN".into(),
            JoinType::Full => "FULL JOIN".into(),
        }
    }
}
#[derive(Debug)]
pub struct OnCondition {
    left: QueryBuilderValue,
    value: SQLCondition,
    then: Option<(AndOr, Box<OnCondition>)>,
}
impl FormatSql for OnCondition {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        let mut sql = self.left.format_sql().into_owned();
        sql.push_str(" ");
        sql.push_str(&self.value.format_sql());
        if let Some((and_or, then)) = &self.then {
            sql.push(' ');
            sql.push_str(&and_or.format_sql());
            sql.push(' ');
            sql.push_str(&then.format_sql());
        }
        Cow::Owned(sql)
    }
}
#[derive(Debug)]
pub struct Join {
    pub join_type: JoinType,
    pub table: &'static str,
    pub on: OnCondition,
    pub columns_to_select: Vec<DynColumn>,
}
impl FormatSql for Join {
    fn format_sql(&self) -> std::borrow::Cow<'_, str> {
        let mut sql = format!("{} {}", self.join_type.format_sql(), self.table);
        sql.push_str(" ON ");
        sql.push_str(&self.on.format_sql());

        Cow::Owned(sql)
    }
}
pub struct JoinBuilder<'query, 'args, A>
where
    A: HasArguments<'args>,
{
    join_type: JoinType,
    table: &'static str,
    args: &'query mut A,
    pub columns_to_select: Vec<DynColumn>,

    phantoms: std::marker::PhantomData<&'args ()>,
}
impl<'query, 'args, A> JoinBuilder<'query, 'args, A>
where
    A: HasArguments<'args>,
{
    pub fn new(args: &'query mut A, table_name: &'static str, join: JoinType) -> Self {
        Self {
            args,
            join_type: join,
            table: table_name,
            phantoms: std::marker::PhantomData,
            columns_to_select: Vec::new(),
        }
    }
    pub fn select<C>(mut self, column: C) -> Self
    where
        C: ColumnType + 'static,
    {
        self.columns_to_select.push(column.dyn_column());
        self
    }
    pub fn select_many<C>(mut self, columns: Vec<C>) -> Self
    where
        C: ColumnType + 'static,
    {
        self.columns_to_select
            .extend(columns.into_iter().map(|c| c.dyn_column()));
        self
    }
    pub fn on<F>(self, f: F) -> Join
    where
        F: FnOnce(OnConditionBuilder<'query, 'args, A>) -> OnCondition,
    {
        let builder = OnConditionBuilder {
            args: self.args,
            left: None,
            value: None,
            phantoms: std::marker::PhantomData,
        };
        let on = f(builder);
        Join {
            join_type: self.join_type,
            table: self.table,
            on,
            columns_to_select: self.columns_to_select,
        }
    }
}

pub struct OnConditionBuilder<'query, 'args, A>
where
    A: HasArguments<'args>,
{
    args: &'query mut A,
    left: Option<QueryBuilderValue>,
    value: Option<SQLCondition>,
    phantoms: std::marker::PhantomData<&'args ()>,
}

impl<'query, 'args, A: HasArguments<'args>> OnConditionBuilder<'query, 'args, A> {
    pub fn new(args: &'query mut A) -> Self {
        Self {
            args,
            left: None,
            value: None,
            phantoms: std::marker::PhantomData,
        }
    }

    pub fn compare<L, R>(mut self, left: L, comparison: SQLComparison, right: R) -> Self
    where
        L: QueryBuilderValueType<'args> + 'args,
        R: QueryBuilderValueType<'args> + 'args,
    {
        let left = left.process(self.args);
        let right = right.process(self.args);
        self.left = Some(left);
        self.value = Some(SQLCondition::CompareValue {
            comparison,
            value: right,
        });
        self
    }
    /// Compare two values. This can be either a column or a value
    ///
    /// Please note because trait limitations you will need to call .dyn_column() on the column
    pub fn equals<L, R>(self, left: L, right: R) -> Self
    where
        L: QueryBuilderValueType<'args> + 'args,
        R: QueryBuilderValueType<'args> + 'args,
    {
        self.compare(left, SQLComparison::Equals, right)
    }
    pub fn build(self) -> OnCondition {
        OnCondition {
            left: self.left.expect("Left side of ON condition not set"),
            value: self.value.expect("Value of ON condition not set"),
            then: None,
        }
    }
}

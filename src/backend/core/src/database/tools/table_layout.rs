use std::{borrow::Cow, fmt::Debug};

use super::{FormatSql, FunctionCallColumn};

pub trait TableType {
    type Columns: ColumnType;
    fn table_name() -> &'static str
    where
        Self: Sized;
}
#[derive(Debug)]
pub struct DynColumn(Box<dyn ColumnType + Send + Sync>);
impl DynColumn {
    pub fn new<C>(column: C) -> Self
    where
        C: ColumnType + Send + Sync + 'static,
    {
        Self(Box::new(column))
    }
}
impl ColumnType for DynColumn {
    fn column_name(&self) -> &'static str {
        self.0.column_name()
    }
    fn dyn_column(self) -> DynColumn
    where
        Self: Sized + Send + Sync + 'static,
    {
        self
    }

    fn format_column_with_prefix(&self, prefix: Option<&str>) -> Cow<'static, str> {
        self.0.format_column_with_prefix(prefix)
    }
    fn formatted_column(&self) -> Cow<'static, str> {
        self.0.formatted_column()
    }
}

pub trait ColumnType: Debug + Send + Sync {
    fn column_name(&self) -> &'static str;
    /// Should return the `{table_name}.{column_name}` format
    fn formatted_column(&self) -> Cow<'static, str> {
        Cow::Borrowed(self.column_name())
    }
    fn format_column_with_prefix(&self, prefix: Option<&str>) -> Cow<'static, str> {
        if let Some(prefix) = prefix {
            Cow::Owned(format!("{}.{}", prefix, self.column_name()))
        } else {
            Cow::Borrowed(self.column_name())
        }
    }

    fn lower(&self) -> FunctionCallColumn<Self>
    where
        Self: Sized + Copy,
    {
        FunctionCallColumn {
            function_name: "LOWER",
            column: *self,
        }
    }
    fn upper(&self) -> FunctionCallColumn<Self>
    where
        Self: Sized + Copy,
    {
        FunctionCallColumn {
            function_name: "UPPER",
            column: *self,
        }
    }
    fn dyn_column(self) -> DynColumn
    where
        Self: Sized + Send + Sync + 'static,
    {
        DynColumn::new(self)
    }
}
impl<C> FormatSql for C
where
    C: ColumnType,
{
    fn format_sql(&self) -> Cow<'_, str> {
        self.formatted_column()
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnFormatWithPrefix<'prefix, 'column, C> {
    column: &'column C,
    prefix: Option<&'prefix str>,
}
impl<C> ColumnType for ColumnFormatWithPrefix<'_, '_, C>
where
    C: ColumnType,
{
    fn column_name(&self) -> &'static str {
        self.column.column_name()
    }
    fn formatted_column(&self) -> Cow<'static, str> {
        self.column.format_column_with_prefix(self.prefix)
    }
    fn format_column_with_prefix(&self, prefix: Option<&str>) -> Cow<'static, str> {
        self.column
            .format_column_with_prefix(prefix.or(self.prefix))
    }
}
impl<'prefix, 'column, C> ColumnFormatWithPrefix<'prefix, 'column, C>
where
    C: ColumnType,
{
    pub fn new(column: &'column C, prefix: Option<&'prefix str>) -> Self {
        Self { column, prefix }
    }
}
pub trait AllColumns {
    fn all() -> Vec<Self>
    where
        Self: Sized;

    fn all_dyn() -> Vec<DynColumn>
    where
        Self: Sized + ColumnType + 'static,
    {
        Self::all().into_iter().map(|c| c.dyn_column()).collect()
    }
}

pub trait TableQuery {
    type Table: TableType;

    fn columns() -> Vec<<Self::Table as TableType>::Columns>
    where
        Self: Sized;
}

impl<T> TableQuery for T
where
    T: TableType,
    T::Columns: AllColumns,
{
    type Table = T;

    fn columns() -> Vec<<Self::Table as TableType>::Columns> {
        <Self::Table as TableType>::Columns::all()
    }
}

/// A workaround for https://github.com/rust-lang/rust/issues/20041 being unstable
pub mod rust_unstable_workaround {
    use super::{ColumnType, TableQuery, TableType};

    pub trait HasColumns {
        type Columns: ColumnType;
        fn columns() -> Vec<Self::Columns>
        where
            Self: Sized;
    }
    pub trait HasTableName {
        fn table_name() -> &'static str
        where
            Self: Sized;
    }

    impl<T: TableQuery> HasTableName for T {
        fn table_name() -> &'static str {
            <T::Table as TableType>::table_name()
        }
    }

    impl<T: TableQuery> HasColumns for T {
        type Columns = <T::Table as TableType>::Columns;

        fn columns() -> Vec<Self::Columns> {
            T::columns()
        }
    }
}

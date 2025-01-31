#![allow(dead_code)]
//! Testing Utilities for the Database Tooling
use crate::database::prelude::*;
#[derive(Debug, Clone, Columns)]
pub struct TestTable {
    pub id: i32,
    pub name: String,
    pub age: i32,
    pub email: String,
}
impl TableType for TestTable {
    type Columns = TestTableColumn;
    fn table_name() -> &'static str {
        "test_table"
    }
}

#[derive(Debug, Clone, Columns)]
pub struct AnotherTable {
    pub id: i32,
    pub name: String,
    pub age: i32,
    pub email: String,
}
impl TableType for AnotherTable {
    type Columns = TestTableColumn;
    fn table_name() -> &'static str {
        "another_table"
    }
}
/// A fake arguments holder for testing
///
/// Use [`FakeArgumentsHolder::default`] to create a new instance.
pub struct FakeArgumentsHolder<'args> {
    pub arguments: Option<PostgresArguments<'args>>,
}
impl Default for FakeArgumentsHolder<'_> {
    fn default() -> Self {
        Self {
            arguments: Some(Default::default()),
        }
    }
}
impl HasArguments<'_> for FakeArgumentsHolder<'_> {
    fn take_arguments_or_error(&mut self) -> PostgresArguments<'_> {
        self.arguments.take().expect("Arguments already taken")
    }
    fn borrow_arguments_or_error(&mut self) -> &mut PostgresArguments<'_> {
        self.arguments.as_mut().expect("Arguments already taken")
    }
}

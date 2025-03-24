use pg_extended_sqlx_queries::prelude::*;

use serde::{Deserialize, Serialize};
use strum::EnumIs;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, EnumIs, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "value")]
pub enum ArrayQuery<T> {
    /// Must contain all of the following values.
    ///
    /// Order does not matter.
    ///
    /// Uses SQL `ARRAY[1,4,3] @> ARRAY[3,1]`
    ///
    /// This basically says a column has all the values in the given array
    Contains(Vec<T>),
    /// The two arrays are equal
    ///
    /// Ignoring order
    ///
    /// Using `(array1 <@ array2 and array1 @> array2)`
    Equals(Vec<T>),

    /// Atleast one of the following values must be present
    ///
    /// Uses SQL `ARRAY[1,4,3] && ARRAY[3,1]`
    ///
    /// This basically says a column has an intersection with the given values
    ContainsAny(Vec<T>),
}
impl<T> ArrayQuery<T> {
    pub fn len(&self) -> usize {
        match self {
            ArrayQuery::Contains(values) => values.len(),
            ArrayQuery::Equals(values) => values.len(),
            ArrayQuery::ContainsAny(values) => values.len(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl<'args, T> ArrayQuery<T>
where
    T: ExprType<'args> + 'args,
    Vec<T>: ExprType<'args> + 'args + Clone,
{
    pub fn filter(
        self,
        column: impl ColumnType + Copy + 'static,
    ) -> FilterConditionBuilder<'args, DynExpr<'args>, DynExpr<'args>> {
        match self {
            ArrayQuery::Contains(values) => {
                column.dyn_column().array_contains(values).dyn_expression()
            }
            // TODO: We need to be able to use rebound items...
            ArrayQuery::Equals(values) if values.len() > 1 => column
                .dyn_column()
                .array_contained_by(values.clone())
                .and(column.dyn_column().array_contains(values))
                .dyn_expression(),
            ArrayQuery::Equals(values) => column.dyn_column().equals(values).dyn_expression(),
            ArrayQuery::ContainsAny(values) => {
                column.dyn_column().array_overlap(values).dyn_expression()
            }
        }
    }
}

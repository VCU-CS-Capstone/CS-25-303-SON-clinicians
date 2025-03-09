use crate::database::prelude::*;
use crate::user::Permissions;

#[derive(Debug, Clone, PartialEq, Eq, FromRow, TableType)]
#[table(name = "role_permissions")]
pub struct RolePermissions {
    pub id: i32,
    pub role_id: i32,
    pub permission: Permissions,
    pub created_at: DateTime<FixedOffset>,
}
#[derive(Debug, Clone, PartialEq, Eq, FromRow, TableType)]
#[table(name = "roles")]
pub struct Roles {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<FixedOffset>,
}
impl Roles {
    pub async fn get_role_by_name(name: &str, db: &PgPool) -> DBResult<Option<Roles>> {
        SelectQueryBuilder::new(Roles::table_name())
            .select_all()
            .filter(RolesColumn::Name.equals(name))
            .query_as()
            .fetch_optional(db)
            .await
            .map_err(DBError::from)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, FromRow, TableType)]
#[table(name = "user_roles")]
pub struct UserRoles {
    pub id: i32,
    pub user_id: i32,
    pub role_id: i32,
    pub created_on: DateTime<FixedOffset>,
}
impl UserRoles {
    pub async fn add_user_role(
        user_id: i32,
        role_id: i32,
        db: impl Executor<'_, Database = sqlx::Postgres>,
    ) -> DBResult<()> {
        InsertQueryBuilder::new(UserRoles::table_name())
            .insert(UserRolesColumn::UserId, user_id)
            .insert(UserRolesColumn::RoleId, role_id)
            .on_conflict_do_nothing(ConflictTarget::Constraint("unique_user_id_role_id"))
            .query()
            .execute(db)
            .await?;

        Ok(())
    }
}

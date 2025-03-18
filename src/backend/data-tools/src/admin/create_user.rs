use argon2::{
    Argon2,
    password_hash::{PasswordHasher, Salt, SaltString},
};
use clap::Args;
use cs25_303_core::database::user::{
    new::{NewUser, create_or_update_user_password},
    roles::{Roles, UserRoles},
};
use rand::{TryRngCore, rngs::OsRng};

use crate::config::DataToolConfig;
#[derive(Debug, Clone, Args)]
pub struct CreateUserCommand {
    #[clap(long)]
    pub username: String,
    #[clap(long)]
    pub first_name: String,
    #[clap(long)]
    pub last_name: String,
    #[clap(long)]
    pub email: String,
    #[clap(long)]
    pub password: Option<String>,
    #[clap(long)]
    pub role: Option<String>,
}

impl CreateUserCommand {
    pub async fn run(self, config: DataToolConfig) -> anyhow::Result<()> {
        let database = cs25_303_core::database::connect(config.database.try_into()?, true).await?;

        let new_user = NewUser {
            username: self.username,
            first_name: self.first_name,
            last_name: self.last_name,
            email: self.email,
        };

        if new_user.check_if_email_is_in_use(&database).await? {
            anyhow::bail!("Email is already in use");
        }

        if new_user.check_if_username_is_in_use(&database).await? {
            anyhow::bail!("Username is already in use");
        }
        let mut transaction = database.begin().await?;
        let user = new_user.insert_return_user(&mut *transaction).await?;
        println!("User created with id {}", user.id);
        if let Some(password) = self.password {
            let mut bytes = [0u8; Salt::RECOMMENDED_LENGTH];
            OsRng.try_fill_bytes(&mut bytes).unwrap();
            let salt = SaltString::encode_b64(&bytes).expect("Failed to generate salt");

            let argon2 = Argon2::default();

            let password = argon2.hash_password(password.as_ref(), &salt);
            match password {
                Ok(password) => {
                    let password_value = password.to_string();
                    create_or_update_user_password(user.id, &password_value, &mut *transaction)
                        .await?;
                    println!("Password set for user {}", user.username);
                }
                Err(e) => {
                    transaction.rollback().await?;

                    anyhow::bail!("Failed to hash password: {}", e);
                }
            }
        }
        if let Some(role) = self.role {
            let role = Roles::get_role_by_name(&role, &database).await?;
            match role {
                Some(role) => {
                    println!(
                        "Adding role {} with id {} to user {}",
                        role.name, role.id, user.username
                    );
                    UserRoles::add_user_role(user.id, role.id, &mut *transaction).await?;
                }
                None => {
                    transaction.rollback().await?;
                    anyhow::bail!("Role not found");
                }
            }
        }
        transaction.commit().await?;
        Ok(())
    }
}

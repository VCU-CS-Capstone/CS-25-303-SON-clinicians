use axum::{
    extract::{Path, Query, State},
    response::Response,
    routing::{get, post},
    Json,
};
use chrono::Local;
use cs25_303_core::database::{
    tools::{PaginatedResponse, QueryTool, SimpleUpdateQueryBuilder, TableType, WhereableTool},
    user::{does_email_exist, does_username_exist, new::NewUser, User, UserColumn, UserType},
};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};
use utoipa::{OpenApi, ToSchema};

use crate::{
    app::{authentication::Authentication, error::InternalError, SiteState},
    utils::{not_found_response, ok_json_response, ConflictResponse, LookupPage},
};

#[derive(OpenApi)]
#[openapi(
    paths(all_users, new_user, update_user),
    components(schemas(PaginatedResponse<User>, User, NewUser, UpdateUser, ConflictResponse))
)]
pub struct AdminUserAPI;

pub fn admin_user_routes() -> axum::Router<SiteState> {
    axum::Router::new()
        .route("/all", get(all_users))
        .route("/new", post(new_user))
        .route("/:id/update", post(update_user))
}

#[utoipa::path(
    get,
    path = "/all",
    params(
        ("page_size" = i32, Query, description = "Number of items per page"),
        ("page_number" = i32, Query, description = "Page number"),
    ),
    responses(
        (status = 200, description = "Participants Found", body = PaginatedResponse<User>),
        (status = 401, description = "Not Authorized to access all users"),
    ),
    security(
        ("session" = ["ManageUsers"]),
        ("api_token" = ["ManageUsers"]),
    )
)]
#[instrument]
pub async fn all_users(
    State(site): State<SiteState>,
    Query(page): Query<LookupPage>,
    auth: Authentication,
) -> Result<Response, InternalError> {
    let LookupPage {
        page_size,
        page_number,
    } = page;
    let users = User::get_all_paginated(&site.database, page_size, page_number).await?;

    ok_json_response(users)
}

#[utoipa::path(
    post,
    path = "/new",
    request_body(content = NewUser, content_type = "application/json"),
    responses(
        (status = 200, description = "Successfully Created a new user", body = User),
        (status = 401, description = "Not Authorized to create a new user"),
        (status = 409, description = "Username or email already in use", body = ConflictResponse)
    ),
    security(
        ("session" = ["ManageUsers"]),
        ("api_token" = ["ManageUsers"]),
    )
)]
#[instrument]
pub async fn new_user(
    State(site): State<SiteState>,
    auth: Authentication,
    Json(new_user): Json<NewUser>,
) -> Result<Response, InternalError> {
    if new_user.check_if_username_is_in_use(&site.database).await? {
        debug!(?new_user.username, "Username already in use");
        return ConflictResponse::from("username").response();
    }

    if new_user.check_if_email_is_in_use(&site.database).await? {
        debug!(?new_user.email, "Email already in use");
        return ConflictResponse::from("email").response();
    }

    let user = new_user.insert_return_user(&site.database).await?;

    ok_json_response(user)
}
#[derive(Debug, Default, Serialize, Deserialize, ToSchema)]
#[serde(default)]
pub struct UpdateUser {
    /// The new username of the user.
    #[serde(with = "crate::utils::serde_sanitize_string")]
    pub username: Option<String>,
    /// The new email of the user.
    #[serde(with = "crate::utils::serde_sanitize_string")]
    pub email: Option<String>,
    /// The new first name of the user.
    #[serde(with = "crate::utils::serde_sanitize_string")]
    pub first_name: Option<String>,
    /// The new last name of the user.
    #[serde(with = "crate::utils::serde_sanitize_string")]
    pub last_name: Option<String>,
}

#[utoipa::path(
    post,
    path = "/{id}/update",
    request_body(content = UpdateUser, content_type = "application/json"),
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Successfully Created a new user", body = User),
        (status = 401, description = "Not Authorized to update user"),
        (status = 404, description = "User not found"),
        (status = 409, description = "Username or email already in use", body = ConflictResponse)
    ),
    security(
        ("session" = ["ManageUsers"]),
        ("api_token" = ["ManageUsers"]),
    )
)]
#[instrument]
pub async fn update_user(
    State(site): State<SiteState>,
    Path(id): Path<i32>,
    auth: Authentication,
    Json(update): Json<UpdateUser>,
) -> Result<Response, InternalError> {
    let Some(user_to_update) = User::get_by_id(id, &site.database).await? else {
        return not_found_response();
    };
    let UpdateUser {
        username,
        email,
        first_name,
        last_name,
    } = update;

    let mut update = SimpleUpdateQueryBuilder::new(User::table_name());
    update
        .where_equals(UserColumn::Id, id)
        .set(UserColumn::UpdatedAt, Local::now().fixed_offset());
    if let Some(username) = username {
        if !user_to_update.username.eq_ignore_ascii_case(&username) {
            if does_username_exist(&username, &site.database).await? {
                debug!(?username, "Username already in use");
                return ConflictResponse::from("username").response();
            }
            update.set(UserColumn::Username, username);
        }
    }
    if let Some(email) = email {
        if !user_to_update.email.eq_ignore_ascii_case(&email) {
            if does_email_exist(&email, &site.database).await? {
                debug!(?email, "Email already in use");
                return ConflictResponse::from("email").response();
            }
            update.set(UserColumn::Email, email);
        }
    }
    if let Some(first_name) = first_name {
        update.set(UserColumn::FirstName, first_name);
    }
    if let Some(last_name) = last_name {
        update.set(UserColumn::LastName, last_name);
    }

    update.query().execute(&site.database).await?;
    let user = User::get_by_id(id, &site.database).await?;
    ok_json_response(user)
}

use axum::response::{IntoResponse, Response};
use cs25_303_core::user::Permissions;
use serde::Serialize;
use std::marker::PhantomData;
use thiserror::Error;
use utoipa::{
    ToSchema,
    openapi::{example::ExampleBuilder, response, schema::RefBuilder},
};

use crate::{
    app::authentication::PermissionCheck,
    utils::{ErrorReason, ResponseBuilder, api_error_response::APIErrorResponse},
};

pub const MISSING_PERMISSION_MESSAGE: &str = "Missing Permission";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Error)]
#[error("Missing Permission: {0}")]
pub struct MissingPermission(pub Permissions);

impl utoipa::IntoResponses for MissingPermission {
    fn responses() -> std::collections::BTreeMap<String, utoipa::openapi::RefOr<response::Response>>
    {
        let missing_permission_response = APIErrorResponse::<Permissions, ()>::name();
        response::ResponsesBuilder::new()
            .responses_from_iter([(
                "403",
                utoipa::openapi::ResponseBuilder::new()
                    .description("Missing Permission")
                    .content(
                        "application/json",
                        utoipa::openapi::content::ContentBuilder::new()
                            .schema(Some(
                                RefBuilder::new()
                                    .ref_location_from_schema_name(missing_permission_response),
                            ))
                            .into(),
                    )
                    .build(),
            )])
            .build()
            .into()
    }
}

impl From<Permissions> for MissingPermission {
    fn from(permission: Permissions) -> Self {
        MissingPermission(permission)
    }
}
impl IntoResponse for MissingPermission {
    fn into_response(self) -> Response {
        let body: APIErrorResponse<Permissions, ()> = APIErrorResponse {
            message: "Missing Permission".into(),
            details: Some(self.0),
            error: None,
        };
        ResponseBuilder::forbidden()
            .extension(ErrorReason::from(format!("Missing Permission {}", self.0)))
            .json(&body)
    }
}

pub struct MissingPermissionResponse<P: PermissionCheck>(PhantomData<P>);

impl<P> utoipa::IntoResponses for MissingPermissionResponse<P>
where
    P: PermissionCheck,
{
    fn responses() -> std::collections::BTreeMap<String, utoipa::openapi::RefOr<response::Response>>
    {
        let missing_permission_response = APIErrorResponse::<Permissions, ()>::name();

        response::ResponsesBuilder::new()
            .responses_from_iter([(
                "403",
                utoipa::openapi::ResponseBuilder::new()
                    .description("Missing Permission")
                    .content(
                        "application/json",
                        utoipa::openapi::content::ContentBuilder::new()
                            .schema(Some(
                                RefBuilder::new()
                                    .ref_location_from_schema_name(missing_permission_response),
                            ))
                            .examples_from_iter(examples_from_permission_check::<P>())
                            .into(),
                    )
                    .build(),
            )])
            .build()
            .into()
    }
}
/// Generate examples for each permission required by the PermissionCheck
fn examples_from_permission_check<P: PermissionCheck>()
-> impl Iterator<Item = (String, ExampleBuilder)> {
    P::permissions_required().iter().map(|permission| {
        let response: APIErrorResponse<&Permissions, ()> = APIErrorResponse {
            message: MISSING_PERMISSION_MESSAGE.into(),
            details: Some(permission),
            error: None,
        };
        (
            format!("Missing {}", permission),
            utoipa::openapi::example::ExampleBuilder::new()
                .value(Some(serde_json::to_value(response).unwrap())),
        )
    })
}

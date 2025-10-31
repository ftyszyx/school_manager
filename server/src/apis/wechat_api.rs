use crate::{apis::user_api, core::app::AppState};
use crate::core::error::AppError;
use crate::core::response::ApiResponse;
use crate::utils::jwt::create_jwt;
use data_model::{user_roles, users};
use salvo::{oapi::extract::*, prelude::*};
use sea_orm::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::core::constants::TEACHER_ROLE_ID;

#[derive(Deserialize, Debug, ToSchema)]
pub struct WechatLoginPayload {
    pub code: String,
    #[serde(rename = "nickName")]
    pub nickname: Option<String>,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
}

#[derive(Deserialize, Debug)]
struct WechatSessionResponse {
    openid: Option<String>,
    // unionid: Option<String>,
    // // session_key: Option<String>,
    errcode: Option<i32>,
    errmsg: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
}

#[handler]
pub async fn wechat_login(
    depot: &mut Depot,
    req: JsonBody<WechatLoginPayload>,
) -> Result<ApiResponse<AuthResponse>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let config = &state.config.wechat;
    // Call Wechat API to get openid
    let client = Client::new();
    let res = client
        .get("https://api.weixin.qq.com/sns/jscode2session")
        .query(&[
            ("appid", &config.app_id),
            ("secret", &config.app_secret),
            ("js_code", &req.code),
            ("grant_type", &"authorization_code".to_string()),
        ])
        .send()
        .await
        .map_err(|_| AppError::InternalError {
            message: "Failed to send request".to_string(),
        })?
        .json::<WechatSessionResponse>()
        .await
        .map_err(|_| AppError::InternalError {
            message: "Failed to parse response".to_string(),
        })?;
    tracing::info!("Wechat session response: {:?}", res);

    if let Some(errcode) = res.errcode {
        if errcode != 0 {
            return Err(AppError::InternalError {
                message: format!("Wechat API error: {}", res.errmsg.unwrap_or_default()),
            });
        }
    }
    let openid = res.openid.ok_or(AppError::auth_failed("Invalid code"))?;
    // let unionid = res.unionid.ok_or(AppError::auth_failed("Invalid code"))?;
    // Find or create user
    let user = match users::Entity::find()
        .filter(users::Column::WechatOpenid.eq(&openid))
        .one(&state.db)
        .await?
    {
        Some(user) => { // User exists, update their info
            let mut active_user: users::ActiveModel = user.into();
            if let Some(nickname) = &req.nickname {
                active_user.wechat_nickname = Set(Some(nickname.clone()));
            }
            if let Some(avatar_url) = &req.avatar_url {
                active_user.wechat_avatar_url = Set(Some(avatar_url.clone()));
            }
            active_user.update(&state.db).await?
        }
        None => {
            let create_req = user_api::UserCreatePayload {
                username: openid.clone(),
                password: state.config.system.default_user_password.clone(),
                role_ids: Some(vec![TEACHER_ROLE_ID]),
                class_ids: None,
            };
            let user = user_api::add_impl(
                state,
                create_req,
                Some(Box::new(move |active_user| {
                    Box::pin(async move {
                        active_user.wechat_openid = Set(Some(openid));
                        // active_user.wechat_unionid = Set(Some(unionid));
                        Ok(())
                    })
                })),
            )
            .await?;
            user
        }
    };

    // Get role_ids for JWT
    let role_ids: Vec<i32> = user_roles::Entity::find()
        .filter(user_roles::Column::UserId.eq(user.id))
        .all(&state.db)
        .await?
        .iter()
        .map(|r| r.role_id)
        .collect();
    // Create JWT
    let token = create_jwt(user.id, role_ids, &state.config.jwt)?;
    Ok(ApiResponse::success(AuthResponse { token }))
}

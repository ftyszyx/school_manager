use crate::core::app::AppState;
use jsonwebtoken::{decode, DecodingKey, Validation};
use crate::apis::permission_api;
use serde::{Deserialize, Serialize};
use salvo::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub user_id: i32,
    pub role_ids: Vec<i32>,
    pub exp: usize,
}


#[handler]
pub async fn permission_handler(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
)->Result<(), StatusCode> {
 let state=match depot.obtain::<AppState>() {
        Ok(s) => s,
        Err(_) => {
            // res.status_code = Some(StatusCode::INTERNAL_SERVER_ERROR);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let claims=match depot.obtain::<Claims>() {
        Ok(c) => c,
        Err(_) => {
            return Err(StatusCode::UNAUTHORIZED);
        }
    };
    let method=req.method().clone();
    let path=req.uri().path().to_string();
    let result=permission_api::check_path_permission_and_cache(state, claims.user_id, claims.role_ids.clone(), method.as_str(), path.as_str()).await;
    let _=match result {
        Ok(result) => result,
        Err(_) => return Err(StatusCode::FORBIDDEN),
    };
    ctrl.call_next(req, depot, res).await;
    Ok(())
}


#[handler]
pub async fn auth(
    req: &mut Request,
    depot:&mut Depot,
) -> Result<(), StatusCode>{
    let state = depot.obtain::<AppState>().unwrap();
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_value| {
            if auth_value.starts_with("Bearer ") {
                Some(auth_value[7..].to_owned())
            } else {
                None
            }
        });

    let token = token.ok_or(StatusCode::UNAUTHORIZED)?;
    let decoded = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(state.config.jwt.secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;
    depot.inject(decoded.claims);
    Ok(())
}

#[handler]
pub async fn error_handler(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
){
    // 先放行到下游处理
    ctrl.call_next(req, depot, res).await;
    if let Some(code) = res.status_code{
        if code.as_u16() >= 400 {
            tracing::error!("Response status: {}", code);
        }
    }
    // ctrl.call_next(req, depot, res).await;
}
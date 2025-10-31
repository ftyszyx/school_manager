use salvo::prelude::*;
use salvo::cors::{Cors, AllowOrigin, AllowHeaders};
use salvo::http::Method;
use crate::core::app::AppState;
use salvo_oapi::{OpenApi, SecurityScheme};
use salvo_oapi::security::{Http, HttpAuthScheme};
use crate::apis::*;

#[handler]
async fn hello(res: &mut Response) {
    res.render(Text::Plain("Hello, world!"));
}

pub fn build_router(app_state: AppState) -> Router {
    let admin_routes = Router::with_path("/api/admin")
         .hoop(auth_middleware::auth)
         .hoop(auth_middleware::error_handler)
        //users
        .push(Router::with_path("/users").get(user_api::get_list))
        .push(Router::with_path("/users/{id}").get(user_api::get_by_id))
        .push(Router::with_path("/users").post(user_api::add))
        .push(Router::with_path("/users/{id}").put(user_api::update))
        .push(Router::with_path("/users/{id}").delete(user_api::delete))
        .push(Router::with_path("/me").get(user_api::get_current_user))
        .push(Router::with_path("/me/password").post(user_api::change_password))
        .push(Router::with_path("/logout").post(user_api::logout))
        .push(Router::with_path("/bind/class").post(user_api::bind_class))
        .push(Router::with_path("/unbind/class/{class_id}").delete(user_api::unbind_class))
        //roles
        .push(Router::with_path("/roles").get(role_api::get_list))
        .push(Router::with_path("/roles/{id}").get(role_api::get_by_id))
        .push(Router::with_path("/roles").post(role_api::add))
        .push(Router::with_path("/roles/{id}").put(role_api::update))
        .push(Router::with_path("/roles/{id}").delete(role_api::delete))
        //permissions
        .push(Router::with_path("/permissions").get(permission_api::get_list))
        .push(Router::with_path("/permissions/{id}").get(permission_api::get_by_id))
        .push(Router::with_path("/permissions").post(permission_api::add))
        .push(Router::with_path("/permissions/{id}").put(permission_api::update))
        .push(Router::with_path("/permissions/{id}").delete(permission_api::delete))
        //schools
        .push(Router::with_path("/schools").get(school_api::get_list))
        .push(Router::with_path("/schools/{id}").get(school_api::get_by_id))
        .push(Router::with_path("/schools").post(school_api::add))
        .push(Router::with_path("/schools/{id}").put(school_api::update))
        .push(Router::with_path("/schools/{id}").delete(school_api::delete))
        //classes
        .push(Router::with_path("/classes").get(class_api::get_list))
        .push(Router::with_path("/classes/{id}").get(class_api::get_by_id))
        .push(Router::with_path("/classes").post(class_api::add))
        .push(Router::with_path("/classes/{id}").put(class_api::update))
        .push(Router::with_path("/classes/{id}").delete(class_api::delete))
        .push( Router::with_path("/classes/bulk") .post(class_api::add_bulk))
        .push(Router::with_path("/classes/{class_id}/status").put(class_api::update_status))
        .push(Router::with_path("/ws/school/{id}").goal(ws_api::school_ws_handler));
    Router::new()
        .hoop(affix_state::inject(app_state))
        .push(Router::with_path("/api/login").post(user_api::login))
        .push(Router::with_path("/api/register").post(user_api::register))
        .push(Router::with_path("/api/login/wechat").post(wechat_api::wechat_login))
        .get(hello)
        .push(admin_routes)
}

pub fn create_router(app_state: AppState) -> Service {
    let cors = Cors::new()
    .allow_origin(AllowOrigin::any())
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
    .allow_headers(AllowHeaders::any()).into_handler();
    let router=build_router(app_state);
    let doc=OpenApi::new("app_server_api", "1.0.0")
        .add_security_scheme("bearer", SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer).bearer_format("JWT")))
        .merge_router(&router);
    let router=router.unshift(doc.into_router("/api-doc/openapi.json"))
    .unshift(SwaggerUi::new("/api-doc/openapi.json").into_router("/swagger-ui"));
    let service=Service::new(router).hoop(cors).hoop(Logger::new());
    service
}

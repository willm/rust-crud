use actix_web::{error, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
mod db;
mod blog_post;
use blog_post::BlogPost;

#[derive(Deserialize, Serialize)]
struct ApiError {
    message: String
}

#[post("/")]
async fn create_blog_post(blog_post: web::Json<BlogPost>) -> impl Responder {
    match db::insert_post(&blog_post).await {
        Ok(_ok) => return HttpResponse::Ok().json(&blog_post),
        Err(err) => return HttpResponse::InternalServerError()
        .json(ApiError {message: format!("{:?}", err) })
    }
}

fn json_error_handler(err: error::JsonPayloadError, _req: &HttpRequest) -> error::Error {
    use actix_web::error::JsonPayloadError;

    let detail = err.to_string();
    let resp = match &err {
        JsonPayloadError::ContentType => HttpResponse::UnsupportedMediaType().body(detail),
        JsonPayloadError::Deserialize(json_err) if json_err.is_data() => {
            HttpResponse::BadRequest().json(ApiError {message: detail.into()})
        }
        _ => HttpResponse::BadRequest().json(ApiError {message: detail.into()}),
    };
    error::InternalError::from_response(err, resp).into()
}

fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(create_blog_post).app_data(
        web::JsonConfig::default()
            // register error_handler for JSON extractors.
            .error_handler(json_error_handler),
    );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().configure(configure)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use actix_web::{test, web::Bytes, dev::ServiceResponse};
    use super::*;

    async fn get_response(body: String) -> ServiceResponse {
        let app = test::init_service(App::new().configure(configure)).await;
        let req = test::TestRequest::post().uri("/")
            .insert_header(("Content-Type", "application/json"))
            .set_payload(Bytes::from(body))
            .to_request();
        test::call_service(&app, req).await
    }

    #[actix_web::test]
    async fn test_creating_a_post_with_a_missing_name() {
        let resp = get_response(String::from("{}")).await;
        assert!(resp.status().is_client_error());
        assert_eq!(resp.status(), 400);
        let body: ApiError = test::read_body_json(resp).await;
        assert_eq!(body.message, "Json deserialize error: missing field `name` at line 1 column 2");
    }

    #[actix_web::test]
    async fn test_creating_a_post_with_a_missing_body() {
        let resp = get_response(String::from(r#"{"name": "foo"}"#)).await;
        assert!(resp.status().is_client_error());
        assert_eq!(resp.status(), 400);
        let body: ApiError = test::read_body_json(resp).await;
        assert_eq!(body.message, "Json deserialize error: missing field `body` at line 1 column 15");
    }

    #[actix_web::test]
    async fn test_creating_a_post() {
        let resp = get_response(String::from(r#"{"name": "Will", "body": "Hello World!"}"#)).await;
        assert_eq!(resp.status(), 200);
        let body: BlogPost = test::read_body_json(resp).await;
        assert_eq!(body.name, "Will");
        assert_eq!(body.body, "Hello World!");
    }
}

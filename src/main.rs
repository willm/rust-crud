extern crate dotenv;
mod blog_post;
mod config;
mod credentials;
mod db;
use actix_cors::Cors;
use actix_web::{error, get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use base64::{engine::general_purpose, Engine as _};
use credentials::post_credentials;
use db::PostDatabase;
use dotenv::dotenv;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use serde_repr::Serialize_repr;
use std::error::Error;

#[derive(Deserialize, Serialize_repr)]
#[repr(i64)]
enum Alg {
    RS256 = -257,
    ES256 = -7,
}

#[derive(Deserialize, Serialize)]
struct ApiError {
    message: String,
}

#[derive(Deserialize, Serialize)]
struct PubKeyCredParams {
    alg: Alg,
    #[serde(rename(serialize = "type"))]
    _type: String,
}

#[derive(Deserialize, Serialize)]
struct RelayingParty {
    id: String,
    name: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct User {
    id: String,
    name: String,
    display_name: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Credentials {
    challenge: String,
    pub_key_cred_params: [PubKeyCredParams; 2],
    rp: RelayingParty,
    user: User,
}

impl Credentials {
    pub fn new(challenge: String) -> Self {
        Credentials {
            challenge,
            pub_key_cred_params: [
                PubKeyCredParams {
                    alg: Alg::RS256,
                    _type: String::from("public-key"),
                },
                PubKeyCredParams {
                    alg: Alg::ES256,
                    _type: String::from("public-key"),
                },
            ],
            rp: RelayingParty {
                id: String::from(config::ORIGIN),
                name: String::from("Rusty auth"),
            },
            user: User {
                id: random_bytes_base64(),
                name: String::from("User"),
                display_name: String::from("User Disp"),
            },
        }
    }
}

#[derive(Deserialize)]
struct UserRequest {
    email: String,
}

fn random_bytes_base64() -> String {
    let mut data = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut data);
    general_purpose::STANDARD_NO_PAD.encode(data)
}

async fn setup_challenge(
    db: &PostDatabase,
    email: &str,
    challenge: &str,
) -> Result<(), Box<dyn Error>> {
    let user_id = &db.insert_user(email).await?;
    db.insert_user_challenge(user_id.clone(), &challenge)
        .await?;
    Ok(())
}

#[get("/credentials")]
async fn get_credentials(
    user_request: web::Query<UserRequest>,
    db: web::Data<PostDatabase>,
) -> impl Responder {
    let email = &user_request.email;
    if let Ok(user) = &db.get_user(&email).await {
        match user {
            Some(_user_id) => {
                return HttpResponse::Conflict()
                    .body(format!("User {} already exists, log in and retry", &email))
            }
            None => {
                let challenge = random_bytes_base64();
                match setup_challenge(&db, &email, &challenge).await {
                    Ok(_) => return HttpResponse::Ok().json(Credentials::new(challenge)),
                    Err(err) => {
                        return HttpResponse::InternalServerError().body(format!("{:?}", err))
                    }
                }
            }
        }
    }
    HttpResponse::InternalServerError().body("Failed to check user")
}

//#[post("/")]
//async fn create_blog_post(blog_post: web::Json<BlogPost>) -> impl Responder {
//    match db::insert_post(&blog_post).await {
//        Ok(_ok) => return HttpResponse::Ok().json(&blog_post),
//        Err(err) => {
//            return HttpResponse::InternalServerError().json(ApiError {
//                message: format!("{:?}", err),
//            })
//        }
//    }
//}

fn json_error_handler(err: error::JsonPayloadError, _req: &HttpRequest) -> error::Error {
    use actix_web::error::JsonPayloadError;

    let detail = err.to_string();
    let resp = match &err {
        JsonPayloadError::ContentType => HttpResponse::UnsupportedMediaType().body(detail),
        JsonPayloadError::Deserialize(json_err) if json_err.is_data() => HttpResponse::BadRequest()
            .json(ApiError {
                message: detail.into(),
            }),
        _ => HttpResponse::BadRequest().json(ApiError {
            message: detail.into(),
        }),
    };
    error::InternalError::from_response(err, resp).into()
}

fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        //.service(create_blog_post)
        .service(get_credentials)
        .service(post_credentials)
        .app_data(
            web::JsonConfig::default()
                // register error_handler for JSON extractors.
                .error_handler(json_error_handler),
        );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let db = db::PostDatabase::create().await.unwrap();
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:8000")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec!["Content-Type"])
            .max_age(3600);
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(db.clone()))
            .configure(configure)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{dev::ServiceResponse, test, web::Bytes};

    async fn get_response(body: String) -> ServiceResponse {
        let app = test::init_service(App::new().configure(configure)).await;
        let req = test::TestRequest::post()
            .uri("/")
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
        assert_eq!(
            body.message,
            "Json deserialize error: missing field `name` at line 1 column 2"
        );
    }

    #[actix_web::test]
    async fn test_creating_a_post_with_a_missing_body() {
        let resp = get_response(String::from(r#"{"name": "foo"}"#)).await;
        assert!(resp.status().is_client_error());
        assert_eq!(resp.status(), 400);
        let body: ApiError = test::read_body_json(resp).await;
        assert_eq!(
            body.message,
            "Json deserialize error: missing field `body` at line 1 column 15"
        );
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

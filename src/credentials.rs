use crate::db::PostDatabase;
use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ClientData {
    #[serde(rename = "type")]
    _type: String,
    challenge: String,
    origin: String,
    cross_origin: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PublicKeyCredentialResponse {
    attestation_object: String,
    client_data: ClientData,
}

#[derive(Serialize, Deserialize)]
pub struct PublicKeyCredential {
    id: String,
    email: String,
    response: PublicKeyCredentialResponse,
}

// https://www.w3.org/TR/webauthn/#sctn-registering-a-new-credential
async fn verify(
    db: &PostDatabase,
    email: &str,
    client_data: &ClientData,
) -> Result<bool, Box<dyn std::error::Error>> {
    if client_data._type != String::from("webauthn.create") {
        return Ok(false);
    }

    let challenge_exists = db
        .user_challenge_exists(&client_data.challenge, email)
        .await?;
    if !challenge_exists {
        return Ok(false);
    }

    if client_data.origin != crate::config::ORIGIN {
        return Ok(false);
    }

    Ok(true)
}

#[post("/credentials")]
pub async fn post_credentials(
    user_request: web::Json<PublicKeyCredential>,
    db: web::Data<PostDatabase>,
) -> impl Responder {
    let client_data = &user_request.response.client_data;
    let result = verify(&db, &user_request.email, &client_data).await;
    if let Ok(_) = result {
        return HttpResponse::Ok().body("");
    }
    HttpResponse::Unauthorized().body("")
}

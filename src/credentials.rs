use crate::db::PostDatabase;
use actix_web::{post, web, HttpResponse, Responder};
use base64::{engine::general_purpose, Engine as _};
use ring::digest;
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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatorAttestationResponse {
    auth_data: Vec<u8>,
    fmt: String,
}

impl AuthenticatorAttestationResponse {
    fn from_b64_string(encoded: &str) -> Option<Self> {
        let bytes = &general_purpose::STANDARD.decode(encoded).unwrap();
        ciborium::de::from_reader(&bytes[..]).ok()
    }
}

pub struct AuthenticatorData<'a> {
    rp_id_hash: &'a [u8],
}

impl<'a> AuthenticatorData<'a> {
    fn from_bytes(bytes: &'a Vec<u8>) -> Self {
        AuthenticatorData {
            rp_id_hash: &bytes[..32],
        }
    }
}

// https://www.w3.org/TR/webauthn/#sctn-registering-a-new-credential
async fn verify(
    db: &PostDatabase,
    email: &str,
    public_key_credential_response: &PublicKeyCredentialResponse,
) -> Result<bool, Box<dyn std::error::Error>> {
    let client_data = &public_key_credential_response.client_data;
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

    let attestation_response = AuthenticatorAttestationResponse::from_b64_string(
        &public_key_credential_response.attestation_object,
    );
    match attestation_response {
        None => return Ok(false),
        Some(response) => {
            let authenticator_data = AuthenticatorData::from_bytes(&response.auth_data);
            let sha_256_digest = digest::digest(&digest::SHA256, crate::config::ORIGIN.as_bytes());
            if sha_256_digest.as_ref() != authenticator_data.rp_id_hash {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

#[post("/credentials")]
pub async fn post_credentials(
    user_request: web::Json<PublicKeyCredential>,
    db: web::Data<PostDatabase>,
) -> impl Responder {
    let result = verify(&db, &user_request.email, &user_request.response).await;
    if let Ok(_) = result {
        return HttpResponse::Ok().body("");
    }
    HttpResponse::Unauthorized().body("")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verifying_att() {
        let raw_attestation_object = "o2NmbXRkbm9uZWdhdHRTdG10oGhhdXRoRGF0YVjFSZYN5YgOjGh0NBcPZHZgW4/krrmihjLHmVzzuoMdl2NFAAAAAAAAAAAAAAAAAAAAAAAAAAAAQQEA4wyJikPPpb0YSIMW3D6jT2Du0n0Rnfim3hoiRoMdluSS+aCBBnyK7lu/hfpasycIhsV7Rq/QRVd0MVykiiKOpQECAyYgASFYIF5cREuA9SBROn/KmVkv2KS0fwFDwvZvsmA3zY4JGuP5Ilgge52g+rd/0iPU+OISmTTxctOMgcW24KvRMlqTZbasn4s=";
        let response = AuthenticatorAttestationResponse::from_b64_string(raw_attestation_object)
            .ok_or_else(|| panic!("attestation response was none"))
            .unwrap();
        assert_eq!(response.fmt, "none");
        let authenticator_data = AuthenticatorData::from_bytes(&response.auth_data);
        let sha_256_digest = digest::digest(&digest::SHA256, "localhost".as_bytes());
        assert_eq!(sha_256_digest.as_ref(), authenticator_data.rp_id_hash);
    }
}

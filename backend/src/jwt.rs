use std::cell::{RefCell, RefMut};
use std::ops::Deref;

use crypto::hmac::Hmac;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use crypto::mac::Mac;
use crypto::sha2::Sha256;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use anyhow::{Result, anyhow};

use axum::{async_trait, RequestPartsExt, extract::FromRequestParts, http::request::Parts, Json};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum_extra::{
    TypedHeader,
    headers::{
        authorization::Bearer,
        Authorization
    }
};
use axum_extra::headers::Cookie;
use serde_json::json;

const JWT_SECRET: &str = "test_secret";

#[derive(Debug, Serialize, Deserialize)]
pub enum JwtAlg {
    HS256
}

#[derive(Debug, Serialize, Deserialize)]
pub enum JwtTyp {
    JWT
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtHeader {
    alg:    JwtAlg,
    typ:    JwtTyp,
}
impl JwtHeader {
    fn default() -> Self {
        JwtHeader {
            alg: JwtAlg::HS256,
            typ: JwtTyp::JWT
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtPayload {
    user_id:    usize,
    exp_time:   i64,
}
impl JwtPayload {
    fn new(user_id: usize, expire_time_s: i64) -> Self {
        let time = OffsetDateTime::now_utc();
        JwtPayload {
            user_id,
            exp_time: time.unix_timestamp() + expire_time_s
        }
    }
}

pub type JwtSignature = Vec<u8>;

#[derive(Serialize)]
pub struct Jwt {
    header:     JwtHeader,
    payload:    JwtPayload,
    signature:  JwtSignature,

    #[serde(skip_serializing)]
    hmac_handler: RefCell<Hmac<Sha256>>
}

impl Jwt {
    fn new(user_id: usize, expire_duration_s: i64) -> Result<Self> {
        let mut handler = RefCell::new(Hmac::<Sha256>::new(Sha256::new(), JWT_SECRET.as_bytes()));
        let handle = handler.borrow_mut();

        let header = JwtHeader::default();
        let payload = JwtPayload::new(user_id, expire_duration_s);
        let signature = Self::generate_signature(handle, &header, &payload)?;

        Ok(Self {
            header,
            payload,
            signature,
            hmac_handler: handler,
        })
    }

    fn encode(&self) -> Result<String> {
        Ok(format!("{}.{}.{}", self.header_b64()?, self.payload_b64()?, self.signature_b64()))
    }

    pub fn generate(user_id: usize, expire_duration_s: i64) -> Result<String> {
        Jwt::new(user_id, expire_duration_s)?.encode()
    }

    pub fn verify(&self) -> Result<(), JwtError> {
        let result = Self::generate_signature(self.handler_mut(), self.header(), self.payload())
            .map_err(|_| JwtError::InternalError("Failed to generate signature".into()))?;

        if !result.eq(self.signature()) {
            return Err(JwtError::InvalidToken)
        }

        let time = OffsetDateTime::now_utc();
        if time.unix_timestamp() > self.payload.exp_time {
            return Err(JwtError::Expired)
        }

        Ok(())
    }

    pub fn parse_and_verify(s: &str) -> Result<Self, JwtError> {
        let jwt: Jwt = s.try_into().map_err(|_| JwtError::InvalidToken)?;
        jwt.verify()?;
        Ok(jwt)
    }

    fn generate_signature(mut handler: RefMut<Hmac<Sha256>>, h: &JwtHeader, p: &JwtPayload) -> Result<JwtSignature> {
        let h_json  = serde_json::to_string(h)?;
        let p_json = serde_json::to_string(p)?;

        let header_b64  = URL_SAFE.encode(h_json);
        let payload_b64 = URL_SAFE.encode(p_json);

        let content = format!("{header_b64}.{payload_b64}");
        handler.input(content.as_bytes());
        let signature = handler.result().code().to_vec();
        handler.reset();

        Ok(signature)
    }

    fn header_b64(&self) -> Result<String> {
        let h_json  = serde_json::to_string(self.header())?;
        Ok(URL_SAFE.encode(h_json))
    }
    fn payload_b64(&self) -> Result<String> {
        let p_json  = serde_json::to_string(self.payload())?;
        Ok(URL_SAFE.encode(p_json))
    }
    fn signature_b64(&self) -> String {
        URL_SAFE.encode(&self.signature)
    }

    fn header(&self) -> &JwtHeader {
        &self.header
    }
    fn payload(&self) -> &JwtPayload {
        &self.payload
    }
    fn signature(&self) -> &JwtSignature {
        &self.signature
    }
    fn signature_str(&self) -> String {
        String::from_utf8_lossy(self.signature.as_slice()).to_string()
    }
    fn handler_mut(&self) -> RefMut<Hmac<Sha256>> {
        self.hmac_handler.borrow_mut()
    }

}

impl TryInto<Jwt> for String {
    type Error = anyhow::Error;

    fn try_into(self) -> std::result::Result<Jwt, Self::Error> {
        let jwt_vec: Vec<&str> = self.split(".").collect();
        if jwt_vec.len() != 3 {
            return Err(anyhow!("Invalid JWT."));
        }
        let header_json_u8 = URL_SAFE.decode(jwt_vec[0])?;
        let payload_json_u8 = URL_SAFE.decode(jwt_vec[1])?;
        let header_json = String::from_utf8_lossy(header_json_u8.as_slice());
        let payload_json = String::from_utf8_lossy(payload_json_u8.as_slice());
        let signature = URL_SAFE.decode(jwt_vec[2])?;

        let header: JwtHeader = serde_json::from_str(header_json.deref())?;
        let payload: JwtPayload = serde_json::from_str(payload_json.deref())?;
        let mut hmac = Hmac::<Sha256>::new(Sha256::new(), JWT_SECRET.as_bytes());
        hmac.reset();

        Ok( Jwt {
            header,
            payload,
            signature,
            hmac_handler: RefCell::new(hmac)
        })
    }
}

impl TryFrom<&str> for Jwt {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        let jwt_str = String::from(s);
        let jwt_vec: Vec<&str> = jwt_str.split(".").collect();
        if jwt_vec.len() != 3 {
            return Err(anyhow!("Invalid JWT."));
        }
        let header_json_u8 = URL_SAFE.decode(jwt_vec[0])?;
        let payload_json_u8 = URL_SAFE.decode(jwt_vec[1])?;
        let header_json = String::from_utf8_lossy(header_json_u8.as_slice());
        let payload_json = String::from_utf8_lossy(payload_json_u8.as_slice());
        let signature = URL_SAFE.decode(jwt_vec[2])?;

        let header: JwtHeader = serde_json::from_str(header_json.deref())?;
        let payload: JwtPayload = serde_json::from_str(payload_json.deref())?;
        let mut hmac = Hmac::<Sha256>::new(Sha256::new(), JWT_SECRET.as_bytes());
        hmac.reset();

        Ok( Jwt {
            header,
            payload,
            signature,
            hmac_handler: RefCell::new(hmac)
        })
    }
}

impl TryInto<String> for Jwt {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        Ok(format!("{}.{}.{}", self.header_b64()?, self.payload_b64()?, self.signature_b64()))
    }
}

pub enum JwtError {
    InvalidToken,
    Expired,
    InternalError(String),
}

#[async_trait]
impl<S> FromRequestParts<S> for Jwt where S: Send + Sync {
    type Rejection = JwtError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> std::result::Result<Self, Self::Rejection> {
        let TypedHeader(cookie) = parts
            .extract::<TypedHeader<Cookie>>().await
            .map_err(|_| JwtError::InvalidToken)?;
        let jwt_str = cookie.get("token").ok_or(JwtError::InvalidToken)?;

        // let TypedHeader(Authorization(bearer)) = parts
        //     .extract::<TypedHeader<Authorization<Bearer>>>().await
        //     .map_err(|_| JwtError::InvalidToken)?;
        // let jwt_str = bearer.token();
        Jwt::parse_and_verify(jwt_str)
    }
}

impl IntoResponse for JwtError {
    fn into_response(self) -> Response {
        let (status, err_msg) = match self {
            JwtError::InvalidToken
                => (StatusCode::UNAUTHORIZED, "Invalid token"),
            JwtError::Expired
                => (StatusCode::UNAUTHORIZED, "Token expired"),
            JwtError::InternalError(_)
                =>(StatusCode::INTERNAL_SERVER_ERROR, "Token Validation Error")
        };

        let body = Json(json!({
            "status": "error",
            "error": err_msg,
        }));
        (status, body).into_response()
    }
}

#[test]
fn jwt_test() {
    let jwt = Jwt::new(114514, 60).unwrap();
    let str = jwt.encode().unwrap();
    println!("{:?}", serde_json::to_string(&jwt).unwrap());
    println!("{}", str);
    println!("\n");
    let jwt: Jwt = Jwt::try_from(str.as_str()).unwrap();
    println!("{:?}", serde_json::to_string(&jwt).unwrap());
    println!("{}", jwt.encode().unwrap());

    // let jwt = Jwt::from(r"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyX2lkIjoxMTQ1MTQsImV4cF90aW1lIjoxNzE2ODAxMDY1NjU5fQ==.7D7kMJXmoomnEO8wzRXDQd2uAEsQNaVzJ2BKH_DCZNs=");
}

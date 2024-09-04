use std::cell::{RefCell, RefMut};
use std::ops::Deref;

use crypto::hmac::Hmac;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use crypto::mac::Mac;
use crypto::sha2::Sha256;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use anyhow::{Result, anyhow};

const SECRET: &str = "test_secret";

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
    fn new(user_id: usize, expire_duration_s: i64, secret: &[u8]) -> Result<Self> {
        let mut handler = RefCell::new(Hmac::<Sha256>::new(Sha256::new(), secret));
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
    pub fn verify(&self) -> Result<bool> {
        let result = Self::generate_signature(self.handler_mut(), self.header(), self.payload())?;
        if !result.eq(self.signature()) {
            return Ok(false)
        }

        let time = OffsetDateTime::now_utc();
        if time.unix_timestamp() > self.payload.exp_time {
            return Ok(false)
        }

        Ok(true)
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

impl TryInto<String> for Jwt {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        Ok(format!("{}.{}.{}", self.header_b64()?, self.payload_b64()?, self.signature_b64()))
    }
}

pub struct JwtGenerator {
    secret:         Box<[u8]>,
    exp_duration:   i64,
}

impl JwtGenerator {
    pub fn new(secret: String, default_expire_time_s: i64) -> Self {
        JwtGenerator {
            secret: secret.into_bytes().into_boxed_slice(),
            exp_duration: default_expire_time_s,
        }
    }

    pub fn generate(&self, user_id: usize, expire_time_s: Option<i64>) -> Result<Jwt> {
        let expire_duration = expire_time_s.unwrap_or(self.exp_duration);
        Jwt::new(user_id, expire_duration, &self.secret)
    }

    pub fn parse(&self, jwt_str: &str) -> Result<Jwt> {
        let jwt_str = String::from(jwt_str);
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
        let mut hmac = Hmac::<Sha256>::new(Sha256::new(), &self.secret);
        hmac.reset();

        Ok( Jwt {
            header,
            payload,
            signature,
            hmac_handler: RefCell::new(hmac)
        })
    }

    pub fn parse_and_verify(&self, jwt_str: &str) -> Result<bool> {
        let jwt = Self::parse(self, jwt_str)?;
        jwt.verify()
    }
}

#[test]
fn main() -> Result<()> {
    let jwt = JwtGenerator::new("test_secret".to_string(), 100);
    let jwt1 = jwt.generate(114514, Some(100))?;
    println!("{:?}", serde_json::to_string(&jwt1)?);
    let str1: String = jwt1.try_into()?;
    println!("{}", str1);
    println!("\n");
    let jwt2: Jwt = jwt.parse(str1.as_str())?;
    println!("{:?}", serde_json::to_string(&jwt2)?);
    let str2: String = jwt2.try_into()?;
    println!("{}", str2);

    Ok(())

    // let jwt = Jwt::from(r"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyX2lkIjoxMTQ1MTQsImV4cF90aW1lIjoxNzE2ODAxMDY1NjU5fQ==.7D7kMJXmoomnEO8wzRXDQd2uAEsQNaVzJ2BKH_DCZNs=");
}

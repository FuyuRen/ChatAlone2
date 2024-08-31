use std::cell::{RefCell, RefMut};
use std::error::Error;
use std::ops::Deref;

use crypto::hmac::Hmac;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use crypto::mac::Mac;
use crypto::sha2::Sha256;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

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
    fn new(user_id: usize) -> Self {
        let exp_duration = 300_000;
        let time = OffsetDateTime::now_utc();
        JwtPayload {
            user_id,
            exp_time: time.unix_timestamp() + exp_duration
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
    fn new(user_id: usize) -> Result<Self, Box<dyn Error>> {
        let mut handler = RefCell::new(Hmac::<Sha256>::new(Sha256::new(), SECRET.as_bytes()));
        let handle = handler.borrow_mut();

        let header = JwtHeader::default();
        let payload = JwtPayload::new(user_id);
        let signature = Self::generate_signature(handle, &header, &payload)?;

        Ok(Self {
            header,
            payload,
            signature,
            hmac_handler: handler,
        })
    }
    fn verify(&self) -> Result<bool, Box<dyn Error>> {
        let result = Self::generate_signature(self.handler_mut(), self.header(), self.payload())?;
        Ok(result.eq(self.signature()))
    }

    fn encode(&self) -> Result<String, Box<dyn Error>> {
        Ok(format!("{}.{}.{}", self.header_b64()?, self.payload_b64()?, self.signature_b64()))
    }

    pub fn generate_signature(mut handler: RefMut<Hmac<Sha256>>, h: &JwtHeader, p: &JwtPayload) -> Result<JwtSignature, Box<dyn Error>> {
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

    pub fn header_b64(&self) -> Result<String, Box<dyn Error>> {
        let h_json  = serde_json::to_string(self.header())?;
        Ok(URL_SAFE.encode(h_json))
    }
    pub fn payload_b64(&self) -> Result<String, Box<dyn Error>> {
        let p_json  = serde_json::to_string(self.payload())?;
        Ok(URL_SAFE.encode(p_json))
    }
    pub fn signature_b64(&self) -> String {
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


impl From<&str> for Jwt {
    fn from(s: &str) -> Self {
        let jwt_str = String::from(s);
        let jwt_vec: Vec<&str> = jwt_str.split(".").collect();
        if jwt_vec.len() != 3 {
            panic!("Invalid JWT.")
        }
        let header_json_u8 = URL_SAFE.decode(jwt_vec[0]).unwrap();
        let payload_json_u8 = URL_SAFE.decode(jwt_vec[1]).unwrap();
        let header_json = String::from_utf8_lossy(header_json_u8.as_slice());
        let payload_json = String::from_utf8_lossy(payload_json_u8.as_slice());
        let signature = URL_SAFE.decode(jwt_vec[2]).unwrap();

        let header: JwtHeader = serde_json::from_str(header_json.deref()).unwrap();
        let payload: JwtPayload = serde_json::from_str(payload_json.deref()).unwrap();
        let mut hmac = Hmac::<Sha256>::new(Sha256::new(), SECRET.as_bytes());
        hmac.reset();

        Jwt {
            header,
            payload,
            signature,
            hmac_handler: RefCell::new(hmac)
        }

    }
}

#[test]
fn main() {
    let jwt = Jwt::new(114514).unwrap();
    let str = jwt.encode().unwrap();
    println!("{:?}", serde_json::to_string(&jwt).unwrap());
    println!("{}", str);
    println!("\n");
    let jwt = Jwt::from(str.as_str());
    println!("{:?}", serde_json::to_string(&jwt).unwrap());
    println!("{}", jwt.encode().unwrap());

    // let jwt = Jwt::from(r"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyX2lkIjoxMTQ1MTQsImV4cF90aW1lIjoxNzE2ODAxMDY1NjU5fQ==.7D7kMJXmoomnEO8wzRXDQd2uAEsQNaVzJ2BKH_DCZNs=");
}

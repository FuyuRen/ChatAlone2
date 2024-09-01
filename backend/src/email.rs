use tokio;
use serde::Deserialize;
use anyhow::{Result, anyhow};
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};

#[derive(Debug, Deserialize)]
pub struct Email {
    address:        String,
    password:       String,
    smtp_address:   String,

    #[serde(skip)]
    template:       String,
}

impl Email {
    pub fn from(address: String, password: String, smtp_address: String) -> Self {
        let template = include_str!("../../frontend/email.html").to_string();
        Email {
            address,
            password,
            smtp_address,
            template,
        }
    }

    pub fn from_str(address: &str, password: &str, smtp_address: &str) -> Self {
        Self::from(
            address.to_string(),
            password.to_string(),
            smtp_address.to_string(),
        )
    }

    pub fn from_cfg(config: &String) -> Result<Self> {
        let template = include_str!("../../frontend/email.html").to_string();
        let mut ret = serde_json::from_str::<Email>(config)?;
        ret.template = template;
        Ok(ret)
    }

    pub async fn send(&self,
        to_addr: &str,
        subject: &str,
        content_type: ContentType,
        body: String
    ) -> Result<()> {
        let message = Message::builder()
            .from(self.address.parse()?)
            .to(to_addr.parse()?)
            .subject(subject)
            .header(content_type)
            .body(body)?;

        let creds = Credentials::new(self.address.to_string(), self.password.to_string());

        let mailer =
            if self.address.ends_with("@icloud.com") {
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(self.smtp_address.as_str())?
            } else {
                AsyncSmtpTransport::<Tokio1Executor>::relay(self.smtp_address.as_str())?
            };
        let mailer = mailer.credentials(creds).build();

        if let Err(e) = mailer.send(message).await {
            return Err(anyhow!("Failed to email due to: {}", e));
        }
        Ok(())
    }

    pub async fn send_verify_code(&self, to_addr: &str, code: u32) -> Result<()> {
        let body = self.template.replace("{{code}}", &code.to_string());
        self.send(
            to_addr,
            "[ChatAlone] Account Verification Code",
            ContentType::TEXT_HTML,
            body
        ).await?;
        Ok(())
    }
}

#[tokio::test]
async fn test1() {
    let cfg = include_str!("../cfg/email.json");
    let email = Email::from_cfg(&cfg.to_string()).unwrap();
    email.send_verify_code("liuenyan6@bupt.edu.cn", 114514).await.unwrap();
}
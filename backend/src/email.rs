use anyhow::{anyhow, Result};
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use serde::Deserialize;
use tokio;

#[derive(Debug, Clone, Deserialize)]
pub struct Email<'a> {
    address:        &'a str,
    password:       &'a str,
    smtp_address:   &'a str,

    #[serde(skip)]
    template:       &'a str,
}


impl<'a> Email<'a> {

    pub fn new(address: &'a str, password: &'a str, smtp_address: &'a str, template: &'a str) -> Self {
        Self { address, password, smtp_address, template, }
    }

    pub fn from(config: &'a str, template: &'a str) -> Result<Self> {
        let mut ret = serde_json::from_str::<Email>(config)?;
        ret.template = template;
        Ok(ret)
    }

    pub async fn send(
        &self,
        to_addr: &str,
        subject: &str,
        content_type: ContentType,
        body: String,
    ) -> Result<()> {
        let message = Message::builder()
            .from(self.address.parse()?)
            .to(to_addr.parse()?)
            .subject(subject)
            .header(content_type)
            .body(body)?;

        let creds = Credentials::new(self.address.to_string(), self.password.to_string());

        let mailer = if self.address.ends_with("@icloud.com") {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(self.smtp_address)?
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::relay(self.smtp_address)?
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
            body,
        )
        .await?;
        Ok(())
    }
}


#[tokio::test]
async fn test1() {
    let cfg = include_str!("../cfg/email.json");
    let tmp = include_str!("../../frontend/email.html");
    let emailer = Email::from(cfg, tmp).unwrap();
    emailer
        .send_verify_code("somebody@gmail.com", 114514)
        .await
        .unwrap();
}

use tokio;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Email {
    address: String,
    password: String,
    smtp_address: String,
}


impl Email {
    pub fn from(address: String, password: String, smtp_address: String) -> Self {
        Email {
            address,
            password,
            smtp_address,
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
        Ok(serde_json::from_str::<Email>(config)?)
    }

    pub async fn send(&self, to_addr: &str, subject: String, body: String) -> Result<()> {
        let message = Message::builder()
            .from(self.address.parse()?)
            .to(to_addr.parse()?)
            .subject(subject)
            .body(body)?;

        let creds = Credentials::new(self.address.to_string(), self.password.to_string());

        let mailer =
            if self.address.ends_with("@icloud.com") || self.address.ends_with("swisscows.self") {
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
}
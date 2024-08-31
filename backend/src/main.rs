mod email;
mod route;
mod room;
mod sql;
mod jwt;

use std::str::FromStr;
use tokio::{
    fs::File,
    io::{self, AsyncReadExt, AsyncWriteExt},
};
use anyhow::{Result, anyhow};
use time::OffsetDateTime;
use email::Email;
use crate::sql::UserID;

async fn fs_read(path: &str) -> Result<String> {
    let file = File::open(path).await?;
    let mut reader = io::BufReader::new(file);

    let mut email_cfg = String::new();
    reader.read_to_string(&mut email_cfg).await?;
    Ok(email_cfg)
}

#[tokio::main]
async fn main() -> Result<()> {
    let time = OffsetDateTime::now_utc();
    println!("{}", time);

    let email_cfg = fs_read("./cfg/email.json").await?;
    let email = Email::from_cfg(&email_cfg)?;

    // email.send("liuenyan6@bupt.edu.cn", "Hello".to_string(), "I am ChatAlone.".to_string()).await?;
    Ok(())
}


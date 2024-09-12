mod email;
mod route;
mod room;
mod sql;
mod jwt;
mod uuid;

use std::str::FromStr;
use tokio::{
    fs::File,
    io::{self, AsyncReadExt, AsyncWriteExt},
};
use anyhow::{Result, anyhow};
use time::OffsetDateTime;
use email::Email;
use crate::route::{fs_read, new};
use crate::sql::{
    SqlConn, UserDB, UserTable,
};

#[tokio::test]
async fn test_email() -> Result<()> {
    let time = OffsetDateTime::now_utc();
    println!("{}", time);

    let email_cfg = fs_read("./cfg/email.json").await?;
    let email = Email::from_cfg(&email_cfg)?;

    // email.send("somebody@gmail.com", "Hello".to_string(), "I am ChatAlone.".to_string()).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "0.0.0.0:55555";
    let sql_conn = SqlConn::new()?;
    let user_db = UserDB::init("users", sql_conn).await?;
    new(addr, user_db).await
}


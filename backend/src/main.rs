mod email;
mod room;
mod sql;
mod jwt;
mod uuid;
mod test;
mod server;

use std::str::FromStr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use anyhow::Result;
use server::{fs_read, route};
use time::OffsetDateTime;
use email::Email;

use crate::sql::{
    SqlConn, UserDB,
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

    let app = route(user_db);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("listening on {}", listener.local_addr()?);
    Ok(axum::serve(listener, app).await?)
}


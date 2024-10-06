mod email;
mod entities;
mod jwt;
mod room;
mod server;
mod uuid;
mod sql;

use anyhow::Result;
use email::Email;
use server::{fs_read, route};
use std::net::SocketAddr;
use std::str::FromStr;
use chrono::Utc;
use anyhow::anyhow;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::sql::{DataBase, DataBaseConfig, UserDB};

#[tokio::test]
async fn test_email() -> Result<()> {
    let time = Utc::now();
    println!("{}", time);

    let email_cfg = fs_read("./cfg/email.json").await?;
    let email = Email::from_cfg(&email_cfg)?;

    // email.send("somebody@gmail.com", "Hello".to_string(), "I am ChatAlone.".to_string()).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "0.0.0.0:55555";

    let conn = async {
        let config: DataBaseConfig
            = serde_json::from_str(&fs_read("./cfg/sql.json").await?)?;
        Ok(config.to_conn().await?)
    };

    let conn = conn.await
        .map_err(|e: anyhow::Error|anyhow!(format!("[Error] {}\tPlease check cfg/sql.json.", e)))?;

    let app = route(conn);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("listening on {}", listener.local_addr()?);
    Ok(axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?)
}

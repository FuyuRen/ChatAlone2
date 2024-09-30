use std::sync::Arc;
use std::time::Duration;
use anyhow::Error;
use crypto::scrypt::ScryptParams;
use futures::AsyncWriteExt;
use sea_orm::{ActiveValue, ConnectOptions, Database};
use sea_orm::prelude::*;
use sea_orm::prelude::async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::entities::{room_info, user_info};
use crate::entities::prelude::{RoomInfo, UserInfo};
use crate::uuid::UUID;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataBaseConfig {
    pub port:   u16,
    pub host:   String,
    username:   String,
    password:   String,
    schema:     String,
}

#[tokio::test]
async fn db_test() -> anyhow::Result<()> {
    use crate::server::fs_read;
    use serde_json;
    let config: DataBaseConfig = serde_json::from_str(fs_read("./cfg/sql.json")?)?;
    println!("{:?}", config);
    let db = UserDB::from_cfg(&config).await?;

    let user_table = user_info::ActiveModel {
        user_id:    ActiveValue::Set(UUID::new().into()),
        email:      ActiveValue::Set("somebody@gmail.com".to_string()),
        username:   ActiveValue::Set("ayi".to_string()),
        password:   ActiveValue::Set("114514".to_string()),
        join_time:  ActiveValue::Set("1919810".to_string()),
    };
    let res = UserInfo::insert(user_table).exec(&db.conn).await?;
    Ok(())
}

// "EntityTrait"::insert("ActiveModel")

#[async_trait]
pub trait DataBase {
    async fn from_cfg(config: &DataBaseConfig) -> Result<Self, anyhow::Error> {
        let addr = format!("postgres://{}:{}@{}:{}/database?currentSchema={}",
            config.username, config.password, config.host, config.port, config.schema);

        let mut opt = ConnectOptions::new(&addr);
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(true)
            .sqlx_logging_level(log::LevelFilter::Info)
            .set_schema_search_path(config.schema.as_ref());

        let conn = Database::connect(opt).await?;

        Ok(Self::with_conn(conn))
    }

    fn with_conn(conn: DatabaseConnection) -> Self;
    fn conn(&self) -> &DatabaseConnection;
}

#[async_trait]
pub trait CRUD<T> where T: DataBase {
    async fn insert(&self, entity: &impl EntityTrait, table: &impl ActiveModelTrait) -> Result<(), DbErr> {
        entity.insert(table).exec(self.conn()).await;
    }
}

pub struct UserDB {
    conn:   DatabaseConnection,
}

impl DataBase for UserDB {
    fn with_conn(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
    fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }

}
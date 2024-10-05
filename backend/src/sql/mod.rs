mod user;

use migration::{IntoCondition, Order};
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::prelude::*;
use sea_orm::{ConnectOptions, Database, DeleteResult, QueryOrder};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::entities::prelude::UserInfo;
use crate::entities::user_info;
pub use crate::sql::user::UserDB as UserDB;
pub use crate::sql::user::UserTable as UserTable;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataBaseConfig {
    port:       u16,
    host:       String,
    username:   String,
    password:   String,
    database:   String,
    schema:     String,
}

impl DataBaseConfig {
    pub async fn to_conn(&self) -> Result<DatabaseConnection, DbErr> {
        let addr = format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        );

        let mut opt = ConnectOptions::new(&addr);
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(true)
            .sqlx_logging_level(log::LevelFilter::Info)
            .set_schema_search_path::<&String>(&self.schema);

        Database::connect(opt).await
    }
}

#[async_trait]
pub trait DataBase {
    fn with_conn(conn: DatabaseConnection) -> Self;
    fn conn(&self) -> &DatabaseConnection;
}

#[async_trait]
pub trait BasicCRUD<T> where T: DataBase {
    type PrimaryKey;
    type Column;
    type Table;

    async fn insert(&self, model: UserTable) -> Result<Self::PrimaryKey, DbErr>;

    async fn select(&self,
                    conditions: Vec<impl IntoCondition>,
                    order: Option<(Order, Self::Column)>,
    ) -> Result<Vec<Self::Table>, DbErr>;
    async fn select_pk(&self, pk: Self::PrimaryKey) -> Result<Option<Self::Table>, DbErr>;
    async fn select_one(&self,
                        conditions: Vec<impl IntoCondition>,
    ) -> Result<Option<Self::Table>, DbErr>;

    async fn delete(&self, model: Self::Table) -> Result<DeleteResult, DbErr>;
    async fn delete_pk(&self, pk: Self::PrimaryKey) -> Result<DeleteResult, DbErr>;
    async fn delete_all(&self, conditions: Vec<impl IntoCondition>) -> Result<DeleteResult, DbErr>;
}

#[tokio::test]
async fn db_test() -> anyhow::Result<()> {
    use crate::server::fs_read;
    use serde_json;
    let config: DataBaseConfig = serde_json::from_str(&fs_read("./cfg/sql.json").await?)?;
    println!("{:?}", config);
    let db = UserDB::from_cfg(&config).await?;

    let user_table = UserTable::new(
        "somebody@gmail.com", "ayi", "114514");

    let res = db.insert(user_table).await?;
    println!("{:?}", res);
    Ok(())
}

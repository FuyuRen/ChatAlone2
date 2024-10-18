mod user;
mod room;
mod lone;
mod lone_user;
mod role;

use std::future::Future;
use migration::{IntoCondition, Order};
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::prelude::*;
use sea_orm::{ConnectOptions, Database, DeleteResult, Insert, QueryOrder};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::entities::prelude::UserInfo;
use crate::server::AppState;

pub use crate::sql::user::UserDB as UserDB;
pub use crate::sql::user::UserTable as UserTable;

pub use crate::sql::lone::LoneDB as LoneDB;
pub use crate::sql::lone::LoneTable as LoneTable;


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
    type PrimaryKey;
    type Table: From<Self::Model> + Into<Self::ActiveModel>;
    type Column: ColumnTrait;
    type ActiveModel: ActiveModelTrait<Entity = Self::Entity>;
    type Model: ModelTrait;
    type Entity: EntityTrait;

    fn with_conn(conn: DatabaseConnection) -> Self;
    fn conn(&self) -> &DatabaseConnection;
    fn from_state(state: &AppState) -> Self where Self: Sized {
        Self::with_conn(state.db_conn.clone())
    }
}

impl<T: DataBase> BasicCRUD for T {
    type PrimaryKey = T::PrimaryKey;
    type Table = T::Table;
    type Column = T::Column;
    type ActiveModel = T::ActiveModel;
    type Model = T::Model;
    type Entity = T::Entity;


    async fn insert(&self, model: Self::Table) -> Result<Self::PrimaryKey, DbErr> {
        let model: Self::ActiveModel = model.into();
        let res= Self::Entity::insert(model).exec(self.conn()).await?;
        Ok(res.last_insert_id)
    }

    async fn select(&self,
                    conditions: Vec<impl IntoCondition + Send>,
                    order: Option<(Order, Self::Column)>,
    ) -> Result<Vec<Self::Table>, DbErr> {
        let mut sel = Self::Entity::find();

        for cond in conditions {
            sel = sel.filter(cond);
        }

        Ok(
            if let Some((order, column)) = order {
                sel.order_by(column, order.clone()).all(self.conn()).await?
            } else {
                sel.all(self.conn()).await?
            }.into_iter().map(|x| Self::Table::from(x)).collect()
        )
    }

    async fn select_pk(&self,
                       pk: Self::PrimaryKey,
    ) -> Result<Option<Self::Table>, DbErr> {
        Ok(Self::Entity::find_by_id(pk).one(self.conn()).await?.map(|x| Self::Table::from(x)))
    }

    async fn select_one(&self,
                        conditions: Vec<impl IntoCondition + Send>,
    ) -> Result<Option<Self::Table>, DbErr> {
        let mut sel = Self::Entity::find();
        for cond in conditions {
            sel = sel.filter(cond);
        }
        Ok(sel.one(self.conn()).await?.map(|x| Self::Table::from(x)))
    }

    async fn delete(&self, model: Self::Table) -> Result<DeleteResult, DbErr> {
        let model: Self::ActiveModel = model.into();
        model.delete(self.conn()).await
    }

    async fn delete_pk(&self,
                       pk: Self::PrimaryKey
    ) -> Result<DeleteResult, DbErr> {
        Self::Entity::delete_by_id(pk).exec(self.conn()).await
    }

    async fn delete_many(&self,
                         conditions: Vec<impl IntoCondition + Send>
    ) -> Result<DeleteResult, DbErr> {
        let mut del = Self::Entity::delete_many();
        for cond in conditions {
            del = del.filter(cond);
        }
        del.exec(self.conn()).await
    }
}

#[async_trait]
pub trait BasicCRUD {
    type PrimaryKey;
    type Table: From<Self::Model> + Into<Self::ActiveModel>;
    type Column: ColumnTrait;
    type ActiveModel: ActiveModelTrait<Entity = Self::Entity>;
    type Model: ModelTrait;
    type Entity: EntityTrait;
    async fn insert(&self, model: Self::Table) -> Result<Self::PrimaryKey, DbErr>;
    async fn select(&self,
                    conditions: Vec<impl IntoCondition + Send>,
                    order: Option<(Order, Self::Column)>,
    ) -> Result<Vec<Self::Table>, DbErr>;
    async fn select_pk(&self,
                       pk: Self::PrimaryKey,
    ) -> Result<Option<Self::Table>, DbErr>;
    async fn select_one(&self,
                        conditions: Vec<impl IntoCondition + Send>,
    ) -> Result<Option<Self::Table>, DbErr>;
    async fn delete(&self, model: Self::Table) -> Result<DeleteResult, DbErr>;
    async fn delete_pk(&self,
                       pk: Self::PrimaryKey
    ) -> Result<DeleteResult, DbErr>;
    async fn delete_many(&self,
                         conditions: Vec<impl IntoCondition + Send>
    ) -> Result<DeleteResult, DbErr>;
}


#[tokio::test]
async fn db_test() -> anyhow::Result<()> {
    use crate::server::fs_read;
    use serde_json;
    let config: DataBaseConfig = serde_json::from_str(&fs_read("./cfg/sql.json").await?)?;
    println!("{:?}", config);
    let db = UserDB::from_cfg(&config).await?;

    let user_table = UserTable::new("somebody@gmail.com", "ayi", "114514");

    let res = db.insert(user_table).await?;
    println!("{:?}", res);
    Ok(())
}

pub(crate) mod room;
pub(crate) mod user;
pub(crate) mod lone;

use std::time::Duration;
use serde::{Deserialize, Serialize};
use sea_orm::prelude::*;
use sea_orm::{ConnectOptions, Database, Order, QueryOrder};
use sea_orm::sea_query::{SimpleExpr};

use crate::server::AppState;

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

pub trait DataBase {
    type Entity: EntityTrait<ActiveModel: Send, Model: Send>;

    fn from_conn(conn: DatabaseConnection) -> Self;
    fn from_state(state: &AppState) -> Self where Self: Sized {
        Self::from_conn(state.db_conn.clone())
    }
    fn conn(&self) -> &DatabaseConnection;
}

#[macro_export] macro_rules! database {
    ($entity:ident) => {
        use anyhow::Error;
        use sea_orm::{
            ColumnTrait,
            EntityTrait,
            PrimaryKeyTrait,
            DatabaseConnection,
        };
        
        use crate::sql::{BasicCRUD, DataBase, DataBaseConfig};
        
        pub type Entity = $entity;
        pub type Model = <$entity as EntityTrait>::Model;
        pub type Column = <$entity as EntityTrait>::Column;
        pub type ActiveModel = <$entity as EntityTrait>::ActiveModel;
        pub type PrimaryKey = <<$entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType;

        #[derive(Debug, Clone)]
        pub struct DB {
            conn: DatabaseConnection,
        }

        impl DataBase for DB {
            type Entity = $entity;

            fn from_conn(conn: DatabaseConnection) -> Self {
                Self { conn }
            }
            fn conn(&self) -> &DatabaseConnection {
                &self.conn
            }
        }

        impl DB {
            pub async fn from_cfg(config: &DataBaseConfig) -> Result<Self, Error> {
                let conn = config.to_conn().await?;
                Ok(Self::from_conn(conn))
            }
        }
    };
}



#[async_trait::async_trait]
pub trait BasicCRUD {
    type Entity: EntityTrait<ActiveModel: Send, Model: Send>;

    async fn insert(&self,
                    model: <Self::Entity as EntityTrait>::ActiveModel,
    ) -> Result<<<Self::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType, DbErr>;

    async fn select(&self,
                    conditions: Vec<SimpleExpr>,
                    order: Option<(<Self::Entity as EntityTrait>::Column, Order)>
    ) -> Result<Vec<<Self::Entity as EntityTrait>::Model>, DbErr>;

    async fn select_pk(&self,
                       pk: <<Self::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType
    ) -> Result<Option<<Self::Entity as EntityTrait>::Model>, DbErr>;

    async fn select_one(&self,
                        conditions: Vec<SimpleExpr>
    ) -> Result<Option<<Self::Entity as EntityTrait>::Model>, DbErr>;

    async fn delete(&self,
                    model: <Self::Entity as EntityTrait>::ActiveModel
    ) -> Result<u64, DbErr>;

    async fn delete_pk(&self,
                       pk: <<Self::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType
    ) -> Result<bool, DbErr>;
}

#[async_trait::async_trait]
impl<T: DataBase + Send + Sync> BasicCRUD for T {
    type Entity = T::Entity;

    /// Inserts a new Column into the database.
    ///
    /// # Arguments
    ///
    /// * `model` - The target model to be inserted, represented by `Self::Model`.
    ///
    /// # Returns
    ///
    /// * `Result<<<Self::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType, DbErr>`
    ///   - On success, returns the value of the last inserted primary key.
    ///   - On failure, returns a `DbErr` indicating the error encountered during insertion.
    ///
    /// # Notes
    ///
    /// This function converts the provided model into an active model suitable for insertion,
    /// performs the insert operation asynchronously, and retrieves the last inserted ID.
    async fn insert(&self,
                    model: <Self::Entity as EntityTrait>::ActiveModel,
    ) -> Result<<<Self::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType, DbErr> {
        
        let res = <Self::Entity as EntityTrait>::insert(model).exec(self.conn()).await?;

        Ok(res.last_insert_id)
    }


    async fn select(&self,
                    conditions: Vec<SimpleExpr>,
                    order: Option<(<Self::Entity as EntityTrait>::Column, Order)>
    ) -> Result<Vec<<Self::Entity as EntityTrait>::Model>, DbErr> {

        let mut query = <Self::Entity as EntityTrait>::find();
        for condition in conditions {
            query = query.filter(condition);
        }
        if let Some((column, order)) = order {
            query = query.order_by(column, order);
        }
        let res = query.all(self.conn()).await?;

        Ok(res)
    }

    async fn select_pk(&self,
                       pk: <<Self::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType
    ) -> Result<Option<<Self::Entity as EntityTrait>::Model>, DbErr> {

        let res =
            <Self::Entity as EntityTrait>::find_by_id(pk).one(self.conn()).await?;

        Ok(res)
    }

    async fn select_one(&self,
                        conditions: Vec<SimpleExpr>
    ) -> Result<Option<<Self::Entity as EntityTrait>::Model>, DbErr> {

        let mut query = <Self::Entity as EntityTrait>::find();
        for cond in conditions {
            query = query.filter(cond);
        }
        let res = query.one(self.conn()).await?;

        Ok(res)
    }

    async fn delete(&self,
                    model: <Self::Entity as EntityTrait>::ActiveModel
    ) -> Result<u64, DbErr> {

        let res = <Self::Entity as EntityTrait>::delete(model).exec(self.conn()).await?;

        Ok(res.rows_affected)
    }

    async fn delete_pk(&self,
                       pk: <<Self::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType
    ) -> Result<bool, DbErr> {

        let res = <Self::Entity as EntityTrait>::delete_by_id(pk).exec(self.conn()).await?;

        match res.rows_affected {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DbErr::Custom(
                "More than one row was deleted".to_string()
            )),
        }
    }
}

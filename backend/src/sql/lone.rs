use sea_orm::ColumnTrait;
use sea_orm::{
    DbErr,
    EntityTrait,
    DatabaseConnection,
};

use crate::entities::prelude::{LoneInfo};
use crate::entities::lone_info::{ActiveModel, Column, Model};
use crate::sql::{BasicCRUD, DataBase, DataBaseConfig};

pub use crate::entities::lone_info::LoneTable as LoneTable;
use crate::id::{GeneralId, UserId};

#[derive(Debug, Clone)]
pub struct LoneDB {
    conn: DatabaseConnection,
}

impl DataBase for LoneDB {
    type PrimaryKey = PrimaryKey;
    type Table = LoneTable;
    type Column = Column;
    type ActiveModel = ActiveModel;
    type Model = Model;
    type Entity = LoneInfo;

    fn with_conn(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
    fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }
}

pub type PrimaryKey = i32;

impl LoneDB {
    pub async fn from_cfg(config: &DataBaseConfig) -> Result<Self, anyhow::Error> {
        let conn = config.to_conn().await?;
        Ok(Self::with_conn(conn))
    }

    async fn select_by_owner(&self, uid: UserId) -> Result<Option<LoneTable>, DbErr> {
        self.select_one(vec![Column::OwnerId.eq(uid.decode() as i32),]).await
    }
}


#[tokio::test]
async fn test_lone_cross_search() -> anyhow::Result<()> {
    use crate::server::fs_read;
    use serde_json;
    let config: DataBaseConfig = serde_json::from_str(&fs_read("./cfg/sql.json").await?)?;
    println!("{:?}", config);
    let db = LoneDB::from_cfg(&config).await?;

    let lone_table = LoneTable::new(114514, "test");
    let lone_id = db.insert(lone_table).await?;

    println!("Insertion OK! lone_id: {}", lone_id);
    Ok(())
}
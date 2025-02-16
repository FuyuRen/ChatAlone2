use sea_orm::{
    DbErr,
    ActiveValue,
    EntityTrait,
    ActiveModelTrait,
    DatabaseConnection,
};

use crate::entities::prelude::{LoneRoleInfo};
use crate::entities::lone_role_info::{Column, ActiveModel, RolePrivilege, Model};
use crate::sql::{BasicCRUD, DataBase, DataBaseConfig};

pub use crate::entities::lone_role_info::LoneRoleTable as LoneRoleTable;
use crate::id::RoleId;

pub struct RoleDB {
    conn: DatabaseConnection,
}

impl DataBase for RoleDB {
    type PrimaryKey = PrimaryKey;
    type Table = LoneRoleTable;
    type Column = Column;
    type ActiveModel = ActiveModel;
    type Model = Model;
    type Entity = LoneRoleInfo;

    fn with_conn(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
    fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }
}

pub type PrimaryKey = i32;

impl RoleDB {
    pub async fn from_cfg(config: &DataBaseConfig) -> Result<Self, anyhow::Error> {
        let conn = config.to_conn().await?;
        Ok(Self::with_conn(conn))
    }

    pub async fn update(
        &self, role_id: RoleId,
        privilege: Option<RolePrivilege>, name: Option<&str>
    ) -> Result<(), DbErr> {

        let name =
            name.map_or(ActiveValue::NotSet, |s| ActiveValue::Set(s.to_owned()));
        let privilege =
            privilege.map_or(ActiveValue::NotSet, |s| ActiveValue::Set(s.into()));

        let model = ActiveModel {
            id:         ActiveValue::Set(role_id.into()),
            name,
            privilege,
            lone_id:    ActiveValue::NotSet,
        };
        model.update(&self.conn).await?;
        Ok(())
    }
}

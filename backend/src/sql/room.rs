use sea_orm::DatabaseConnection;
use crate::entities::prelude::RoomInfo;
use crate::entities::room_info::{ActiveModel, Column, Model, RoomTable};
use crate::sql::DataBase;

#[derive(Debug, Clone)]
pub struct RoomDB {
    conn: DatabaseConnection,
}

impl DataBase for RoomDB {
    type PrimaryKey = PrimaryKey;
    type Table = RoomTable;
    type Column = Column;
    type ActiveModel = ActiveModel;
    type Model = Model;
    type Entity = RoomInfo;

    fn with_conn(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
    fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }
}

pub type PrimaryKey = i32;
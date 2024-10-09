use sea_orm::DatabaseConnection;
use crate::sql::DataBase;

#[derive(Debug, Clone)]
pub struct RoomDB {
    conn: DatabaseConnection,
}

impl DataBase for RoomDB {
    fn with_conn(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
    fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }
}

pub type PrimaryKey = i32;
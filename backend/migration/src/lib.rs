pub use sea_orm_migration::prelude::*;

mod m20240930_000001_create_user_info_table;
mod m20240930_000002_create_room_info_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240930_000001_create_user_info_table::Migration),
            Box::new(m20240930_000002_create_room_info_table::Migration),
        ]
    }
}

pub use sea_orm_migration::prelude::*;

mod m20241006_000001_user_info;
mod m20241006_000004_room_info;
mod m20241006_000002_lone_info;
mod m20241006_000003_lone_role_info;
mod m20241006_000005_room_identity_info;
mod m20241006_000006_assoc_lone_user;
mod m20241006_000007_assoc_room_user;


pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241006_000001_user_info::Migration),
            Box::new(m20241006_000002_lone_info::Migration),
            Box::new(m20241006_000003_lone_role_info::Migration),
            Box::new(m20241006_000004_room_info::Migration),
            Box::new(m20241006_000005_room_identity_info::Migration),
            Box::new(m20241006_000006_assoc_lone_user::Migration),
            Box::new(m20241006_000007_assoc_room_user::Migration),
        ]
    }
}

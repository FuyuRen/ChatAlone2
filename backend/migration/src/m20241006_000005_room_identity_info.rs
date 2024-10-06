use sea_orm_migration::{prelude::*, schema::*};
use crate::m20241006_000004_room_info::RoomInfo;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(RoomIdentityInfo::Table).if_not_exists()
                .col(pk_auto(RoomIdentityInfo::Id))
                .col(string_len(RoomIdentityInfo::Name, 32))
                .col(integer(RoomIdentityInfo::RoomId))
                .col(big_integer(RoomIdentityInfo::Privilege))
                .to_owned()
        ).await?;
        
        manager.create_foreign_key(
            ForeignKey::create()
                .name("fk_room_id")
                .from(RoomIdentityInfo::Table,  RoomIdentityInfo::RoomId)
                .to(  RoomInfo::Table,          RoomInfo::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned()
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_foreign_key(ForeignKey::drop().table(RoomIdentityInfo::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(RoomIdentityInfo::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum RoomIdentityInfo {
    Table,
    Id,
    Name,
    RoomId,
    Privilege
}

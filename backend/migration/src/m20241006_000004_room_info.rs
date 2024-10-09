use sea_orm_migration::{prelude::*, schema::*};
use crate::m20241006_000002_lone_info::LoneInfo;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(RoomInfo::Table)
                .if_not_exists()
                .col(pk_auto(RoomInfo::Id))
                .col(integer(RoomInfo::LoneId))
                .col(string_len(RoomInfo::Name, 32))
                .col(char(RoomInfo::RoomType).default(Expr::val(0)))
                
                .col(timestamp(RoomInfo::CreatedAt).default(Expr::current_timestamp()))
                .to_owned(),
        ).await?;

        manager.create_foreign_key(
            ForeignKey::create()
                .name("fk_lone_id")
                .from(RoomInfo::Table, RoomInfo::LoneId)
                .to(  LoneInfo::Table, LoneInfo::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned()
        ).await?;
        
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_foreign_key(ForeignKey::drop().table(RoomInfo::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(RoomInfo::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum RoomInfo {
    Table,
    Id,
    Name,
    LoneId,
    RoomType,
    CreatedAt,
}

use sea_orm_migration::{prelude::*, schema::*};
use crate::m20241006_000001_user_info::UserInfo;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(LoneInfo::Table)
                .if_not_exists()
                .col(pk_auto(LoneInfo::Id))
                .col(string_len(LoneInfo::Name, 32))
                .col(integer(LoneInfo::OwnerId))
                
                .col(timestamp(LoneInfo::CreatedAt).default(Expr::current_timestamp()))
                .to_owned(),
        ).await?;

        manager.create_foreign_key(
            ForeignKey::create()
                .name("fk_owner_id")
                .from(LoneInfo::Table, LoneInfo::OwnerId)
                .to(  UserInfo::Table, UserInfo::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned()
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_foreign_key(ForeignKey::drop().table(LoneInfo::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(LoneInfo::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum LoneInfo {
    Table,
    Id,
    Name,
    OwnerId,
    CreatedAt,
}

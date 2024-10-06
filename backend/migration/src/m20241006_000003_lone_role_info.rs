use sea_orm_migration::{prelude::*, schema::*};
use crate::m20241006_000002_lone_info::LoneInfo;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(LoneRoleInfo::Table)
                .if_not_exists()
                .col(pk_auto(LoneRoleInfo::Id))
                .col(string_len(LoneRoleInfo::Name, 32))
                .col(integer(LoneRoleInfo::LoneId))
                .col(big_integer(LoneRoleInfo::Privilege))
                .to_owned()
        ).await?;

        manager.create_foreign_key(
            ForeignKey::create()
                .name("fk_lone_id")
                .from(LoneRoleInfo::Table, LoneRoleInfo::LoneId)
                .to(  LoneInfo::Table,     LoneInfo::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned()
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_foreign_key(ForeignKey::drop().table(LoneRoleInfo::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(LoneRoleInfo::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum LoneRoleInfo {
    Table,
    Id,
    Name,
    LoneId, 
    Privilege
}

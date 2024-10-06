use sea_orm_migration::{prelude::*, schema::*};
use crate::m20241006_000001_user_info::UserInfo;
use crate::m20241006_000002_lone_info::LoneInfo;
use crate::m20241006_000003_lone_role_info::LoneRoleInfo;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(AssocLoneUser::Table).if_not_exists()
                .col(integer(AssocLoneUser::LoneId))
                .col(integer(AssocLoneUser::UserId))
                .col(integer(AssocLoneUser::RoleId))
                .primary_key(Index::create()
                    .col(AssocLoneUser::LoneId)
                    .col(AssocLoneUser::UserId)
                    .col(AssocLoneUser::RoleId))
                .to_owned()
        ).await?;

        manager.create_foreign_key(
            ForeignKey::create()
                .name("fk_lone_id")
                .from(AssocLoneUser::Table, AssocLoneUser::LoneId)
                .to(  LoneInfo::Table,      LoneInfo::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned()
        ).await?;

        manager.create_foreign_key(
            ForeignKey::create()
                .name("fk_user_id")
                .from(AssocLoneUser::Table, AssocLoneUser::UserId)
                .to(  UserInfo::Table,      UserInfo::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned()
        ).await?;

        manager.create_foreign_key(
            ForeignKey::create()
                .name("fk_role_id")
                .from(AssocLoneUser::Table, AssocLoneUser::RoleId)
                .to(  LoneRoleInfo::Table,  LoneRoleInfo::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned()
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_foreign_key(ForeignKey::drop().table(AssocLoneUser::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(AssocLoneUser::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum AssocLoneUser {
    Table,
    LoneId,
    UserId,
    RoleId,
}

use sea_orm_migration::{prelude::*, schema::*};
use crate::m20241006_000001_user_info::UserInfo;
use crate::m20241006_000004_room_info::RoomInfo;
use crate::m20241006_000005_room_identity_info::RoomIdentityInfo;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(AssocRoomUser::Table).if_not_exists()
                .col(integer(AssocRoomUser::RoomId))
                .col(integer(AssocRoomUser::UserId))
                .col(integer(AssocRoomUser::IdenId))
                .primary_key(Index::create()
                                .col(AssocRoomUser::RoomId)
                                .col(AssocRoomUser::UserId)
                                .col(AssocRoomUser::IdenId))
                .to_owned()
        ).await?;
        
        manager.create_foreign_key(
            ForeignKey::create()
                .name("fk_room_id")
                .from(AssocRoomUser::Table, AssocRoomUser::RoomId)
                .to(  RoomInfo::Table,      RoomInfo::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned()
        ).await?;

        manager.create_foreign_key(
            ForeignKey::create()
                .name("fk_user_id")
                .from(AssocRoomUser::Table, AssocRoomUser::UserId)
                .to(  UserInfo::Table,      UserInfo::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned()
        ).await?;

        manager.create_foreign_key(
            ForeignKey::create()
                .name("fk_role_id")
                .from(AssocRoomUser::Table,     AssocRoomUser::IdenId)
                .to(  RoomIdentityInfo::Table,  RoomIdentityInfo::Id)
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade)
                .to_owned()
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_foreign_key(ForeignKey::drop().table(AssocRoomUser::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(AssocRoomUser::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum AssocRoomUser {
    Table,
    RoomId,
    UserId,
    IdenId,
}

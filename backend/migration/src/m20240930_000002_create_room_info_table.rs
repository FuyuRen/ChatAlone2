use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    //  CREATE TABLE IF NOT EXISTS ChatAlone2.room_info(
    //      room_id     INTEGER         NOT NULL,
    //      room_name   VARCHAR(32)     NOT NULL,
    //      create_time TIMESTAMP       NOT NULL
    //      is_vip      BOOLEAN         NOT NULL,
    //  )

    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RoomInfo::Table)
                    .col(
                        ColumnDef::new(RoomInfo::RoomId)
                            .big_unsigned()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(RoomInfo::RoomName).string().not_null())
                    .col(ColumnDef::new(RoomInfo::CreateTime).timestamp().not_null())
                    // .col(ColumnDef::new(RoomInfo::IsVIP).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RoomInfo::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum RoomInfo {
    Table,
    RoomId,
    RoomName,
    // IsVIP,
    CreateTime,
}

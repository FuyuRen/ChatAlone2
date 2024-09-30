use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {

    //  CREATE TABLE IF NOT EXISTS ChatAlone2.user_info(
    //      user_id     INTEGER         NOT NULL        PRIMARY KEY ,
    //      email       VARCHAR(32)     NOT NULL        UNIQUE      ,
    //      username    VARCHAR(32)     NOT NULL                    ,
    //      password    VARCHAR(32)     NOT NULL                    ,
    //      join_time   TIMESTAMP       NOT NULL
    //  )

    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserInfo::Table)
                    .col(ColumnDef::new(UserInfo::UserId).big_unsigned().not_null().primary_key())
                    .col(ColumnDef::new(UserInfo::Email).string().not_null().unique_key())
                    .col(ColumnDef::new(UserInfo::Username).string().not_null())
                    .col(ColumnDef::new(UserInfo::Password).string().not_null())
                    .col(ColumnDef::new(UserInfo::JoinTime).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserInfo::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserInfo {
    Table,
    UserId,
    Email,
    Username,
    Password,
    JoinTime,
}

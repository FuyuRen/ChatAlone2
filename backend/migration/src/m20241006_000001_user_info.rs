use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(UserInfo::Table)
                .if_not_exists()
                .col(pk_auto(UserInfo::Id))
                .col(string_len(UserInfo::Username, 32))
                .col(string_len(UserInfo::Password, 64))
                .col(string_len_uniq(UserInfo::Email, 64))
                
                .col(timestamp(UserInfo::CreatedAt).default(Expr::current_timestamp()))
                .to_owned()
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(UserInfo::Table).to_owned()).await?;
        manager.drop_foreign_key(ForeignKey::drop().table(UserInfo::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum UserInfo {
    Table,
    Id,
    Email,
    Username,
    Password,
    CreatedAt,
}

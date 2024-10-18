use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};
use sea_orm::DbErr;
use crate::entities::assoc_lone_user::ActiveModel;
use crate::entities::prelude::AssocLoneUser;
use crate::id::{GeneralId, LoneId, RoleId, UserId};

async fn insert(
    conn: &DatabaseConnection,
    lone_id: LoneId, user_id: UserId, role_id: RoleId
) -> Result<(), DbErr> {
    let model = ActiveModel {
        lone_id: ActiveValue::Set(lone_id.decode() as i32),
        user_id: ActiveValue::Set(user_id.decode() as i32),
        role_id: ActiveValue::Set(role_id.decode() as i32),
    };
    AssocLoneUser::insert(model).exec(conn).await?;
    Ok(())
}
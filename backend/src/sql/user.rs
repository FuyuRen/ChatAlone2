use crate::entities::prelude::UserInfo;
use crate::entities::user_info;

use anyhow::Error;
use migration::IntoCondition;
use sea_orm::prelude::*;
use sea_orm::{DeleteResult, Order, QueryOrder};
use crate::entities::room_info::Column;
use crate::sql::{BasicCRUD, DataBase, DataBaseConfig};
pub use crate::entities::user_info::UserTable as UserTable;

#[derive(Debug, Clone)]
pub struct UserDB {
    conn: DatabaseConnection,
}

impl DataBase for UserDB {
    fn with_conn(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
    fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }
}

type PrimaryKey = i32;

impl UserDB {
    pub async fn from_cfg(config: &DataBaseConfig) -> Result<Self, Error> {
        let conn = config.to_conn().await?;
        Ok(Self::with_conn(conn))
    }

    pub async fn select_email(&self, email: &str) -> Result<Option<UserTable>, DbErr> {
        Ok(UserInfo::find()
            .filter(user_info::Column::Email.eq(email))
            .one(&self.conn).await?
            .map(|x| UserTable::from(x))
        )
    }

    pub async fn insert(&self, model: UserTable) -> Result<PrimaryKey, DbErr> {
        let model: user_info::ActiveModel = model.into();
        let res= UserInfo::insert(model).exec(&self.conn).await?;
        Ok(res.last_insert_id)
    }

    pub async fn select(&self,
                        conditions: Vec<impl IntoCondition>,
                        order: Option<(Order, user_info::Column)>,
    ) -> Result<Vec<UserTable>, DbErr> {
        let mut sel = UserInfo::find();
        for cond in conditions {
            sel = sel.filter(cond);
        }

        Ok(
            if let Some((order, column)) = order {
                sel.order_by(column, order.clone()).all(&self.conn).await?
            } else {
                sel.all(&self.conn).await?
            }.into_iter().map(|x| UserTable::from(x)).collect()
        )
    }

    pub async fn select_pk(&self,
                           pk: PrimaryKey,
    ) -> Result<Option<UserTable>, DbErr> {
        Ok(UserInfo::find_by_id(pk).one(&self.conn).await?.map(|x| UserTable::from(x)))
    }

    pub async fn select_one(&self,
                            conditions: Vec<impl IntoCondition>,
    ) -> Result<Option<UserTable>, DbErr> {
        let mut sel = UserInfo::find();
        for cond in conditions {
            sel = sel.filter(cond);
        }
        Ok(sel.one(&self.conn).await?.map(|x| UserTable::from(x)))
    }

    pub async fn delete(&self, model: UserTable) -> Result<DeleteResult, DbErr> {
        let model: user_info::ActiveModel = model.into();
        model.delete(&self.conn).await
    }

    pub async fn delete_pk(&self,
                           pk: PrimaryKey
    ) -> Result<DeleteResult, DbErr> {
        UserInfo::delete_by_id(pk).exec(&self.conn).await
    }

    pub async fn delete_many(&self,
                             conditions: Vec<impl IntoCondition>
    ) -> Result<DeleteResult, DbErr> {
        let mut del = UserInfo::delete_many();
        for cond in conditions {
            del = del.filter(cond);
        }
        del.exec(&self.conn).await
    }
}
//
// impl BasicCRUD<UserDB> for UserDB {
//     type PrimaryKey = PrimaryKey;
//     type Column = Column;
//     type Table = UserTable;
//
//     async fn insert(&self, model: UserTable) -> Result<PrimaryKey, DbErr> {
//         let model: user_info::ActiveModel = model.into();
//         let res= UserInfo::insert(model).exec(&self.conn).await?;
//         Ok(res.last_insert_id)
//     }
//
//     async fn select(&self,
//                     conditions: Vec<impl IntoCondition>,
//                     order: Option<(Order, user_info::Column)>,
//     ) -> Result<Vec<UserTable>, DbErr> {
//         let mut sel = UserInfo::find();
//         for cond in conditions {
//             sel = sel.filter(cond);
//         }
//
//         Ok(
//             if let Some((order, column)) = order {
//                 sel.order_by(column, order.clone()).all(&self.conn).await?
//             } else {
//                 sel.all(&self.conn).await?
//             }.into_iter().map(|x| UserTable::from(x)).collect()
//         )
//     }
//
//     async fn select_pk(&self,
//                        pk: PrimaryKey,
//     ) -> Result<Option<UserTable>, DbErr> {
//         Ok(UserInfo::find_by_id(pk).one(&self.conn).await?.map(|x| UserTable::from(x)))
//     }
//
//     async fn select_one(&self,
//                         conditions: Vec<impl IntoCondition>,
//     ) -> Result<Option<UserTable>, DbErr> {
//         let mut sel = UserInfo::find();
//         for cond in conditions {
//             sel = sel.filter(cond);
//         }
//         Ok(sel.one(&self.conn).await?.map(|x| UserTable::from(x)))
//     }
//
//     async fn delete(&self, model: UserTable) -> Result<DeleteResult, DbErr> {
//         let model: user_info::ActiveModel = model.into();
//         model.delete(&self.conn).await
//     }
//
//     async fn delete_pk(&self,
//                        pk: PrimaryKey
//     ) -> Result<DeleteResult, DbErr> {
//         UserInfo::delete_by_id(pk).exec(&self.conn).await
//     }
//
//     async fn delete_all(&self,
//                         conditions: Vec<impl IntoCondition>
//     ) -> Result<DeleteResult, DbErr> {
//         let mut del = UserInfo::delete_many();
//         for cond in conditions {
//             del = del.filter(cond);
//         }
//         del.exec(&self.conn).await
//     }
// }

use std::future::Future;
use anyhow::Error;
use migration::IntoCondition;
use sea_orm::prelude::*;
use sea_orm::{ActiveValue, DeleteResult, Insert, Order, QueryOrder};
use sea_orm::prelude::async_trait::async_trait;

use crate::entities::prelude::{LoneInfo, UserInfo};
use crate::entities::user_info::{Column, ActiveModel, Model};
use crate::sql::{BasicCRUD, DataBase, DataBaseConfig};

pub use crate::entities::user_info::UserTable as UserTable;
use crate::id::UserId;

#[derive(Debug, Clone)]
pub struct UserDB {
    conn: DatabaseConnection,
}

impl DataBase for UserDB {
    type PrimaryKey = PrimaryKey;
    type Table = UserTable;
    type Column = Column;
    type ActiveModel = ActiveModel;
    type Model = Model;
    type Entity = UserInfo;

    fn with_conn(conn: DatabaseConnection) -> Self {
        Self { conn }
    }
    fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }

}

pub type PrimaryKey = i32;

impl UserDB {
    pub async fn from_cfg(config: &DataBaseConfig) -> Result<Self, Error> {
        let conn = config.to_conn().await?;
        Ok(Self::with_conn(conn))
    }


    pub async fn select_by_email(&self, email: &str) -> Result<Option<UserTable>, DbErr> {
        Ok(UserInfo::find()
            .filter(Column::Email.eq(email))
            .one(&self.conn).await?
            .map(|x| UserTable::from(x))
        )
    }


    // pub async fn insert(&self, model: UserTable) -> Result<PrimaryKey, DbErr> {
    //     let model: ActiveModel = model.into();
    //     let res= UserInfo::insert(model).exec(&self.conn).await?;
    //     Ok(res.last_insert_id)
    // }
    
    pub async fn update(
        &self, id: PrimaryKey,
        username: Option<&str>,
        password: Option<&str>
    ) -> Result<(), DbErr> {
        if username.is_none() && password.is_none() { return Ok(()) }
        
        let username =
            username.map_or(ActiveValue::NotSet, |s| ActiveValue::Set(s.to_owned()));
        let password = 
            password.map_or(ActiveValue::NotSet, |s| ActiveValue::Set(s.to_owned()));
        
        let model = ActiveModel {
            id:         ActiveValue::Set(id),
            username,
            password,
            email:      ActiveValue::NotSet,
            created_at: ActiveValue::NotSet,
        };
        
        model.update(&self.conn).await?;
        Ok(())
    }

    // pub async fn select(&self,
    //                     conditions: Vec<impl IntoCondition>,
    //                     order: Option<(Order, user_info::Column)>,
    // ) -> Result<Vec<UserTable>, DbErr> {
    //     let mut sel = UserInfo::find();
    //     for cond in conditions {
    //         sel = sel.filter(cond);
    //     }
    // 
    //     Ok(
    //         if let Some((order, column)) = order {
    //             sel.order_by(column, order.clone()).all(&self.conn).await?
    //         } else {
    //             sel.all(&self.conn).await?
    //         }.into_iter().map(|x| UserTable::from(x)).collect()
    //     )
    // // }
    // 
    // pub async fn select_pk(&self,
    //                        pk: PrimaryKey,
    // ) -> Result<Option<UserTable>, DbErr> {
    //     Ok(UserInfo::find_by_id(pk).one(&self.conn).await?.map(|x| UserTable::from(x)))
    // }
    // 
    // pub async fn select_one(&self,
    //                         conditions: Vec<impl IntoCondition>,
    // ) -> Result<Option<UserTable>, DbErr> {
    //     let mut sel = UserInfo::find();
    //     for cond in conditions {
    //         sel = sel.filter(cond);
    //     }
    //     Ok(sel.one(&self.conn).await?.map(|x| UserTable::from(x)))
    // }
    // 
    // pub async fn delete(&self, model: UserTable) -> Result<DeleteResult, DbErr> {
    //     let model: ActiveModel = model.into();
    //     model.delete(&self.conn).await
    // }
    // 
    // pub async fn delete_pk(&self,
    //                        pk: PrimaryKey
    // ) -> Result<DeleteResult, DbErr> {
    //     UserInfo::delete_by_id(pk).exec(&self.conn).await
    // }
    // 
    // pub async fn delete_many(&self,
    //                          conditions: Vec<impl IntoCondition>
    // ) -> Result<DeleteResult, DbErr> {
    //     let mut del = UserInfo::delete_many();
    //     for cond in conditions {
    //         del = del.filter(cond);
    //     }
    //     del.exec(&self.conn).await
    // }
}
//
//
// #[async_trait]
// impl BasicCRUD for UserDB {
//     type PrimaryKey = PrimaryKey;
//     type Column = Column;
//     type Table = UserTable;
//
//     async fn insert(&self, model: UserTable) -> Result<PrimaryKey, DbErr> {
//         let model: ActiveModel = model.into();
//         let res= UserInfo::insert(model).exec(&self.conn).await?;
//         Ok(res.last_insert_id)
//     }
//
//     async fn select(&self,
//                     conditions: Vec<impl IntoCondition + Send>,
//                     order: Option<(Order, Column)>,
//     ) -> Result<Vec<UserTable>, DbErr> {
//         let mut sel = UserInfo::find();
//
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
//                         conditions: Vec<impl IntoCondition + Send>,
//     ) -> Result<Option<UserTable>, DbErr> {
//         let mut sel = UserInfo::find();
//         for cond in conditions {
//             sel = sel.filter(cond);
//         }
//         Ok(sel.one(&self.conn).await?.map(|x| UserTable::from(x)))
//     }
//
//     async fn delete(&self, model: UserTable) -> Result<DeleteResult, DbErr> {
//         let model: ActiveModel = model.into();
//         model.delete(&self.conn).await
//     }
//
//     async fn delete_pk(&self,
//                        pk: PrimaryKey
//     ) -> Result<DeleteResult, DbErr> {
//         UserInfo::delete_by_id(pk).exec(&self.conn).await
//     }
//
//     async fn delete_many(&self,
//                         conditions: Vec<impl IntoCondition + Send>
//     ) -> Result<DeleteResult, DbErr> {
//         let mut del = UserInfo::delete_many();
//         for cond in conditions {
//             del = del.filter(cond);
//         }
//         del.exec(&self.conn).await
//     }
// }

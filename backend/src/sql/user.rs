use crate::entities::prelude::UserInfo;
crate::database!(UserInfo);

impl DB {
    pub async fn select_email(&self, email: &str) -> Result<Option<Model>, Error> {
        let model 
            = self.select_one(vec![Column::Email.eq(email)]).await?;
        Ok(model)
    }
}


#[tokio::test]
async fn db_test() -> anyhow::Result<()> {
    use sea_orm::IntoActiveModel;
    use crate::entities::user_info::ActiveModel;
    use crate::sql::BasicCRUD;
    
    let config: DataBaseConfig =
        serde_json::from_str(&tokio::fs::read_to_string("./cfg/sql.json").await?)?;

    println!("{:?}", config);
    let db = DB::from_cfg(&config).await?;

    // ------------ insert test ---------------
    let model = ActiveModel {
        username:   sea_orm::ActiveValue::Set("fuyu".to_string()),
        password:   sea_orm::ActiveValue::Set("114514".to_string()),
        email:      sea_orm::ActiveValue::Set("sb@chatalone.asia".to_string()),
        ..Default::default()
    };
    let last_insert_id = db.insert(model).await?;
    // assert_eq!(last_insert_id, 1);

    let model = ActiveModel {
        username:   sea_orm::ActiveValue::Set("ayi".to_string()),
        password:   sea_orm::ActiveValue::Set("114514".to_string()),
        email:      sea_orm::ActiveValue::Set("ayi@chatalone.asia".to_string()),
        ..Default::default()
    };
    let last_insert_id = db.insert(model).await?;
    // assert_eq!(last_insert_id, 2);
    

    // // ------------ select test ---------------
    // let user_table = db.select(vec![], Some((user_info::Column::Id, Order::Asc))).await?;
    // let user_table = db.select_pk(0).await?;
    // let user_table = db.select_one(vec![]).await?;
    // println!("{:?}", user_table);
    
    // ------------ delete test ---------------
    let user_table = db.select_pk(1).await?;
    if let Some(user_table) = user_table {
        let res = db.delete(user_table.into_active_model()).await?;
        println!("del {:?} user", res);
    } else {
        panic!("No user found");
    }

    let user_table = db.select_pk(1).await?;
    if let Some(_) = user_table {
        panic!("Del Failed")
    } else {
        println!("No user found")
    }
    
    let del_res = db.delete_pk(2).await?;
    assert!(del_res, "User not found");
    
    let user_table = db.delete_pk(2).await?;
    assert!(!user_table, "Del Failed");

    Ok(())
}
use std::{
    fmt::Display,
    str::FromStr,
    sync::Arc,
};
use std::fmt::format;
use anyhow::{Result, anyhow};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::json;
use tokio::sync::Mutex;
use crate::uuid::UUID;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserTable {
    pub register_time:  i64,
    pub userid:         UUID,
    pub email:          String,
    pub username:       String,
    pub password:       String,
}

impl UserTable {
    pub fn new(email: &str, username: &str, password: &str) -> Self {
        UserTable {
            register_time:  time::OffsetDateTime::now_utc().unix_timestamp(),
            userid:         UUID::new(),
            email:          email.to_string(),
            username:       username.to_string(),
            password:       password.to_string(),
        }
    }
    pub fn verify_password(&self, password: &String) -> bool {
        self.password.eq(password)
    }
}


#[derive(Debug, Clone)]
pub struct UserDB {
    db_name:    &'static str,
    sql:        SqlConn,
}

impl UserDB {
    pub async fn init(db_name: &'static str, sql_conn: SqlConn) -> Result<Self> {
        sql_conn.0.lock().await.execute(
            format!(
                "CREATE TABLE if not exists {db_name} (
                    userid          integer  primary key    NOT NULL,
                    register_time   int                     NOT NULL,
                    email           text                    NOT NULL,
                    username        text                    NOT NULL,
                    password        text                    NOT NULL
                )"
            ).as_str(),
            (),
        )?;

        Ok(Self {
            db_name,
            sql: sql_conn,
        })
    }

    pub async fn insert(&self, t: &UserTable) -> Result<()> {
        let table_name = self.db_name;
        self.sql.0.lock().await.execute(
            format!(
                "INSERT INTO {table_name} (userid, email, register_time, username, password)
                 VALUES (?1, ?2, ?3, ?4, ?5)"
            ).as_str(),
            params![
                i64::from(&t.userid),
                t.email.as_str(),
                t.register_time,
                t.username.as_str(),
                t.password.as_str()
            ]
        )?;
        Ok(())
    }

    pub async fn select(&self, email: &str) -> Result<UserTable> {
        let name = self.db_name;
        let conn = self.sql.0.lock().await;
        let mut stmt = conn.prepare(
            format!(
                "SELECT userid, register_time, email, username, password FROM {name}
                 WHERE email = ?1"
            ).as_str()
        )?;
        let mut rows = stmt.query(params![email])?;
        // only first one
        let row = rows.next()?;

        if let Some(row) = row {
            let userid: i64 = row.get(0)?;

            Ok(UserTable {
                userid: UUID::from(userid),
                register_time: row.get(1)?,
                email: row.get(2)?,
                username: row.get(3)?,
                password: row.get(4)?,
            })
        } else {
            Err(anyhow!("user not found"))
        }
    }
}

#[derive(Debug)]
pub struct SqlConn(Arc<Mutex<Connection>>);
impl Clone for SqlConn {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl SqlConn {
    pub fn new() -> Result<Self> {
        let c = Connection::open("chatAlone.db")?;
        let sql = SqlConn(Arc::new(Mutex::new(c)));
        Ok(sql)
    }

    pub async fn drop_table(&self, name: &str) -> Result<()> {
        let conn = self.0.lock().await;
        conn.execute(format!("DROP TABLE {name}").as_str(), ())?;
        Ok(())
    }

    pub fn conn(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.0)
    }
}

#[test]
fn test_user_table_serialize_deserialize() {
    let user1 = UserTable {
        register_time: 0,
        userid: UUID::new(),
        email: "no-reply@chatalone.asia".to_string(),
        username: "admin".to_string(),
        password: "foobar".to_string(),
    };
    let json = json!(user1).to_string();
    let user2: UserTable = serde_json::from_str(json.as_str()).unwrap();
    println!("{:?} {:?}", user1, user2);
}

#[tokio::test]
async fn test_user_table_insert_select() {
    let sql = SqlConn::new().unwrap();
    let user_db = UserDB::init("user", sql.clone()).await.unwrap();

    let time = time::OffsetDateTime::now_utc();
    let time = (time.unix_timestamp_nanos() / 1_000_000) as i64;

    let user1 = UserTable {
        register_time: time,
        userid: UUID::new(),
        email: "no-reply@chatalone.asia".to_string(),
        username: "admin".to_string(),
        password: "foobar".to_string(),
    };
    println!("{:?}", user1.userid);
    user_db.insert(&user1).await.unwrap();

    let user2 = user_db.select("sb").await;
    println!("{:?}", user2);
}


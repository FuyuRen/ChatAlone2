use std::{
    fmt::Display,
    str::FromStr,
    sync::Arc,
};

use nanoid::nanoid;
use anyhow::{Result, anyhow};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json::json;

const NANOID_LEN: usize = 16;
const TABLE_NAME: &str = "user";


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserID(Vec<u8>);

impl UserID {
    pub fn new() -> Self {
        let alphabet: [char; 16] = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'
        ];
        let ret = nanoid!(NANOID_LEN, &alphabet);
        Self::from_str(ret.as_str()).unwrap()
    }
}

impl FromStr for UserID {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let ret = s.to_string();
        if ret.len().ne(&NANOID_LEN) {
            return Err(anyhow!("长度不对☝️"));
        }
        if !ret.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(anyhow!("字符不对☝️"));
        }

        let ret = ret.to_ascii_uppercase().bytes()
            .map(|b| if b > 64 {b - 55} else {b - 48} ).collect::<Vec<u8>>()
            .chunks(2).map(|c| (c[0] << 4)|c[1] )
            .collect::<Vec<u8>>();

        Ok(Self(ret))
    }
}

impl Display for UserID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.iter().map(|b| format!("{:02x}", b)).collect::<String>())
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UserTable {
    register_time:  i64,
    userid:         UserID,
    username:       String,
    password:       String,
}

struct Sql {
    conn: Arc<Connection>
}
impl Clone for Sql {
    fn clone(&self) -> Self {
        Sql { conn: Arc::clone(&self.conn) }
    }
}

impl Sql {
    pub fn new() -> Result<Self> {
        let c = Connection::open("chatAlone.db")?;
        let sql = Sql { conn: Arc::from(c) };
        sql.init()?;
        Ok(sql)
    }

    pub fn init(&self) -> Result<()> {
        self.conn.execute(
            format!(
                "CREATE TABLE if not exists {TABLE_NAME} (
                    userid          integer  primary key    AUTOINCREMENT,
                    register_time   int                     NOT NULL,
                    username        text                    NOT NULL,
                    password        text                    NOT NULL
                )"
            ).as_str(),
            (),
        )?;

        Ok(())
    }

    pub fn insert(&self, t: &UserTable) -> Result<()> {
        self.conn.execute(
            format!(
                "INSERT INTO {TABLE_NAME} (register_time, username, password)
                 VALUES (?1, ?2, ?3)"
            ).as_str(),
            params![t.register_time, t.username.as_str(), t.password.as_str()]
        )?;
        Ok(())
    }

    // pub fn select(&self, username: &str) -> Result<UserTable> {
    //     let mut stmt = self.conn.prepare(
    //         format!(
    //             "SELECT register_time, username, password
    //              FROM {TABLE_NAME}
    //              WHERE username = ?1"
    //         ).as_str()
    //     )?;
    //     let mut rows = stmt.query(params![username])?;
    //     if let Some(row) = rows.next()? {
    //         let user = UserTable {
    //             register_time:row.get(0)?,
    //         }
    //     };
    //
    // }
}

#[test]
fn main() {
    let user = UserTable {
        register_time: 0,
        userid: UserID::new(),
        username: "test".to_string(),
        password: "test".to_string(),
    };
    println!("{}", json!(user))
}


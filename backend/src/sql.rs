use std::{
    fmt::Display,
    str::FromStr,
    sync::Arc,
};
use nanoid::nanoid;
use anyhow::{Result, anyhow};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::json;

const NANOID_LEN: usize = 2*8;
const TABLE_NAME: &str = "user";


#[derive(Debug, Clone)]
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

impl From<i64> for UserID {
    fn from(value: i64) -> Self {
        let value = value as u64;
        let mut ret: Vec<u8> = vec![];
        for i in (0..NANOID_LEN/2).rev() {
            ret.push(((value >> (8*i)) & 0xff) as u8);
        }

        Self(ret)
    }
}

impl From<&UserID> for i64 {
    fn from(userid: &UserID) -> Self {
        let mut ret = 0u64;
        for (i, b) in userid.0.iter().enumerate() {
            ret |= (*b as u64) << (8*((NANOID_LEN/2 - 1) - i));
        }
        ret as i64
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

impl Serialize for UserID {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where S: Serializer {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for UserID {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        UserID::from_str(s.as_str()).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct UserTable {
    register_time:  i64,
    userid:         UserID,
    email:          String,
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
                    userid          integer  primary key    NOT NULL,
                    register_time   int                     NOT NULL,
                    email           text                    NOT NULL,
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
                "INSERT INTO {TABLE_NAME} (userid, email, register_time, username, password)
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

    pub fn select(&self, username: &str) -> Result<UserTable> {
        let mut stmt = self.conn.prepare(
            format!(
                "SELECT userid, register_time, email, username, password
                 FROM {TABLE_NAME}
                 WHERE username = ?1"
            ).as_str()
        )?;
        let mut rows = stmt.query(params![username])?;
            let row = rows.next()?;

            if let Some(row) = row {
                let userid: i64 = row.get(0)?;

                Ok(UserTable {
                    userid: UserID::from(userid),
                    register_time: row.get(1)?,
                    email: row.get(2)?,
                    username: row.get(3)?,
                    password: row.get(4)?,
                })
            } else {
                Err(anyhow!("用户不存在"))
            }
    }
}

#[test]
fn test_user_table_serialize_deserialize() {
    let user1 = UserTable {
        register_time: 0,
        userid: UserID::new(),
        email: "no-reply@chatalone.asia".to_string(),
        username: "admin".to_string(),
        password: "foobar".to_string(),
    };
    let json = json!(user1).to_string();
    let user2: UserTable = serde_json::from_str(json.as_str()).unwrap();
    println!("{:?} {:?}", user1, user2);
}

#[test]
fn test_user_table_insert_select() {
    let sql = Sql::new().unwrap();
    sql.init().unwrap();

    let time = time::OffsetDateTime::now_utc();
    let time = (time.unix_timestamp_nanos() / 1_000_000) as i64;

    let user1 = UserTable {
        register_time: time,
        userid: UserID::new(),
        email: "no-reply@chatalone.asia".to_string(),
        username: "admin".to_string(),
        password: "foobar".to_string(),
    };
    println!("{:?}", user1.userid);
    sql.insert(&user1).unwrap();

    let user2 = sql.select("admin").unwrap();
    println!("{:?}", user2);
}


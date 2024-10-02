use anyhow::{anyhow, Result};
use nanoid::nanoid;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt::Display;
use std::str::FromStr;

const NANOID_LEN: usize = 2 * 8;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct UUID(Vec<u8>);

impl UUID {
    pub fn new() -> Self {
        let alphabet: [char; 16] = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
        ];
        let ret = nanoid!(NANOID_LEN, &alphabet);
        Self::from_str(ret.as_str()).unwrap()
    }
}

impl From<i64> for UUID {
    fn from(value: i64) -> Self {
        let value = value as u64;
        let mut ret: Vec<u8> = vec![];
        for i in (0..NANOID_LEN / 2).rev() {
            ret.push(((value >> (8 * i)) & 0xff) as u8);
        }

        Self(ret)
    }
}

impl From<&UUID> for i64 {
    fn from(userid: &UUID) -> Self {
        let mut ret = 0u64;
        for (i, b) in userid.0.iter().enumerate() {
            ret |= (*b as u64) << (8 * ((NANOID_LEN / 2 - 1) - i));
        }
        ret as i64
    }
}

impl Into<usize> for UUID {
    fn into(self) -> usize {
        let mut ret = 0usize;
        for (i, b) in self.0.iter().enumerate() {
            ret |= (*b as usize) << (8 * ((NANOID_LEN / 2 - 1) - i));
        }
        ret
    }
}

impl Into<i64> for UUID {
    fn into(self) -> i64 {
        let ret: usize = self.into();
        ret as i64
    }
}

impl FromStr for UUID {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let ret = s.to_string();
        if ret.len().ne(&NANOID_LEN) {
            return Err(anyhow!("长度不对☝️"));
        }
        if !ret.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(anyhow!("字符不对☝️"));
        }

        let ret = ret
            .to_ascii_uppercase()
            .bytes()
            .map(|b| if b > 64 { b - 55 } else { b - 48 })
            .collect::<Vec<u8>>()
            .chunks(2)
            .map(|c| (c[0] << 4) | c[1])
            .collect::<Vec<u8>>();

        Ok(Self(ret))
    }
}

impl Display for UUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>()
        )
    }
}

impl Serialize for UUID {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for UUID {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        UUID::from_str(s.as_str()).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

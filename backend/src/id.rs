use std::fmt::Display;

const USER_ID_BIT_MAP :[u8; 32] = [
    31, 30, 29, 28, 27, 26, 25, 24,
    18, 22, 21, 20, 23, 17, 19, 16,
    12, 14, 11, 13,  9,  8, 15, 10,
     6,  5,  7,  4,  3,  0,  1,  2
];

const USER_ID_BIT_MAP_REV :[u8; 32] = [
    31, 30, 29, 28, 27, 26, 25, 24,
    19, 22, 21, 20, 17, 23, 18, 16,
     9, 14, 12, 15, 13,  8, 11, 10,
     5,  7,  6,  4,  3,  0,  1,  2
];

pub trait GeneralId {
    type IdType;
    fn from_decoded<T: Into<Self::IdType>>(val: T) -> Self;
    fn from_encoded<T: Into<Self::IdType>>(val: T) -> Self;
    fn decode(&self) -> Self::IdType;
    fn encode(&self) -> Self::IdType;
    
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Copy)]
pub struct Id(u32);
impl GeneralId for Id {
    type IdType = u32;
    fn from_decoded<T: Into<u32>>(val: T) -> Self {
        Self(val.into())
    }
    fn from_encoded<T: Into<u32>>(val: T) -> Self {
        let val: u32 = val.into();
        let mut output = 0u32;
        for i in 0..32 {
            if val & (1 << USER_ID_BIT_MAP_REV[31-i]) != 0 {
                output |= 1 << i;
            }
        }
        Self(output)
    }
    fn decode(&self) -> u32 {
        self.0
    }
    fn encode(&self) -> u32 {
        let mut output = 0u32;
        let input = self.0;
        for i in 0..32 {
            if input & (1 << USER_ID_BIT_MAP[31-i]) != 0 {
                output |= 1 << i;
            }
        }
        output
    }
}
impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Into<i32> for Id {
    fn into(self) -> i32 {
        self.decode() as i32
    }
}

pub type UserId = Id;
pub type RoomId = Id;
pub type LoneId = Id;
pub type RoleId = Id;


#[test]
fn gen_rev() -> () {
    let iter = USER_ID_BIT_MAP.iter().rev().enumerate();
    let mut res = [0u8; 32];
    for (i, v) in iter {
        res[*v as usize] = i as u8;
    }
    println!("{:?}", res.into_iter().rev().collect::<Vec<u8>>())
}


#[test]
fn uuid_test() -> () {
    let uid1 = UserId::from_decoded(114514u32);
    let uid2 = UserId::from_encoded(uid1.encode());
    assert_eq!(114514, uid2.decode())
}

#[test]
fn uuid_test2() -> () {
    for i in 0..10000u32 {
        let uid = UserId::from_decoded(i);
        println!("ID: {} ---- UID: {}", uid.decode(), uid.encode())
    }
}
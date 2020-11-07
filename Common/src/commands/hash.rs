use std::cmp::min;
use std::fmt::{Display, Formatter};
use std::fmt;

use serde::{Deserialize, Serialize};

pub type UserPrivateKey = [u8; 32];
pub type UserPublicKey = u32;
pub type RoomId = u64;


use std::io::Cursor;
use std::rc::Rc;

use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};

#[cfg(test)]
pub mod udp;

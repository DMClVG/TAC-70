use std::{borrow::BorrowMut, cell::RefCell};

use bytes::*;

pub struct TAC70 {
    pub mem: [u8; 0x18000],
    pub code: String,
}

impl TAC70 {
    pub fn new(mem: &[u8]) -> Self {
        Self {
            mem: mem.try_into().unwrap(),
            code: String::new(),
        }
    }
}
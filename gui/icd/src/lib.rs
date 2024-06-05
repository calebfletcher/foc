#![no_std]

use postcard::experimental::schema::Schema;
use postcard_rpc::endpoint;
use serde::{Deserialize, Serialize};

endpoint!(PingEndpoint, u32, u32, "ping");

endpoint!(WriteValueEndpoint, u32, (), "value/write");
endpoint!(ReadValueEndpoint, (), State, "value/read");

#[derive(Debug, PartialEq, Serialize, Deserialize, Schema)]
pub struct State {
    pub value: u32,
    pub times_written: u32,
}

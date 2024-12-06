#![no_std]

use serde::{Deserialize, Serialize};

pub trait Endpoint {
    const ID: u8;
    type Request: Serialize + for<'a> Deserialize<'a>;
    type Response: Serialize + for<'a> Deserialize<'a>;
}

#[allow(unused)]
struct IdCheck;
#[allow(unused)]
trait DuplicateEndpointDetected<const ID: u8> {}

macro_rules! endpoint {
    ($id: literal, $name: ident, $req: ty, $resp: ty) => {
        pub struct $name;
        impl Endpoint for $name {
            const ID: u8 = $id;
            type Request = $req;
            type Response = $resp;
        }
        impl DuplicateEndpointDetected<$id> for IdCheck {}
    };
}

#[macro_export]
macro_rules! generate_endpoint_handler {
    ($frame: ident, $tx_buffer: ident, $(($name: path, $handler: expr))+) => {
        match $frame[0] {
            $(
                <$name>::ID => {
                    let resp = $handler(postcard::from_bytes(&$frame[1..]).unwrap());
                    let encoded_msg = postcard::to_slice_cobs(
                        &Frame {
                            endpoint: $frame[0],
                            msg: resp,
                        },
                        $tx_buffer,
                    )
                    .unwrap();
                    Ok(encoded_msg.len())
                }
            )*
            _ => Err(()),
        }
    };
}

endpoint!(0, PingEndpoint, (), ());
endpoint!(1, ReadEndpoint, (), u32);
endpoint!(2, WriteEndpoint, u32, ());
endpoint!(3, SinCosEndpoint, f32, (f32, f32));
endpoint!(4, EncoderAngle, (), u16);

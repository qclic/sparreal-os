#![no_std]
#![no_main]

use alloc::string::ToString;
use log::info;
use sparreal_rt::task::{spawn_with_config, TaskConfig};

extern crate alloc;
extern crate sparreal_rt;

#[sparreal_rt::entry]
fn main() {
    info!("hello world");
    let s = "hello world".to_string();
    let st = s.as_str();
    // unsafe {
    // let a = *(0xffff_ffff_ffff_ffff as *const u8);
    // sparreal_rt::println!("{:x}", a);
    // }

    assert_eq!(st, "hello world");

    spawn_with_config(
        || {
            info!("hello task");
        },
        TaskConfig {
            name: "hello".into(),
            priority: 0,
            stack_size: 0x1000,
        },
    );

    info!("2222");
}

use anyhow::Result;

use super::{Device, DeviceList};
use once_cell::sync::Lazy;

static DEVICE_LIST: Lazy<DeviceList> = Lazy::new(|| {
    let contents = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/regexes/device/mobiles.yml"));
    DeviceList::from_file(contents).expect("loading mobiles.yml")
});

pub fn lookup(ua: &str) -> Result<Option<Device>> {
    DEVICE_LIST.lookup(ua, "mobile")
}

use page_table_generic::{AccessSetting, CacheSetting};

use crate::platform_if::PlatformImpl;

use super::addr2::PhysRange;

pub fn bss() -> &'static [u8] {
    PlatformImpl::kernel_regions().bss.as_slice()
}

pub fn text() -> &'static [u8] {
    PlatformImpl::kernel_regions().text.as_slice()
}
use core::panic::PanicInfo;

use log::error;

use crate::platform::PlatformImpl;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{info}");

    unsafe {
        PlatformImpl::shutdown();
    };
    unreachable!()
}

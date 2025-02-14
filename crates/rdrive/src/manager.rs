use super::device;

#[derive(Default)]
pub struct Manager {
    pub intc: device::intc::Container,
}

impl Manager {
    pub const fn new() -> Self {
        Self {
            intc: device::Container::new(),
        }
    }
           
}

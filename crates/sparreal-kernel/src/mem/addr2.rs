use core::{fmt::Debug, ops::Range};

macro_rules! def_addr {
    ($name:ident, $t:ty) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, PartialOrd)]
        pub struct $name<T>($t, core::marker::PhantomData<T>);

        impl<T> $name<T> {
            pub const fn new(val: $t) -> Self {
                Self(val, core::marker::PhantomData)
            }

            pub fn raw(self) -> $t {
                self.0
            }
        }
        impl<T> core::fmt::Debug for $name<T> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:#x}", self.0)
            }
        }
        impl<T> core::fmt::Display for $name<T> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:#x}", self.0)
            }
        }
        impl<T> From<$t> for $name<T> {
            fn from(value: $t) -> Self {
                Self(value, core::marker::PhantomData)
            }
        }
        impl<T> From<$name<T>> for $t {
            fn from(value: $name<T>) -> Self {
                value.0
            }
        }

        impl<T> core::ops::Add<usize> for $name<T> {
            type Output = Self;

            fn add(self, rhs: usize) -> Self::Output {
                Self(self.0 as usize + rhs, core::marker::PhantomData)
            }
        }

        impl<T> core::ops::Sub<usize> for $name<T> {
            type Output = Self;

            fn sub(self, rhs: usize) -> Self::Output {
                Self(self.0 as usize - rhs, core::marker::PhantomData)
            }
        }
    };
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CRange<T: Debug + Sized + Clone + Copy> {
    pub start: T,
    pub end: T,
}

impl<T: Debug + Sized + Clone + Copy> From<Range<T>> for CRange<T> {
    fn from(value: Range<T>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl<T: Debug + Sized + Clone + Copy> Debug for CRange<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[{:?}, {:?})", self.start, self.end)
    }
}

def_addr!(Virt, usize);
def_addr!(Phys, usize);

pub type VirtAddr = Virt<u8>;
pub type PhysAddr = Phys<u8>;
pub type VirtRange = CRange<VirtAddr>;
pub type PhysRange = CRange<PhysAddr>;

impl<T> From<Virt<T>> for *mut T {
    fn from(value: Virt<T>) -> *mut T {
        value.0 as _
    }
}

#[macro_export]
macro_rules! pa {
    (val: $val:expr) => {
        Phys::new($val as _)
    };
}

#[macro_export]
macro_rules! va {
    (val: $val:expr) => {
        Virt::new($val as _)
    };
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_addr() {
        let a = VirtAddr::new(0x1000);
        let b = VirtAddr::new(0x2000);
        assert_eq!(a + 0x1000, b);
        assert_eq!(b - 0x1000, a);

        let c = a..b;
    }
}

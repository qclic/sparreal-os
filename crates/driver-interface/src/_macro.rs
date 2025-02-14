#[macro_export]
macro_rules! custom_type {
    ($name:ident, $target:ty, $debug: expr) => {
        #[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        #[repr(transparent)]
        pub struct $name($target);

        impl From<$target> for $name {
            fn from(value: $target) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $target {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, $debug, self.0)
            }
        }
    };

    ($name:ident, $target:ty) => {
        $crate::custom_type!($name, $target, "{:?}")
    };
}

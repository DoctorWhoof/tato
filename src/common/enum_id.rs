
pub trait EnumID: Clone + Copy + PartialEq + Into<u8> + Into<usize> + From<u8> {
    fn count() -> usize;
}

#[macro_export]
macro_rules! enum_id {
    ($name:ident { $($variants:ident),* $(,)? }) => {
        // Define the enum with the provided name and variants
        #[derive(Clone, Copy, Debug, PartialEq)]#[repr(u8)]
        pub enum $name {
            $($variants),*
        }

        #[allow(clippy::from_over_into)]
        impl Into<u8> for $name { fn into(self) -> u8 { self as u8 } }


        impl Into<usize> for $name { fn into(self) -> usize { self as usize } }


        impl From<u8> for $name {
            fn from(value: u8) -> Self {
                if value < $name::count() as u8 { return unsafe { core::mem::transmute(value) } } 
                panic!("Invalid source value for Group enum")
            }
        }


        impl EnumID for $name {
            fn count() -> usize {
                let variants = [$(stringify!($variants)),*];
                variants.len()
            }
        }
    };
}



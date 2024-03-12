
pub trait Countable { fn count() -> usize; }

#[doc(hidden)]
pub trait EnumID: Clone + Copy + PartialEq + Into<u8> + Into<usize> + From<u8> + Countable {}

pub trait TilesetEnum: EnumID {}

pub trait PaletteEnum: EnumID {}

pub trait GroupEnum: EnumID {}


#[doc(hidden)]
#[macro_export]
macro_rules! implement_enum_id {
    ($name:ident { $($variants:ident),* $(,)? }) => {
        // Define the enum with the provided name and variants
        #[derive(Clone, Copy, Debug, PartialEq)
        ]#[repr(u8)]
        pub enum $name {
            $($variants),*
        }

        impl Into<u8> for $name { fn into(self) -> u8 { self as u8 } }


        impl Into<usize> for $name { fn into(self) -> usize { self as usize } }


        impl From<u8> for $name {
            fn from(value: u8) -> Self {
                if value < $name::count() as u8 { return unsafe { core::mem::transmute(value) } } 
                panic!("Invalid source value for Group enum")
            }
        }


        impl Countable for $name {
            fn count() -> usize {
                let variants = [$(stringify!($variants)),*];
                variants.len()
            }
        }
    };
}


#[macro_export]
macro_rules! tileset_enum {
    ($name:ident { $($variants:ident),* $(,)? }) => {
        implement_enum_id! {
            $name {
                $($variants),*
            }
        }
        impl EnumID for $name {}
        impl TilesetEnum for $name {}
    };
}

#[macro_export]
macro_rules! palette_enum {
    ($name:ident { $($variants:ident),* $(,)? }) => {
        implement_enum_id! {
            $name {
                $($variants),*
            }
        }
        impl EnumID for $name {}
        impl PaletteEnum for $name {}
    };
}

#[macro_export]
macro_rules! group_enum {
    ($name:ident { $($variants:ident),* $(,)? }) => {
        implement_enum_id! {
            $name {
                $($variants),*
            }
        }
        impl EnumID for $name {}
        impl GroupEnum for $name {}
    };
}
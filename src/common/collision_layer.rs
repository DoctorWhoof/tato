use super::Countable;
use core::fmt::Debug;

pub trait CollisionLayer: Debug + Countable + Clone + Copy + Into<u16> {
    fn pow2(self) -> u16;
}


#[macro_export]
macro_rules! collision_layer_enum {
    ($name:ident { $($variants:ident),* $(,)? }) => {
        #[derive(Clone, Copy, PartialEq, Debug)]
        #[repr(u16)]
        
        pub enum $name {
            $($variants),*
        }

        impl Into<u16> for $name { fn into(self) -> u16 { self as u16 } }


        impl CollisionLayer for $name {
            fn pow2(self) -> u16 { 2u16.pow(self as u32) }
        }
        
        impl Countable for $name {
            fn count() -> usize {
                let variants = [$(stringify!($variants)),*];
                variants.len()
            }
        }
        
    };
}

#[macro_export]
macro_rules! implement_enum_index {
    ($enum_name:ident) => {
        impl From<u8> for $enum_name {
            fn from(value: u8) -> Self {
                // TODO: This causes error in a macro, although it works fine otherwise. Once this feature is stable,
                // perform a check before transmute
                // let count = u8::try_from(core::mem::variant_count::<Self>()).unwrap();
                // if value < count as u8 { return unsafe { core::mem::transmute(value) } } 
                // panic!("Invalid source value for Enum. ")
                unsafe { return core::mem::transmute(value) }
            }
        }

        impl Into<u8> for $enum_name {
            fn into(self) -> u8 {
                self as u8
            }
        }
        
        impl Into<usize> for $enum_name {
            fn into(self) -> usize {
                self as usize
            }
        }
    }
}


// #[macro_export]
// macro_rules! enum_id {
//     ($enum_name:ident { $($variant:ident),+ $(,)? }) => {
//         #[repr(u8)] #[derive(Clone, Copy, Debug, PartialEq)]
//         pub enum $enum_name {
//             $($variant),+
//         }

//         #[allow(clippy::from_over_into)]
//         impl Into<u8> for $enum_name { fn into(self) -> u8 { self as u8 } }

//         impl From<u8> for $enum_name {
//             fn from(value: u8) -> Self {
//                 if value < $enum_name::Count as u8 { return unsafe { core::mem::transmute(value) } } 
//                 panic!("Invalid source value for Group enum")
//             }
//         }
//     };
// }
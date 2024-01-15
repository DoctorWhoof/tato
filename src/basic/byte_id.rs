
pub trait ByteID: Copy + Clone + PartialEq {

    fn to_u8(self) -> u8;

    fn to_usize(self) -> usize;

    fn from_byte(value:u8) -> Self;

    fn len() -> usize;
}


#[macro_export]
macro_rules! implement_byte_id {
    ($enum_name:ident) => {
        impl ByteID for $enum_name {
        
            fn to_u8(self) -> u8 {
                self as u8
            }
    
            fn to_usize(self) -> usize {
                self as usize
            }
        
            fn from_byte(value:u8) -> Self {
                if value < $enum_name::Count as u8 { return unsafe { core::mem::transmute(value) } } 
                panic!("Invalid source value for Enum. ")
            }
            
            fn len() -> usize {
                $enum_name::Count as usize
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
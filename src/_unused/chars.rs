use core::array;

#[derive(Debug)]
pub struct Chars<const CAP:usize> {
    data:[u8; CAP],
    len:usize
}

impl<const CAP:usize> Default
for Chars<CAP> {
    fn default() -> Self {
        Self {
            data: array::from_fn(|_| 0 ),
            len: 0
        }
    }
}

// // Technically TryFrom is more correct, since the conversion may fail. However, it is more annoying.
// impl<const CAP:usize> TryFrom<&str>
// for Chars<CAP> {
//     type Error = &'static str;

//     fn try_from(text: &str) -> Result<Self, Self::Error> {
//         if !text.is_ascii() { return Err("") }
//         if text.len() > CAP { return Err("") }
//         let mut result = Self::default();
//         result.unchecked_set(text);
//         Ok(result)
//     }
// }


impl<const CAP:usize> From<&str>
for Chars<CAP> {
    fn from(text:&str) -> Self {
        let mut result = Self::default();
        result.set(text);
        result
    }
}


impl<const CAP:usize> Chars<CAP> {

    // pub fn empty(&self) -> bool {
    //     self.len == 0
    // }

    fn unchecked_set(&mut self, text:&str) {
        self.len = usize::min(text.len(), CAP);
        self.data[..self.len].copy_from_slice(&text.as_bytes()[..self.len])
    }

    pub fn set(&mut self, text:&str) {
        if !text.is_ascii() { panic!("Chars: Error, attempt to convert from non ASCII text") }
        if text.len() > CAP { panic!("Chars: Error, text length is longer than capacity {}", CAP) }
        self.unchecked_set(text);
    }


    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.data[..self.len]).unwrap()
    }

}


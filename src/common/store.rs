
// pub struct InitError {}

pub const fn find_tag(name:&str, bytes:&[u8]) -> usize  {
    let name = name.as_bytes();
    let mut index = 0;
    let mut name_index = 0;
    let mut current_match = name[0];
    while index < bytes.len() {
        if bytes[index] == current_match {
            name_index += 1;
            if name_index == name.len() {
                return index - (name.len() - 1);
            }
            current_match = name[name_index];
        } else {
            name_index = 0;
            current_match = name[0];
        };
        index += 1
    }
    panic!("Tag not found!")
}


// pub struct Store<const LEN:usize> {
//     pub data:[u8; LEN]
// }

// impl<const LEN:usize> Store<LEN> {
    
//     pub const fn find_index(&self, search_value:u8) -> Result<usize,InitError>  {
//         let mut index = 0;
//         while index < self.data.len() {
//             if self.data[index] == search_value {
//                 return Ok(index)
//             };
//             index += 1
//         }
//         Err(InitError{})
//     }

// }
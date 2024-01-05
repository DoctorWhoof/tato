#[derive(Debug, Clone, Default)]
pub struct KeyChain {
    head:u8,
}


impl KeyChain {

    pub fn new() -> Self {
        KeyChain { head:0 }
    }
    
    pub fn get(&mut self) -> u8 {
        let result = self.head;
        self.head += 1;
        result
    }

    pub fn len(&mut self) -> u8 {
        self.head
    }

}
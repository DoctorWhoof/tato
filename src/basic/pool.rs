
pub struct Pool<T, const CAP:usize> {
    pub data:[Option<T>; CAP],
    head:usize
}

impl<T, const CAP:usize> Default
for Pool<T, CAP>
where T:Default {
    fn default() -> Self {
        Self {
            data: core::array::from_fn( |_| None ),
            head: 0
        }
    }
}

impl<T, const CAP:usize> Pool<T, CAP> {

    pub fn is_empty(&self) -> bool {
        self.head == 0
    }


    pub fn set(&mut self, i:usize, value:T) {
        if i >= self.head { panic!("Pool error: Invalid index {}", i) }
        self.data[i] = Some(value);
    }


    pub fn push(&mut self, elem:T){
        if self.head == CAP { panic!("Pool Error: Capacity of {} exceeded", CAP) }
        self.data[self.head] = Some(elem);
        self.head += 1;
    }


    pub fn get(&self, i:usize) -> Option<&T> {
        if i >= self.head { return None }
        self.data[i].as_ref()
    }


    pub fn get_mut(&mut self, i:usize) -> Option<&mut T> {
        if i >= self.head { return None }
        self.data[i].as_mut()
    }


    /// Expands pool to include index! Not sure about this.
    pub fn insert(&mut self, i:usize, elem:T) {
        if i >= CAP { panic!("Pool Error: Capacity of {} exceeded", CAP) }
        if i >= self.head { self.head = i }
        self.data[i] = Some(elem);
        self.head += 1;
    }


    pub fn len(&self) -> usize {
        self.head
    }


    pub fn capacity(&self) -> usize {
        CAP
    }

}

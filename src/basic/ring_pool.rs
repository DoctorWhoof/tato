pub struct RingPool<T, const CAP:usize>
where T:Default + Clone {
    data:[T; CAP],
    head:usize,
    tail:usize,
    spawn_head:usize,
}

impl<T, const CAP:usize> Default for RingPool<T, CAP>
where T:Default + Clone {
    fn default() -> Self {
        Self {
            data: core::array::from_fn( |_| Default::default() ),
            head: 0,
            tail: 0,
            spawn_head: 0,
        }
    }
}

impl<T, const CAP:usize> RingPool<T, CAP>
where T:Default + Clone {


    pub fn new() -> Self {
        Default::default()
    }


    pub fn len(&self) -> usize {
        self.head - self.tail
    }


    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }


    pub fn clear(&mut self) {
        self.tail = 0;
        self.head = 0;
        self.spawn_head = 0;
    }


    // Simply returns the element at the spawn head, does not modify the other counters.
    pub fn spawn(&mut self) -> Option<&T> {
        let mut result = None;
        if !self.is_empty() {
            result = Some(&self.data[self.spawn_head % CAP]);
            self.spawn_head += 1;
        }
        result
    }


    /// Pushes a new item to the head of the buffer, returning the old one that was "pushed out" if this is a ring-buffer.
    pub fn push(&mut self, elem:T) -> Option<T> {
        let mut result = None;
        if self.head - self.tail == CAP {
            result = Some(self.data[self.tail % CAP].clone());
            self.data[self.head % CAP] = elem;
            self.tail += 1;
            self.head += 1;
        } else {
            self.data[self.head % CAP] = elem;
            self.head += 1;
        }
        result
    }

    /// "Pops" the tail end of the buffer, reducing the length and returning the popped item.
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty(){ return None }
        let result = Some(self.data[self.tail % CAP].clone());
        self.tail += 1;
        result
    }


    pub fn get(&self, i:usize) -> Option<&T> {
        if i >= self.len() { return None }
        Some(&self.data[i])
    }


    pub fn get_mut(&mut self, i:usize) -> Option<&mut T> {
        if i >= self.len() { return None }
        Some(&mut self.data[i])
    }


    pub fn capacity(&self) -> usize {
        CAP
    }

    pub fn iter(&self) -> core::slice::Iter<T> {
        self.data[..self.len()].iter()
    }


    pub fn iter_mut(&mut self) -> core::slice::IterMut<T> {
        let len = self.len();
        self.data[..len].iter_mut()
    }


    /// Iterates through all items, only keeping the ones where "func" returns true
    pub fn retain(&mut self, mut func:impl FnMut(&T) -> bool){
        if self.is_empty() { return };
        let mut i = self.tail;
        while i < self.head {
            // Remove item if false
            if !func(&self.data[i % CAP]) {                 
                if self.len() > 1 {
                    // Promote item at tail (already updated with "func") to current index
                    self.data[i % CAP] = self.data[self.tail % CAP].clone();
                } 
                self.tail += 1;
            }
            i += 1;
        }
    }

}



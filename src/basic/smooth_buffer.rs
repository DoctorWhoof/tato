//! Simple ringbuffer with averaging and smoothing.

pub struct SmoothBuffer<const CAP:usize> {
    data: [f32; CAP],
    head: usize,
    sum: Option<f32>,
    max: Option<f32>,
    min: Option<f32>,
    filled_len: f32,
}


impl<const CAP:usize> Default for SmoothBuffer<CAP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAP:usize>  SmoothBuffer<CAP>{

    pub fn new() -> Self {
        SmoothBuffer{
            data: [0.0; CAP],
            head:0,
            sum: None,
            max: None,
            min: None,
            filled_len:0.0,
        }
    }

    // Fast! "Sum" is always kept up to date on push. No need iterate.
    pub fn average(&self) -> f32 {
        if self.filled_len > 0.0 {
            return self.sum.unwrap_or(0.0) / self.filled_len
        }
        0.0
    }


    #[allow(unused)]
    pub fn clear(&mut self) {
        for n in 0..self.data.len(){
            self.data[n] = 0.0;
        }
        self.sum = None;
        self.max = None;
        self.min = None;
        self.filled_len = 0.0;
        self.head = 0;
    }


    pub fn is_empty(&self) -> bool { self.filled_len == 0.0 }


    pub fn max(&self) -> f32 { self.max.unwrap_or(0.0) }
    
    
    pub fn min(&self) -> f32 { self.min.unwrap_or(0.0) }


    pub fn len(&self) -> usize { self.data.len() }


    pub fn push(&mut self, value:f32) {
        match self.max {
            None => self.max = Some(value),
            Some( max ) => {
                self.max = Some(f32::max(max, value))
            }
        }
        match self.min {
            None => self.min = Some(value),
            Some( min ) => {
                self.min = Some(f32::min(min, value))
            }
        }
        match self.sum {
            None => self.sum = Some(value),
            Some( sum ) => {
                self.sum = Some(sum - self.data[self.head] + value)
            }
        }
        
        // Push data into storage
        self.data[self.head] = value;
        self.head += 1;
        if self.head >= self.data.len() { self.head = 0 }
        if (self.filled_len as usize) < self.data.len() { self.filled_len += 1.0; }
    }


    // The offset is in relation to the current head index ("0" means "at the current head")
    // pub fn get(&self, offset:usize) -> f32 {
    //     self.data[(offset + self.head) % self.data.len()]
    // }


    // May be slow for long buffers that fall outside the special cases.
    // pub fn smooth(&self) -> f32 {
    //     let len = self.len();
    //     match len {
    //         1 => {
    //             self.data[0]
    //         }
    //         2 => {
    //             (self.data[0] + self.data[1]) * 0.5
    //         }
    //         3 => {
    //             let p0 = self.data[self.head]*0.025;
    //             let p1 = self.data[(self.head+1) % len]*0.95;
    //             let p2 = self.data[(self.head+2) % len]*0.025;
    //             p0+p1+p2
    //         }
    //         4 => {
    //             let p0 = self.data[self.head]*0.016667;
    //             let p1 = self.data[(self.head+1) % len]*0.483333;
    //             let p2 = self.data[(self.head+2) % len]*0.483333;
    //             let p3 = self.data[(self.head+3) % len]*0.016667;
    //             p0+p1+p2+p3
    //         }
    //         5 => {
    //             let p0 = self.data[self.head]*0.01667;
    //             let p1 = self.data[(self.head+1) % len]*0.0333;
    //             let p2 = self.data[(self.head+2) % len]*0.9;
    //             let p3 = self.data[(self.head+3) % len]*0.0333;
    //             let p4 = self.data[(self.head+4) % len]*0.01667;
    //             p0+p1+p2+p3+p4
    //         }
    //         _ => { // General case, any other number of items. Can be slow!
    //             use std::f32::consts::TAU;
    //             let mut acc = 0.0;
    //             for i in 0..self.len(){
    //                 let x = i as f32 / self.filled_len;
    //                 let y = (((x-0.25)*TAU).sin()/2.0)+0.5;
    //                 acc += self.get(i) * y;
    //             }
    //             acc / len as f32 * 2.222
    //         }
    //     }
    // }

}



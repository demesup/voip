use std::collections::VecDeque;

pub struct JitterBuffer {
    pub buffer: VecDeque<i16>,
    min_delay: usize, 
}

impl JitterBuffer {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
            min_delay: 4800, 
        }
    }

    pub fn push_packet(&mut self, samples: &[i16]) {
        for &s in samples {
            self.buffer.push_back(s);
        }
       
        while self.buffer.len() > 19200 {
            self.buffer.pop_front();
        }
    }

    pub fn pop_sample(&mut self) -> i16 {
       
        if self.buffer.len() < self.min_delay {
            return 0; 
        }
        self.buffer.pop_front().unwrap_or(0)
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn set_min_delay_ms(&mut self, delay_ms: u32) {
       
        self.min_delay = delay_ms as usize * 48;
    }
}

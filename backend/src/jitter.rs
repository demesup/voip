use std::collections::VecDeque;

pub struct JitterBuffer {
    pub buffer: VecDeque<i16>,
    min_delay: usize,  // Minimum delay in samples to absorb jitter
}

impl JitterBuffer {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
            min_delay: 4800,  // 100ms delay at 48kHz
        }
    }

    pub fn push_packet(&mut self, samples: &[i16]) {
        for &s in samples {
            self.buffer.push_back(s);
        }
        // Limit buffer to prevent excessive delay (max 400ms at 48kHz)
        while self.buffer.len() > 19200 {
            self.buffer.pop_front();
        }
    }

    pub fn pop_sample(&mut self) -> i16 {
        // If buffer is below minimum delay, insert silence to build up delay
        if self.buffer.len() < self.min_delay {
            return 0;  // Return silence to build delay
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
        // Assuming 48kHz sample rate
        self.min_delay = delay_ms as usize * 48;
    }
}

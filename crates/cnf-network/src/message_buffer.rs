use std::collections::VecDeque;
use crate::transport::CnfMessage;
use crate::vector_clock::NodeId;
use crate::error::CnfNetworkError;


#[allow(dead_code)]
const DEFAULT_CAPACITY: usize = 1_024;


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority { Low = 0, Normal = 1, High = 2, Critical = 3 }


pub struct BufferedMessage {
    pub target: NodeId,
    pub message: CnfMessage,
    pub priority: Priority,
    pub retry_count: u8,
}


pub struct MessageBuffer {
    queue: VecDeque<BufferedMessage>,
    capacity: usize,
    dropped: u64,   // metrik: pesan yang di-drop karena penuh
}


impl MessageBuffer {
    pub fn new(capacity: usize) -> Self {
        Self { queue: VecDeque::with_capacity(capacity), capacity, dropped: 0 }
    }


    /// Push pesan — High/Critical selalu masuk, Low di-drop jika penuh
    pub fn push(&mut self, msg: BufferedMessage) -> Result<(), CnfNetworkError> {
        if self.queue.len() >= self.capacity {
            if msg.priority >= Priority::High {
                // Drop pesan Low priority tertua untuk beri ruang
                if let Some(pos) = self.queue.iter().position(|m| m.priority == Priority::Low) {
                    self.queue.remove(pos);
                    self.dropped += 1;
                } else {
                    return Err(CnfNetworkError::BufferFull);
                }
            } else {
                self.dropped += 1;
                return Err(CnfNetworkError::BufferFull);
            }
        }
        // Insert sesuai priority (tinggi di depan)
        let pos = self.queue.iter().position(|m| m.priority < msg.priority).unwrap_or(self.queue.len());
        self.queue.insert(pos, msg);
        Ok(())
    }


    pub fn pop(&mut self) -> Option<BufferedMessage> { self.queue.pop_front() }
    pub fn len(&self) -> usize { self.queue.len() }
    pub fn is_empty(&self) -> bool { self.queue.is_empty() }
    pub fn dropped_count(&self) -> u64 { self.dropped }
    pub fn backpressure_ratio(&self) -> f64 { self.queue.len() as f64 / self.capacity as f64 }
}

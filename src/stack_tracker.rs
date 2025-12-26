use std::collections::VecDeque;

pub struct StackTracker {
    writes: VecDeque<StackWrite>,
    pub(crate) max_events: usize,
}

#[derive(Copy, Clone, Debug)]
pub enum StackWriteKind {
    Call,
    Push,
    Interrupt,
    Manual,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct StackWrite {
    pub addr: u16,
    pub kind: StackWriteKind,
    pub pc: u16,
}

impl StackTracker {
    pub fn new(max_events: usize) -> Self {
        Self {
            writes: VecDeque::new(),
            max_events,
        }
    }
    pub fn clear(&mut self) {
        self.writes.clear();
    }
    pub fn record(&mut self, addr: u16, kind: StackWriteKind, pc: u16) {
        if self.writes.len() >= self.max_events {
            self.writes.pop_front();
        }

        self.writes.push_back(StackWrite { addr, kind, pc });
    }

    pub fn last_write_to(&self, addr: u16) -> Option<StackWriteKind> {
        self.writes
            .iter()
            .rev()
            .find(|w| w.addr == addr)
            .map(|w| w.kind)
    }
}




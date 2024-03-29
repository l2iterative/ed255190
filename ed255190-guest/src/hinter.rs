use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecompressionHint {
    pub x: [u32; 8],
}

pub trait ComputeHintStreamer {
    fn next(&mut self) -> [u32; 8];
}

pub struct ComputeHintStore<'a> {
    store: &'a [u32],
    index: usize,
}

impl<'a> ComputeHintStore<'a> {
    pub fn new(store: &'a [u32]) -> Self {
        Self { store, index: 0 }
    }
}

impl<'a> ComputeHintStreamer for ComputeHintStore<'a> {
    fn next(&mut self) -> [u32; 8] {
        let res = <&[u32; 8]>::try_from(&self.store[self.index * 8..self.index * 8 + 8]).unwrap();
        self.index += 1;
        *res
    }
}

#[cfg(target_os = "zkvm")]
#[repr(align(1024))]
pub struct ComputeHintBuffer {
    buf: [u32; 256],
    offset: usize,
    max: usize,
}

#[cfg(target_os = "zkvm")]
impl ComputeHintBuffer {
    pub fn new(max: usize) -> Self {
        Self {
            buf: [0u32; 256],
            offset: 32,
            max,
        }
    }
}

#[cfg(target_os = "zkvm")]
impl ComputeHintStreamer for ComputeHintBuffer {
    #[inline]
    fn next(&mut self) -> [u32; 8] {
        use risc0_zkvm::guest::env;
        if self.offset == 32 {
            if self.max >= 256 {
                env::read_slice(&mut self.buf);
                self.max -= 256;
            } else {
                env::read_slice(&mut self.buf[0..self.max]);
                self.max = 0;
            }
            self.offset = 0;
        }
        let mut res = [0u32; 8];
        for i in 0..8 {
            res[i] = self.buf[i + self.offset * 8];
        }
        self.offset += 1;
        res
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Hint {
    FormatError,
    VerificationFailed,
    Ok,
}

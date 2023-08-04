use super::capabilities::ScaleCapabilities;
use super::reading::Reading;

pub trait ScaleDriver {
    fn capabilities(&self) -> ScaleCapabilities;
}

pub trait ScaleChunkDecoder {
    fn push_chunk(&mut self, chunk: &str) -> Vec<Reading>;
}

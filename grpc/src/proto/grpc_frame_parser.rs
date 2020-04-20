use crate::proto::grpc_frame::parse_grpc_frame_header;
use crate::result;
use bytes::Buf;
use bytes::Bytes;
use httpbis::BufGetBytes;
use httpbis::BytesDeque;

#[derive(Default)]
pub(crate) struct GrpcFrameParser {
    /// Next frame length; stripped prefix from the queue
    next_frame_len: Option<u32>,
    /// Unparsed bytes
    buffer: BytesDeque,
}

impl GrpcFrameParser {
    /// Add `Bytes` object to the queue.
    pub fn enqueue(&mut self, bytes: Bytes) {
        self.buffer.extend(bytes);
    }

    /// Try populating `next_frame_len` field from the queue.
    fn fill_next_frame_len(&mut self) -> result::Result<Option<u32>> {
        if let None = self.next_frame_len {
            self.next_frame_len = parse_grpc_frame_header(&mut self.buffer)?;
        }
        Ok(self.next_frame_len)
    }

    /// Parse next frame if buffer has full frame.
    pub fn next_frame(&mut self) -> result::Result<Option<Bytes>> {
        if let Some(len) = self.fill_next_frame_len()? {
            if self.buffer.remaining() > len as usize {
                self.next_frame_len = None;
                return Ok(Some(BufGetBytes::get_bytes(&mut self.buffer, len as usize)));
            }
        }
        Ok(None)
    }

    /// Parse all frames from buffer.
    pub fn next_frames(&mut self) -> result::Result<Vec<Bytes>> {
        let mut r = Vec::new();
        while let Some(frame) = self.next_frame()? {
            r.push(frame);
        }
        Ok(r)
    }

    /// Buffered data is empty.
    pub fn is_empty(&self) -> bool {
        self.next_frame_len.is_none() && !self.buffer.has_remaining()
    }
}

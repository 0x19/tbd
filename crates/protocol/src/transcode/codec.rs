//! A tonic codec over dynamic messages. `ProstCodec` needs `Default` to
//! decode, and a [`DynamicMessage`] needs its descriptor instead, so the
//! decoder carries the output descriptor and the encoder relies on
//! `prost::Message` alone.

use prost::Message as _;
use prost_reflect::{DynamicMessage, MessageDescriptor};
use tonic::{
    Status,
    codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder},
};

/// Encodes request messages and decodes response messages of one RPC.
#[derive(Debug, Clone)]
pub struct DynamicCodec {
    output: MessageDescriptor,
}

impl DynamicCodec {
    /// A codec whose responses are `output`.
    #[must_use]
    pub fn new(output: MessageDescriptor) -> Self {
        Self { output }
    }
}

impl Codec for DynamicCodec {
    type Encode = DynamicMessage;
    type Decode = DynamicMessage;
    type Encoder = DynamicEncoder;
    type Decoder = DynamicDecoder;

    fn encoder(&mut self) -> Self::Encoder {
        DynamicEncoder
    }

    fn decoder(&mut self) -> Self::Decoder {
        DynamicDecoder {
            desc: self.output.clone(),
        }
    }
}

/// Encodes a dynamic message into a gRPC frame.
#[derive(Debug, Clone, Copy)]
pub struct DynamicEncoder;

impl Encoder for DynamicEncoder {
    type Item = DynamicMessage;
    type Error = Status;

    fn encode(&mut self, item: Self::Item, dst: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        item.encode(dst)
            .map_err(|e| Status::internal(format!("encode: {e}")))
    }
}

/// Decodes one gRPC frame into a dynamic message of the response type.
#[derive(Debug, Clone)]
pub struct DynamicDecoder {
    desc: MessageDescriptor,
}

impl Decoder for DynamicDecoder {
    type Item = DynamicMessage;
    type Error = Status;

    fn decode(&mut self, src: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        DynamicMessage::decode(self.desc.clone(), src)
            .map(Some)
            .map_err(|e| Status::internal(format!("decode: {e}")))
    }
}

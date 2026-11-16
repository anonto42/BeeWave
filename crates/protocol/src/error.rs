//! Errors returned by the codec.

#[derive(Debug, PartialEq)]
pub enum Error {
    /// First byte was not `PROTO_VERSION`; carries what we saw.
    BadVersion(u8),
    /// Buffer shorter than the header.
    TooShort,
    /// postcard could not encode/decode the body.
    Codec,
}

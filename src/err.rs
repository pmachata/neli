//! This is the module that contains the error types used in `neli`
//!
//! There are three main types:
//! * `NlError` - typically socket errors
//! * `DeError` - Error while deserializing
//! * `SerError` - Error while serializing
//!
//! Additionally there is one other type: `Nlmsgerr`. This type is returned at the protocol level
//! by netlink sockets when an error has been returned in response to the given request.
//!
//! # Design decisions
//!
//! `NlError` can either be created with a custom `String` message or using three variants, one for
//! no ACK received, one for a bad PID that does not correspond to that assigned to the socket, or
//! one for a bad sequence number that does not correspond to the request sequence number.

use std::{
    self,
    error::Error,
    fmt::{self, Display},
    io,
    str,
    string,
};

use bytes::{Bytes, BytesMut};
use libc;

use crate::{
    consts::NlType,
    nl::{NlEmpty, Nlmsghdr},
    Nl,
};

macro_rules! try_err_compat {
    ($err_name:ident, $($from_err_name:path),*) => {
        $(
            impl From<$from_err_name> for $err_name {
                fn from(v: $from_err_name) -> Self {
                    $err_name::new(v)
                }
            }
        )*
    }
}

/// Struct representing netlink packets containing errors
#[derive(Debug)]
pub struct Nlmsgerr<T> {
    /// Error code
    pub error: libc::c_int,
    /// Packet header for request that failed
    pub nlmsg: Nlmsghdr<T, NlEmpty>,
}

impl<T> Nl for Nlmsgerr<T>
where
    T: NlType,
{
    fn serialize(&self, mem: BytesMut) -> Result<BytesMut, SerError> {
        Ok(serialize! {
            PAD self;
            mem;
            self.error, size;
            self.nlmsg, size
        })
    }

    fn deserialize(mem: Bytes) -> Result<Self, DeError> {
        Ok(deserialize! {
            STRIP Self;
            mem;
            Nlmsgerr {
                error: libc::c_int => deserialize_type_size!(libc::c_int => type_size),
                nlmsg: Nlmsghdr<T, NlEmpty> => mem.len() - libc::c_int::type_size()
                    .expect("Integers have static sizes")
            } => mem.len()
        })
    }

    fn size(&self) -> usize {
        self.error.size() + self.nlmsg.size()
    }

    fn type_size() -> Option<usize> {
        Nlmsghdr::<T, NlEmpty>::type_size()
            .and_then(|nhdr_sz| {
                libc::c_int::type_size().map(|cint| cint + nhdr_sz)
            })
    }
}

/// Netlink protocol error
#[derive(Debug)]
pub enum NlError {
    /// Type indicating a message from a converted error
    Msg(String),
    /// No ack was received when `NlmF::Ack` was specified in the request
    NoAck,
    /// The sequence number for the response did not match the request
    BadSeq,
    /// Incorrect PID socket identifier in received message
    BadPid,
}

try_err_compat!(NlError, io::Error, SerError, DeError);

impl NlError {
    /// Create new error from a data type implementing `Display`
    pub fn new<D>(s: D) -> Self
    where
        D: Display,
    {
        NlError::Msg(s.to_string())
    }
}

/// Netlink protocol error
impl Display for NlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match *self {
            NlError::Msg(ref msg) => msg,
            NlError::NoAck => "No ack received",
            NlError::BadSeq => "Sequence number does not match the request",
            NlError::BadPid => "PID does not match the socket",
        };
        write!(f, "{}", msg)
    }
}

impl Error for NlError {
    fn description(&self) -> &str {
        match *self {
            NlError::Msg(ref msg) => msg.as_str(),
            NlError::NoAck => "No ack received",
            NlError::BadSeq => "Sequence number does not match the request",
            NlError::BadPid => "PID does not match the socket",
        }
    }
}

/// Serialization error
#[derive(Debug)]
pub enum SerError {
    /// Abitrary error message 
    Msg(String, BytesMut),
    /// The end of the buffer was reached before serialization finished
    UnexpectedEOB(BytesMut),
    /// Serialization did not fill the buffer
    BufferNotFilled(BytesMut),
    /// Wrapper for an `io::Error`
    IOError(io::Error, BytesMut),
}

impl SerError {
    /// Create a new error with the given message as description
    pub fn new<D>(msg: D, bytes: BytesMut) -> Self where D: Display {
        SerError::Msg(msg.to_string(), bytes)
    }

    /// Reconstruct `BytesMut` at current level to bubble error up
    pub fn reconstruct(self, start: Option<BytesMut>, end: Option<BytesMut>) -> Self {
        match (start, end) {
            (Some(mut s), Some(e)) => {
                match self {
                    SerError::BufferNotFilled(b) => {
                        s.unsplit(b);
                        s.unsplit(e);
                        SerError::BufferNotFilled(s)
                    }
                    SerError::UnexpectedEOB(b) => {
                        s.unsplit(b);
                        s.unsplit(e);
                        SerError::UnexpectedEOB(s)
                    }
                    SerError::Msg(m, b) => {
                        s.unsplit(b);
                        s.unsplit(e);
                        SerError::Msg(m, s)
                    }
                    SerError::IOError(err, b) => {
                        s.unsplit(b);
                        s.unsplit(e);
                        SerError::IOError(err, s)
                    }
                }
            },
            (Some(mut s), _) => {
                match self {
                    SerError::BufferNotFilled(b) => {
                        s.unsplit(b);
                        SerError::BufferNotFilled(s)
                    }
                    SerError::UnexpectedEOB(b) => {
                        s.unsplit(b);
                        SerError::UnexpectedEOB(s)
                    }
                    SerError::Msg(m, b) => {
                        s.unsplit(b);
                        SerError::Msg(m, s)
                    }
                    SerError::IOError(err, b) => {
                        s.unsplit(b);
                        SerError::IOError(err, s)
                    }
                }
            },
            (_, Some(e)) => {
                match self {
                    SerError::BufferNotFilled(mut b) => {
                        b.unsplit(e);
                        SerError::BufferNotFilled(b)
                    }
                    SerError::UnexpectedEOB(mut b) => {
                        b.unsplit(e);
                        SerError::UnexpectedEOB(b)
                    }
                    SerError::Msg(m, mut b) => {
                        b.unsplit(e);
                        SerError::Msg(m, b)
                    }
                    SerError::IOError(err, mut b) => {
                        b.unsplit(e);
                        SerError::IOError(err, b)
                    }
                }
            },
            (_, _) => self,
        }
    }
}

impl Display for SerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SerError::Msg(ref s, _) => write!(f, "{}", s),
            SerError::IOError(ref e, _) => write!(f, "IO error while serializing: {}", e),
            SerError::UnexpectedEOB(_) => write!(
                f,
                "The buffer was too small for the requested serialization operation",
            ),
            SerError::BufferNotFilled(_) => write!(
                f,
                "The number of bytes written to the buffer did not fill the \
                given space",
            ),
        }
    }
}

impl Error for SerError {}

/// Deserialization error
#[derive(Debug)]
pub enum DeError {
    /// Abitrary error message 
    Msg(String),
    /// The end of the buffer was reached before deserialization finished
    UnexpectedEOB,
    /// Deserialization did not fill the buffer
    BufferNotParsed,
    /// A null byte was found before the end of the serialized `String`
    NullError,
    /// A null byte was not found at the end of the serialized `String`
    NoNullError,
}

impl DeError {
    /// Create new error from `&str`
    pub fn new<D>(s: D) -> Self where D: Display {
        DeError::Msg(s.to_string())
    }
}

try_err_compat!(
    DeError,
    io::Error,
    str::Utf8Error,
    string::FromUtf8Error,
    std::ffi::FromBytesWithNulError
);

impl Display for DeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DeError::Msg(ref s) => write!(f, "{}", s),
            DeError::UnexpectedEOB => write!(
                f,
                "The buffer was not large enough to complete the deserialize \
                operation",
            ),
            DeError::BufferNotParsed => write!(
                f,
                "Unparsed data left in buffer",
            ),
            DeError::NullError => write!(
                f,
                "A null was found before the end of the buffer",
            ),
            DeError::NoNullError => write!(
                f,
                "No terminating null byte was found in the buffer",
            ),
        }
    }
}

impl Error for DeError {}

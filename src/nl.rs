//! This module contains the top level netlink header code and attribute parsing. Every netlink
//! message will be encapsulated in a top level `Nlmsghdr`.
//!
//! `Nlmsghdr` is the structure representing a header that all netlink protocols require to be
//! passed to the correct destination.
//!
//! # Design decisions
//!
//! Payloads for `Nlmsghdr` can be any type that implements the `Nl` trait.

use bytes::{Bytes, BytesMut};

use crate::{
    consts::{alignto, NlType, NlmFFlags},
    err::{DeError, SerError},
    Nl,
};

/// Top level netlink header and payload
#[derive(Debug, PartialEq)]
pub struct Nlmsghdr<T, P> {
    /// Length of the netlink message
    pub nl_len: u32,
    /// Type of the netlink message
    pub nl_type: T,
    /// Flags indicating properties of the request or response
    pub nl_flags: NlmFFlags,
    /// Sequence number for netlink protocol
    pub nl_seq: u32,
    /// ID of the netlink destination for requests and source for responses
    pub nl_pid: u32,
    /// Payload of netlink message
    pub nl_payload: P,
}

impl<T, P> Nlmsghdr<T, P>
where
    T: NlType,
    P: Nl,
{
    /// Create a new top level netlink packet with a payload
    pub fn new(
        nl_len: Option<u32>,
        nl_type: T,
        nl_flags: NlmFFlags,
        nl_seq: Option<u32>,
        nl_pid: Option<u32>,
        nl_payload: P,
    ) -> Self {
        let mut nl = Nlmsghdr {
            nl_type,
            nl_flags,
            nl_seq: nl_seq.unwrap_or(0),
            nl_pid: nl_pid.unwrap_or(0),
            nl_payload,
            nl_len: 0,
        };
        nl.nl_len = nl_len.unwrap_or(nl.size() as u32);
        nl
    }
}

impl<T, P> Nl for Nlmsghdr<T, P>
where
    T: NlType,
    P: Nl,
{
    fn serialize(&self, mem: BytesMut) -> Result<BytesMut, SerError> {
        Ok(serialize! {
            PAD self;
            mem;
            self.nl_len, size;
            self.nl_type, size;
            self.nl_flags, size;
            self.nl_seq, size;
            self.nl_pid, size;
            self.nl_payload, size
        })
    }

    fn deserialize(mem: Bytes) -> Result<Self, DeError> {
        Ok(deserialize! {
            STRIP Self;
            mem;
            Nlmsghdr {
                nl_len: u32 => deserialize_type_size!(u32 => type_size),
                nl_type: T => deserialize_type_size!(T => type_size),
                nl_flags: NlmFFlags => deserialize_type_size!(NlmFFlags => type_size),
                nl_seq: u32 => deserialize_type_size!(u32 => type_size),
                nl_pid: u32 => deserialize_type_size!(u32 => type_size),
                nl_payload: P => deserialize_type_size!(P => type_size)
            } => alignto(nl_len as usize) - nl_len as usize
        })
    }

    fn size(&self) -> usize {
        self.nl_len.size()
            + <T as Nl>::size(&self.nl_type)
            + self.nl_flags.size()
            + self.nl_seq.size()
            + self.nl_pid.size()
            + self.nl_payload.size()
    }

    fn type_size() -> Option<usize> {
        u32::type_size()
            .and_then(|sz| T::type_size().map(|subsz| sz + subsz))
            .and_then(|sz| NlmFFlags::type_size().map(|subsz| sz + subsz))
            .and_then(|sz| u32::type_size().map(|subsz| sz + subsz))
            .and_then(|sz| u32::type_size().map(|subsz| sz + subsz))
            .and_then(|sz| P::type_size().map(|subsz| sz + subsz))
    }
}

/// Struct indicating an empty payload
#[derive(Debug, PartialEq)]
pub struct NlEmpty;

impl Nl for NlEmpty {
    #[inline]
    fn serialize(&self, mem: BytesMut) -> Result<BytesMut, SerError> {
        Ok(mem)
    }

    #[inline]
    fn deserialize(_mem: Bytes) -> Result<Self, DeError> {
        Ok(NlEmpty)
    }

    #[inline]
    fn size(&self) -> usize {
        0
    }

    #[inline]
    fn type_size() -> Option<usize> {
        Some(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::io::Cursor;

    use byteorder::{NativeEndian, WriteBytesExt};

    use crate::consts::nl::{NlmF, Nlmsg};

    #[test]
    fn test_nlmsghdr_serialize() {
        let nl = Nlmsghdr::<Nlmsg, NlEmpty>::new(
            None,
            Nlmsg::Noop,
            NlmFFlags::empty(),
            None,
            None,
            NlEmpty,
        );
        let mut mem = BytesMut::from(vec![0u8; nl.asize()]);
        mem = nl.serialize(mem).unwrap();
        let mut s = [0u8; 16];
        {
            let mut c = Cursor::new(&mut s as &mut [u8]);
            c.write_u32::<NativeEndian>(16).unwrap();
            c.write_u16::<NativeEndian>(1).unwrap();
        };
        assert_eq!(&s, mem.as_ref())
    }

    #[test]
    fn test_nlmsghdr_deserialize() {
        let mut s = [0u8; 16];
        {
            let mut c = Cursor::new(&mut s as &mut [u8]);
            c.write_u32::<NativeEndian>(16).unwrap();
            c.write_u16::<NativeEndian>(1).unwrap();
            c.write_u16::<NativeEndian>(NlmF::Ack.into()).unwrap();
        }
        let nl = Nlmsghdr::<Nlmsg, NlEmpty>::deserialize(Bytes::from(&s as &[u8])).unwrap();
        assert_eq!(
            Nlmsghdr::<Nlmsg, NlEmpty>::new(
                None,
                Nlmsg::Noop,
                NlmFFlags::new(&[NlmF::Ack]),
                None,
                None,
                NlEmpty
            ),
            nl
        );
    }
}

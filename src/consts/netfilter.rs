//! Constants for netfilter related protocols
//!
//! Note that this doesn't cover everything yet, both the list of types and variants in enums will
//! be added over time.

impl_var! {
    /// Attributes inside a netfilter log packet message.
    ///
    /// These are send by the kernel and describe a logged packet.
    NfLogAttr, u16,
    PacketHdr => 1,
    Mark => 2,
    Timestamp => 3,
    IfindexIndev => 4,
    IfindexOutdev => 5,
    IfindexPhyindev => 6,
    IfindexPhyoutdev => 7,
    Hwaddr => 8,
    Payload => 9,
    Prefix => 10,
    Uid => 11,
    Seq => 12,
    SeqGlobal => 13,
    Gid => 14,
    Hwtype => 15,
    Hwheader => 16,
    Hwlen => 17,
    Ct => 18,
    CtInfo => 19
}

impl_var! {
    /// Configuration attributes for netfilter logging.
    ///
    /// See [LogConfigReq][crate::netfilter::LogConfigReq]
    NfLogCfg, u16,
    Cmd => 1,
    Mode => 2,
    NlBufSize => 3,
    Timeout => 4,
    QThresh => 5,
    Flags => 6
}

impl_var! {
    /// Messages related to the netfilter netlink protocols.
    ///
    /// These appear on the [NlFamily::Netfilter][super::NlFamily::Netfilter] sockets.
    NetfilterMsg, u16,
    // TODO: Docs here /// A logged packet, going from kernel to userspace.
    LogPacket => 0x0400,
    // TODO: Docs here /// A logging configuration request, going from userspace to kernel.
    LogConfig => 0x0401
}

impl_trait! {
    /// Parameters for the [NfLogCfg::Cmd].
    LogCfgCmd, u8,
    /// Wrapper that is valid anywhere that accepts a value implementing the `LogCfgCmd` trait
    LogCfgCmdWrapper,
    LogCmd
}

impl_var! {
    /// Command value for the [NfLogCfg::Cmd].
    LogCmd, u8,
    Bind => 1,
    Unbind => 2,
    PfBind => 3,
    PfUnbind => 4
}

impl_var! {
    /// Copy mode of the logged packets.
    LogCopyMode, u8,
    None => 0,
    Meta => 1,
    Packet => 2
}

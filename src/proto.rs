// Constants from usbredirproto.h
pub const USB_REDIR_CAPS_SIZE: u32 = 1;
pub const USBREDIR_VERSION: u32 = 0x0000_0701;

use std::ptr;

use bitflags::bitflags;

// Constants from usbredirproto.h

bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct ParserFlags: u32 {
        const USB_HOST = 0x01;
        const WRITE_CB_OWNS_BUFFER = 0x02;
        const NO_HELLO = 0x04;
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct CapabilityFlags: u32 {
        /* Supports USB 3 bulk streams */
        const BULK_STREAMS = 1 << 0;
        /* The device_connect packet has the device_version_bcd field */
        const CONNECT_DEVICE_VERSION = 1 << 1;
        /* Supports usb_redir_filter_reject and usb_redir_filter_filter pkts */
        const FILTER = 1 << 2;
        /* Supports the usb_redir_device_disconnect_ack packet */
        const DEVICE_DISCONNECT_ACK = 1 << 3;
        /* The ep_info packet has the max_packet_size field */
        const EP_INFO_MAX_PACKET_SIZE = 1 << 4;
        /* Supports 64 bits ids in usb_redir_header */
        const IDS_64BITS = 1 << 5;
        /* Supports 32 bits length in usb_redir_bulk_packet_header */
        const BULK_LENGTH_32BITS = 1 << 6;
        /* Supports bulk receiving / buffered bulk input */
        const BULK_RECEIVING = 1 << 7;
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum UsbPacketType {
    #[default]
    Hello = 0,
    DeviceConnect = 1,
    DeviceDisconnect = 2,
    Reset = 3,
    InterfaceInfo = 4,
    EpInfo = 5,
    SetConfiguration = 6,
    GetConfiguration = 7,
    ConfigurationStatus = 8,
    SetAltSetting = 9,
    GetAltSetting = 10,
    AltSettingStatus = 11,
    StartIsoStream = 12,
    StopIsoStream = 13,
    IsoStreamStatus = 14,
    StartInterruptReceiving = 15,
    StopInterruptReceiving = 16,
    InterruptReceivingStatus = 17,
    AllocBulkStreams = 18,
    FreeBulkStreams = 19,
    BulkStreamsStatus = 20,
    CancelDataPacket = 21,
    FilterReject = 22,
    FilterFilter = 23,
    DeviceDisconnectAck = 24,
    StartBulkReceiving = 25,
    StopBulkReceiving = 26,
    BulkReceivingStatus = 27,

    // Data packets
    ControlPacket = 100,
    BulkPacket = 101,
    IsoPacket = 102,
    InterruptPacket = 103,
    BufferedBulkPacket = 104,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum UsbEndpointType {
    Control = 0,
    Iso = 1,
    Bulk = 2,
    Interrupt = 3,
    #[default]
    Invalid = 255,
}

/// Trait for types that can be safely viewed as bytes.
/// # Safety
/// Implementor must ensure the type has no padding or invalid bit patterns.
pub unsafe trait AsBytes: Sized {
    fn as_bytes(&self) -> &[u8] {
        // SAFETY: The trait helper guarantees self is valid for byte viewing.
        unsafe {
            std::slice::from_raw_parts(
                (ptr::from_ref(self)).cast::<u8>(),
                std::mem::size_of::<Self>(),
            )
        }
    }
}

#[repr(C, packed)]
#[derive(Default, Clone, Copy)]
pub struct UsbRedirHeader {
    pub type_: u32,
    pub length: u32,
    pub id: u64,
}

unsafe impl AsBytes for UsbRedirHeader {}

#[repr(C, packed)]
pub struct UsbRedirHelloHeader {
    pub version: [u8; 64],
    pub capabilities: [u32; 0], // Flexible array member
}

unsafe impl AsBytes for UsbRedirHelloHeader {}

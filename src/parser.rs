use crate::proto::{
    CapabilityFlags, ParserFlags, USB_REDIR_CAPS_SIZE, UsbPacketType, UsbRedirHeader,
    UsbRedirHelloHeader,
};
use std::ffi::{c_char, c_int, c_void};
use std::mem;

// Re-export constants if needed for the parser header, or just rely on proto header inclusion.
// But for now, we use them internally.

// Flags for usbredirparser_init
pub const USBREDIRPARSER_FL_USB_HOST: c_int = 0x01;
pub const USBREDIRPARSER_FL_WRITE_CB_OWNS_BUFFER: c_int = 0x02;
pub const USBREDIRPARSER_FL_NO_HELLO: c_int = 0x04;

// Structs from usbredirparser.h

// Callback type definitions
pub type UsbRedirParserLog =
    Option<unsafe extern "C" fn(priv_: *mut c_void, level: c_int, msg: *const c_char)>;
pub type UsbRedirParserRead =
    Option<unsafe extern "C" fn(priv_: *mut c_void, data: *mut u8, count: c_int) -> c_int>;
pub type UsbRedirParserWrite =
    Option<unsafe extern "C" fn(priv_: *mut c_void, data: *mut u8, count: c_int) -> c_int>;

pub type UsbRedirParserCallback = Option<unsafe extern "C" fn()>; // Generic placeholder

#[repr(C)]
pub struct usbredirparser {
    pub r#priv: *mut c_void,
    pub log_func: UsbRedirParserLog,
    pub read_func: UsbRedirParserRead,
    pub write_func: UsbRedirParserWrite,
    pub device_connect_func: UsbRedirParserCallback,
    pub device_disconnect_func: UsbRedirParserCallback,
    pub reset_func: UsbRedirParserCallback,
    pub interface_info_func: UsbRedirParserCallback,
    pub ep_info_func: UsbRedirParserCallback,
    pub set_configuration_func: UsbRedirParserCallback,
    pub get_configuration_func: UsbRedirParserCallback,
    pub configuration_status_func: UsbRedirParserCallback,
    pub set_alt_setting_func: UsbRedirParserCallback,
    pub get_alt_setting_func: UsbRedirParserCallback,
    pub alt_setting_status_func: UsbRedirParserCallback,
    pub start_iso_stream_func: UsbRedirParserCallback,
    pub stop_iso_stream_func: UsbRedirParserCallback,
    pub iso_stream_status_func: UsbRedirParserCallback,
    pub start_interrupt_receiving_func: UsbRedirParserCallback,
    pub stop_interrupt_receiving_func: UsbRedirParserCallback,
    pub interrupt_receiving_status_func: UsbRedirParserCallback,
    pub alloc_bulk_streams_func: UsbRedirParserCallback,
    pub free_bulk_streams_func: UsbRedirParserCallback,
    pub bulk_streams_status_func: UsbRedirParserCallback,
    pub cancel_data_packet_func: UsbRedirParserCallback,
    pub control_packet_func: UsbRedirParserCallback,
    pub bulk_packet_func: UsbRedirParserCallback,
    pub iso_packet_func: UsbRedirParserCallback,
    pub interrupt_packet_func: UsbRedirParserCallback,
    pub alloc_lock_func: Option<unsafe extern "C" fn() -> *mut c_void>,
    pub lock_func: Option<unsafe extern "C" fn(*mut c_void)>,
    pub unlock_func: Option<unsafe extern "C" fn(*mut c_void)>,
    pub free_lock_func: Option<unsafe extern "C" fn(*mut c_void)>,
    pub hello_func: UsbRedirParserCallback,
    pub filter_reject_func: UsbRedirParserCallback,
    pub filter_filter_func: UsbRedirParserCallback,
    pub device_disconnect_ack_func: UsbRedirParserCallback,
    pub start_bulk_receiving_func: UsbRedirParserCallback,
    pub stop_bulk_receiving_func: UsbRedirParserCallback,
    pub bulk_receiving_status_func: UsbRedirParserCallback,
    pub buffered_bulk_packet_func: UsbRedirParserCallback,
}

impl Default for usbredirparser {
    fn default() -> Self {
        // SAFETY: All fields are option/pointers, so zeroed is valid.
        unsafe { mem::zeroed() }
    }
}

// Internal parser struct (mimics usbredirparser_priv)
#[repr(C)]
pub struct Parser {
    pub interface: usbredirparser, // Must be first
    pub flags: ParserFlags,
    pub have_peer_caps: bool,
    pub our_caps: CapabilityFlags,
    pub peer_caps: CapabilityFlags,

    pub lock: *mut c_void, // User-supplied lock

    // Internal state matching usbredirparser_priv
    pub header: UsbRedirHeader,
    pub type_header: Vec<u8>, // Dynamic buffer for packet type header
    pub header_read: usize,
    pub type_header_len: usize,
    pub type_header_read: usize,
    pub data: Vec<u8>, // Buffer for incoming data
    pub data_read: usize,
    pub to_skip: usize,

    pub write_buffer: Vec<u8>, // Vector to store queued data
}

impl Default for Parser {
    fn default() -> Self {
        Self {
            interface: usbredirparser::default(),
            flags: ParserFlags::empty(),
            have_peer_caps: false,
            our_caps: CapabilityFlags::empty(),
            peer_caps: CapabilityFlags::empty(),
            lock: std::ptr::null_mut(),
            header: UsbRedirHeader::default(),
            type_header: Vec::new(),
            header_read: 0,
            type_header_len: 0,
            type_header_read: 0,
            data: Vec::new(),
            data_read: 0,
            to_skip: 0,
            write_buffer: Vec::new(),
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn usbredirparser_create() -> *mut usbredirparser {
    let parser = Box::new(Parser::default());
    Box::into_raw(parser) as *mut usbredirparser
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn usbredirparser_destroy(parser_ptr: *mut usbredirparser) {
    if parser_ptr.is_null() {
        return;
    }
    // SAFETY: caller guarantees parser_ptr is valid, and we know it was created via Box::into_raw.
    // Reconstruct the Box to deallocate it.
    let parser = unsafe { Box::from_raw(parser_ptr as *mut Parser) };

    // If a lock was allocated, free it.
    if let Some(free_func) = parser.interface.free_lock_func {
        if !parser.lock.is_null() {
            // SAFETY: free_func is a valid function pointer from C interface, and parser.lock was allocated by alloc_func.
            unsafe { free_func(parser.lock) };
        }
    }
    // The Box will be dropped here, deallocating the Parser.
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn usbredirparser_init(
    parser_ptr: *mut usbredirparser,
    version: *const c_char,
    caps: *const u32,
    caps_len: c_int,
    flags: c_int,
) {
    if parser_ptr.is_null() {
        return;
    }
    // SAFETY: caller guarantees parser_ptr is valid.
    let parser = unsafe { &mut *(parser_ptr as *mut Parser) };

    parser.flags = ParserFlags::from_bits_truncate(flags as u32) & !ParserFlags::NO_HELLO;

    // Allocate lock if supported
    if let Some(alloc_func) = parser.interface.alloc_lock_func {
        parser.lock = unsafe { alloc_func() };
    }

    // Copy capabilities
    let len = if caps_len > USB_REDIR_CAPS_SIZE as c_int {
        USB_REDIR_CAPS_SIZE
    } else {
        caps_len as usize
    };
    if !caps.is_null() && len > 0 {
        // SAFETY: caps is valid for 1 element.
        // It has been over a decade and caps has not grown over 32bits
        if len >= 1 {
            let raw_caps = unsafe { *caps };
            parser.our_caps = CapabilityFlags::from_bits_truncate(raw_caps);
        }
    }

    // Reset internal state (similar to usbredirparser_reset in C, but manual here for now)
    parser.header_read = 0;
    parser.type_header_read = 0;
    parser.type_header.clear();
    parser.data_read = 0;
    parser.to_skip = 0;
    parser.write_buffer.clear();

    // Send hello if needed
    if (flags & ParserFlags::NO_HELLO.bits() as i32) == 0 {
        // Prepare Version String
        let mut version_arr = [0u8; 64];
        if !version.is_null() {
            // SAFETY: trusting that version is a valid C string if not null.
            unsafe {
                let c_str = std::ffi::CStr::from_ptr(version);
                let bytes = c_str.to_bytes();
                let copy_len = std::cmp::min(bytes.len(), 63); // Leave one null byte
                version_arr[..copy_len].copy_from_slice(&bytes[..copy_len]);
            }
        }

        // Prepare Hello Header
        let hello = UsbRedirHelloHeader {
            version: version_arr,
            capabilities: [],
        };

        let caps_size = USB_REDIR_CAPS_SIZE * mem::size_of::<u32>();
        let packet_len = mem::size_of::<UsbRedirHelloHeader>() + caps_size;

        // Prepare Generic Header
        let header = UsbRedirHeader {
            type_: UsbPacketType::Hello as u32,
            length: packet_len as u32,
            id: 0,
        };

        // Serialize to write buffer
        // Header
        parser.write_buffer.extend_from_slice(header.as_bytes());

        // Hello Header
        parser.write_buffer.extend_from_slice(hello.as_bytes());

        // Caps
        // We need to serialize our CapabilityFlags back to bytes.
        // Since it's u32 transparent, we can write the bits directly.
        let caps_val = parser.our_caps.bits();
        parser
            .write_buffer
            .extend_from_slice(&caps_val.to_le_bytes());
    }
}

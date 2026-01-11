use std::ffi::CString;
use usbredir_safe::parser::{
    Parser, usbredirparser, usbredirparser_create, usbredirparser_destroy, usbredirparser_init,
};
use usbredir_safe::proto::{CapabilityFlags, ParserFlags, UsbPacketType};

// Helper to access the internal Parser struct from the public opaque pointer
unsafe fn get_parser(ptr: *mut usbredirparser) -> &'static mut Parser {
    unsafe { &mut *(ptr as *mut Parser) }
}

#[test]
fn test_create_and_init_basic() {
    unsafe {
        let parser_ptr = usbredirparser_create();
        assert!(!parser_ptr.is_null(), "Parser should not be null");

        let version = CString::new("0.7.1").unwrap();
        let mut caps = [0u32; 1];

        usbredirparser_init(
            parser_ptr,
            version.as_ptr(),
            caps.as_mut_ptr(),
            caps.len() as i32,
            0,
        );

        let parser = get_parser(parser_ptr);
        assert!(
            !parser.flags.contains(ParserFlags::NO_HELLO),
            "NO_HELLO should be cleared by default if not passed"
        );
        assert!(
            !parser.write_buffer.is_empty(),
            "Hello packet should be in write buffer"
        );

        // Verify Hello Packet
        // Expected structure: Header (16) + Hello Header (64) + Caps (4) = 84 bytes

        // Let's verify type
        let header_type_u32 = u32::from_le_bytes(parser.write_buffer[0..4].try_into().unwrap());
        assert_eq!(header_type_u32, UsbPacketType::Hello as u32);

        let _box = Box::from_raw(parser_ptr as *mut Parser);
    }
}

#[test]
fn test_init_with_flags_and_caps() {
    unsafe {
        let parser_ptr = usbredirparser_create();

        let version = CString::new("0.7.1").unwrap();
        // Set Bulk Streams capability
        let mut caps = [CapabilityFlags::BULK_STREAMS.bits()];

        let flags = ParserFlags::NO_HELLO.bits() | ParserFlags::USB_HOST.bits();

        usbredirparser_init(
            parser_ptr,
            version.as_ptr(),
            caps.as_mut_ptr(),
            caps.len() as i32,
            flags as i32,
        );

        let parser = get_parser(parser_ptr);

        // Check flags
        assert!(parser.flags.contains(ParserFlags::USB_HOST));
        // NO_HELLO should be cleared in stored flags
        assert!(!parser.flags.contains(ParserFlags::NO_HELLO));

        // Check caps
        assert!(parser.our_caps.contains(CapabilityFlags::BULK_STREAMS));
        assert!(!parser.our_caps.contains(CapabilityFlags::IDS_64BITS));

        // Check buffer is empty because NO_HELLO was set
        assert!(
            parser.write_buffer.is_empty(),
            "Write buffer should be empty with NO_HELLO"
        );

        let _box = Box::from_raw(parser_ptr as *mut Parser);
    }
}

#[test]
fn test_init_caps_conversion() {
    unsafe {
        let parser_ptr = usbredirparser_create();
        let version = CString::new("0.7.1").unwrap();

        // Test multiple bits
        let combined_caps = CapabilityFlags::BULK_STREAMS | CapabilityFlags::DEVICE_DISCONNECT_ACK;
        let mut caps = [combined_caps.bits()];

        usbredirparser_init(
            parser_ptr,
            version.as_ptr(),
            caps.as_mut_ptr(),
            caps.len() as i32,
            0,
        );

        let parser = get_parser(parser_ptr);
        assert_eq!(parser.our_caps, combined_caps);

        let _box = Box::from_raw(parser_ptr as *mut Parser);
    }
}

#[test]
fn test_lock_callbacks() {
    unsafe {
        let parser_ptr = usbredirparser_create();

        static mut LOCK_ALLOCATED: bool = false;
        static mut LOCK_FREED: bool = false;

        unsafe extern "C" fn alloc_lock() -> *mut std::ffi::c_void {
            unsafe {
                LOCK_ALLOCATED = true;
            }
            Box::into_raw(Box::new(42)) as *mut std::ffi::c_void
        }

        unsafe extern "C" fn free_lock(ptr: *mut std::ffi::c_void) {
            unsafe {
                LOCK_FREED = true;
            }
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }

        let parser = get_parser(parser_ptr);
        parser.interface.alloc_lock_func = Some(alloc_lock);
        parser.interface.free_lock_func = Some(free_lock);

        let version = CString::new("0.7.1").unwrap();
        let mut caps = [0u32; 1];

        usbredirparser_init(
            parser_ptr,
            version.as_ptr(),
            caps.as_mut_ptr(),
            caps.len() as i32,
            0,
        );

        // destroy should trigger free_lock
        usbredirparser_destroy(parser_ptr);

        assert!(LOCK_ALLOCATED, "alloc_lock should have been called");
        assert!(LOCK_FREED, "free_lock should have been called");
    }
}

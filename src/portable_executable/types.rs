#![allow(dead_code)]

// Header sections
pub const SECTION_TEXT: &str = ".text\0\0\0";
pub const SECTION_RDATA: &str = ".rdata\0\0";
pub const SECTION_DATA: &str = ".data\0\0\0";
pub const SECTION_PDATA: &str = ".pdata\0\0";
pub const SECTION_RETPLNE: &str = ".retplne";
pub const SECTION_TLS: &str = ".tls\0\0\0\0";
pub const SECTION_RELOC: &str = ".reloc\0\0";
pub const SECTION_RSRC: &str = ".rsrc\0\0\0";

// Resource Types
pub const RT_ICON: i64 = 0x0003;
pub const RT_ICON_GROUP: i64 = 0x000e;
pub const RT_VERSION: i64 = 0x0010;
pub const RT_MANIFEST: i64 = 0x0018;

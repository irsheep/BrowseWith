use std::fs::File;
use std::io::BufReader;
use std::io::SeekFrom;
use std::io::{ Seek, Read };
use std::io::Error;

use crate::portable_executable::WindowsPortableExecutable;
use crate::portable_executable::RESOURCES_ROOT_ADDRESS;

pub struct DosHeader {
  pub magic: [u8; 2],
  _null: [u8; 58], // Ignore other fileds
  pub e_lfanew: [u8; 4] // offset: 0x3c(60)
}
impl DosHeader {
  #[allow(dead_code)]
  pub fn new() -> Self {
    return Self {
      magic: [0; 2],
      _null: [0; 58],
      e_lfanew: [0; 4]
    };
  }
  #[allow(dead_code)]
  pub fn from_buffer(&mut self, buffer:&mut BufReader<File>) {
    *self = Self::to_object(buffer).unwrap();
  }
  #[allow(dead_code)]
  pub fn from_buffer_address(&mut self, buffer:&mut BufReader<File>, address:u64) {
    match buffer.seek(SeekFrom::Start(address)) {
      Ok(..) => { self.from_buffer(buffer); },
      Err(..) => { }
    }
  }
  #[allow(dead_code)]
  pub fn to_object(buffer:&mut BufReader<File>) -> Result<DosHeader, Error> {
    let mut data = DosHeader::new();

    buffer.read_exact(&mut data.magic)?;
    buffer.read_exact(&mut data._null)?;
    buffer.read_exact(&mut data.e_lfanew)?;

    return Ok(data);
  }
}

pub struct ELfanew {
  pub signature:[u8; 4],
  pub file_header: FileHeader,
  pub optional_header: OptionalHeader,
  pub data_directories: Vec<SectionDataDirectory>,
  pub sections: Vec<OptionalHeaderSection>
}
impl ELfanew {
  #[allow(dead_code)]
  pub fn new() -> Self {
    Self {
      signature: [0; 4],
      file_header: FileHeader::new(),
      optional_header: OptionalHeader::new(),
      data_directories: Vec::new(),
      sections: Vec::new()
    }
  }
  #[allow(dead_code)]
  pub fn from_buffer(&mut self, buffer:&mut BufReader<File>) {
    *self = Self::to_object(buffer).unwrap();
  }
  #[allow(dead_code)]
  pub fn from_buffer_address(&mut self, buffer:&mut BufReader<File>, address:u64) {
    match buffer.seek(SeekFrom::Start(address)) {
      Ok(..) => { self.from_buffer(buffer); },
      Err(..) => { }
    }
  }
  #[allow(dead_code)]
  pub fn to_object(buffer:&mut BufReader<File>) -> Result<ELfanew, Error> {
    let mut data = ELfanew::new();
    let mut file_header:FileHeader = FileHeader::new();
    let mut optional_header:OptionalHeader = OptionalHeader::new();

    // buffer.seek(SeekFrom::Start(address));
    buffer.read_exact(&mut data.signature)?;

    file_header.from_buffer(buffer);
    optional_header.from_buffer(buffer);

    data.file_header = file_header;
    data.optional_header = optional_header;

    for _i in 0..data.optional_header.number_of_rva_and_sizes.as_i32() {
      data.data_directories.push(SectionDataDirectory::to_object(buffer).unwrap());
    }

    for _i in 0..data.file_header.number_of_sections.as_i32() {
      data.sections.push(OptionalHeaderSection::to_object(buffer).unwrap());
    }

    return Ok(data);
  }
}

pub struct FileHeader {
  pub machine:[u8; 2],
  pub number_of_sections: [u8; 2],
  pub time_date_stamp: [u8; 4],
  pub pointer_to_symbol_table: [u8; 4],
  pub number_of_symbols: [u8; 4],
  pub size_of_optional_header: [u8; 2],
  pub characteristics: [u8; 2]
}
impl FileHeader {
  #[allow(dead_code)]
  pub fn new() -> Self {
    Self {
      machine: [0; 2],
      number_of_sections: [0; 2],
      time_date_stamp: [0; 4],
      pointer_to_symbol_table: [0; 4],
      number_of_symbols: [0; 4],
      size_of_optional_header: [0; 2],
      characteristics: [0; 2]
    }
  }
  #[allow(dead_code)]
  pub fn from_buffer(&mut self, buffer:&mut BufReader<File>) {
    *self = Self::to_object(buffer).unwrap();
  }
  #[allow(dead_code)]
  pub fn from_buffer_address(&mut self, buffer:&mut BufReader<File>, address:u64) {
    match buffer.seek(SeekFrom::Start(address)) {
      Ok(..) => { self.from_buffer(buffer); },
      Err(..) => { }
    }
  }
  #[allow(dead_code)]
  pub fn to_object(buffer:&mut BufReader<File>) -> Result<FileHeader, Error> {
    let mut data:FileHeader = FileHeader::new();

    buffer.read_exact(&mut data.machine)?;
    buffer.read_exact(&mut data.number_of_sections)?;
    buffer.read_exact(&mut data.time_date_stamp)?;
    buffer.read_exact(&mut data.pointer_to_symbol_table)?;
    buffer.read_exact(&mut data.number_of_symbols)?;
    buffer.read_exact(&mut data.size_of_optional_header)?;
    buffer.read_exact(&mut data.characteristics)?;

    return Ok(data);
  }
}

pub struct OptionalHeader {
  pub magic: [u8; 2],
  pub major_linker_version: [u8; 1],
  pub minor_linker_version: [u8; 1],
  pub size_of_code: [u8; 4],
  pub size_of_inilialized_data: [u8; 4],
  pub size_of_uninitialized_data: [u8; 4],
  pub address_of_entry_point: [u8; 4],
  pub base_of_code: [u8; 4],
  pub image_base: [u8; 8],
  pub section_alignment: [u8; 4],
  pub file_alignment: [u8; 4],
  pub major_operating_system_version: [u8; 2],
  pub minor_operating_system_version: [u8; 2],
  pub major_image_version: [u8; 2],
  pub minor_image_version: [u8; 2],
  pub major_subsystem_version: [u8; 2],
  pub minor_subsystem_version: [u8; 2],
  pub win32_version_value: [u8; 4],
  pub size_of_image: [u8; 4],
  pub size_of_headers: [u8; 4],
  pub checksum: [u8; 4],
  pub subsystem: [u8; 2],
  pub dll_characteristics: [u8; 2],
  pub size_of_stack_reserve: [u8; 8],
  pub size_of_stack_commit: [u8; 8],
  pub size_of_heap_reserve: [u8; 8],
  pub size_of_heap_commit: [u8; 8],
  pub loader_flags: [u8; 4],
  pub number_of_rva_and_sizes: [u8; 4] // Number of OptionalHeaderSections
}
impl OptionalHeader {
  #[allow(dead_code)]
  pub fn new() -> Self {
    Self {
      magic: [0; 2],
      major_linker_version: [0; 1],
      minor_linker_version: [0; 1],
      size_of_code: [0; 4],
      size_of_inilialized_data: [0; 4],
      size_of_uninitialized_data: [0; 4],
      address_of_entry_point: [0; 4],
      base_of_code: [0; 4],
      image_base: [0; 8],
      section_alignment: [0; 4],
      file_alignment: [0; 4],
      major_operating_system_version: [0; 2],
      minor_operating_system_version: [0; 2],
      major_image_version: [0; 2],
      minor_image_version: [0; 2],
      major_subsystem_version: [0; 2],
      minor_subsystem_version: [0; 2],
      win32_version_value: [0; 4],
      size_of_image: [0; 4],
      size_of_headers: [0; 4],
      checksum: [0; 4],
      subsystem: [0; 2],
      dll_characteristics: [0; 2],
      size_of_stack_reserve: [0; 8],
      size_of_stack_commit: [0; 8],
      size_of_heap_reserve: [0; 8],
      size_of_heap_commit: [0; 8],
      loader_flags: [0; 4],
      number_of_rva_and_sizes: [0; 4],
    }
  }
  #[allow(dead_code)]
  pub fn from_buffer(&mut self, buffer:&mut BufReader<File>) {
    *self = Self::to_object(buffer).unwrap();
  }
  #[allow(dead_code)]
  pub fn from_buffer_address(&mut self, buffer:&mut BufReader<File>, address:u64) {
    match buffer.seek(SeekFrom::Start(address)) {
      Ok(..) => { self.from_buffer(buffer); },
      Err(..) => { }
    }
  }
  #[allow(dead_code)]
  pub fn to_object(buffer:&mut BufReader<File>) -> Result<OptionalHeader, Error> {
    let mut data:OptionalHeader = OptionalHeader::new();

    buffer.read_exact(&mut data.magic)?;
    buffer.read_exact(&mut data.major_linker_version)?;
    buffer.read_exact(&mut data.minor_linker_version)?;
    buffer.read_exact(&mut data.size_of_code)?;
    buffer.read_exact(&mut data.size_of_inilialized_data)?;
    buffer.read_exact(&mut data.size_of_uninitialized_data)?;
    buffer.read_exact(&mut data.address_of_entry_point)?;
    buffer.read_exact(&mut data.base_of_code)?;
    buffer.read_exact(&mut data.image_base)?;
    buffer.read_exact(&mut data.section_alignment)?;
    buffer.read_exact(&mut data.file_alignment)?;
    buffer.read_exact(&mut data.major_operating_system_version)?;
    buffer.read_exact(&mut data.minor_operating_system_version)?;
    buffer.read_exact(&mut data.major_image_version)?;
    buffer.read_exact(&mut data.minor_image_version)?;
    buffer.read_exact(&mut data.major_subsystem_version)?;
    buffer.read_exact(&mut data.minor_subsystem_version)?;
    buffer.read_exact(&mut data.win32_version_value)?;
    buffer.read_exact(&mut data.size_of_image)?;
    buffer.read_exact(&mut data.size_of_headers)?;
    buffer.read_exact(&mut data.checksum)?;
    buffer.read_exact(&mut data.subsystem)?;
    buffer.read_exact(&mut data.dll_characteristics)?;
    buffer.read_exact(&mut data.size_of_stack_reserve)?;
    buffer.read_exact(&mut data.size_of_stack_commit)?;
    buffer.read_exact(&mut data.size_of_heap_reserve)?;
    buffer.read_exact(&mut data.size_of_heap_commit)?;
    buffer.read_exact(&mut data.loader_flags)?;
    buffer.read_exact(&mut data.number_of_rva_and_sizes)?;
    // buffer.read_exact(&mut self.)?;

    return Ok(data);
  }
}

#[derive(Copy, Clone)]
pub struct OptionalHeaderSection { // Size: 28 bytes
  pub name: [u8; 8],
  pub physical_address: [u8; 4], // AKA virtual size?
  pub virtual_address: [u8; 4],
  pub size_of_raw_data: [u8; 4],
  pub pointer_to_raw_data: [u8; 4],
  pub pointer_to_relocations: [u8; 4],
  pub pointer_to_line_numbers: [u8; 4],
  pub number_of_relocations: [u8; 2],
  pub number_of_line_numbers: [u8; 2],
  pub characteristics: [u8; 4]
}
impl OptionalHeaderSection {
  #[allow(dead_code)]
  pub fn new() -> Self {
    Self {
      name: [0; 8],
      physical_address: [0; 4],
      virtual_address: [0; 4],
      size_of_raw_data: [0; 4],
      pointer_to_raw_data: [0; 4],
      pointer_to_relocations: [0; 4],
      pointer_to_line_numbers: [0; 4],
      number_of_relocations: [0; 2],
      number_of_line_numbers: [0; 2],
      characteristics: [0; 4],
    }
  }
  #[allow(dead_code)]
  pub fn from_buffer(&mut self, buffer:&mut BufReader<File>) {
    *self = Self::to_object(buffer).unwrap();
  }
  #[allow(dead_code)]
  pub fn from_buffer_address(&mut self, buffer:&mut BufReader<File>, address:u64) {
    match buffer.seek(SeekFrom::Start(address)) {
      Ok(..) => { self.from_buffer(buffer); },
      Err(..) => { }
    }
  }
  #[allow(dead_code)]
  pub fn to_object(buffer:&mut BufReader<File>) -> Result<OptionalHeaderSection, Error> {
    let mut data:OptionalHeaderSection = OptionalHeaderSection::new();

    buffer.read_exact(&mut data.name)?;
    buffer.read_exact(&mut data.physical_address)?;
    buffer.read_exact(&mut data.virtual_address)?;
    buffer.read_exact(&mut data.size_of_raw_data)?;
    buffer.read_exact(&mut data.pointer_to_raw_data)?;
    buffer.read_exact(&mut data.pointer_to_relocations)?;
    buffer.read_exact(&mut data.pointer_to_line_numbers)?;
    buffer.read_exact(&mut data.number_of_relocations)?;
    buffer.read_exact(&mut data.number_of_line_numbers)?;
    buffer.read_exact(&mut data.characteristics)?;
    // buffer.read_exact(&mut self.)?;

    return Ok(data);
  }
}

pub struct SectionDataDirectory {
  virtual_address: [u8; 4],
  size: [u8; 4]
}
impl SectionDataDirectory {
  #[allow(dead_code)]
  pub fn new() -> Self {
    Self {
      virtual_address: [0; 4],
      size: [0; 4]
    }
  }
  #[allow(dead_code)]
  pub fn from_buffer(&mut self, buffer:&mut BufReader<File>) {
    *self = Self::to_object(buffer).unwrap();
  }
  #[allow(dead_code)]
  pub fn from_buffer_address(&mut self, buffer:&mut BufReader<File>, address:u64) {
    match buffer.seek(SeekFrom::Start(address)) {
      Ok(..) => { self.from_buffer(buffer); },
      Err(..) => { }
    }
  }
  #[allow(dead_code)]
  pub fn to_object(buffer:&mut BufReader<File>) -> Result<SectionDataDirectory, Error> {
    let mut data:SectionDataDirectory = SectionDataDirectory::new();

    buffer.read_exact(&mut data.virtual_address)?;
    buffer.read_exact(&mut data.size)?;

    return Ok(data);
  }
}

//
pub struct ResourceDataDirectory {
  // address_pointer: u64,
  pub characteristics: [u8; 4],
  pub time_date_stamp: [u8; 4],
  pub major_version: [u8; 2],
  pub minor_version: [u8; 2],
  pub number_of_named_entries: [u8; 2],
  pub number_of_id_entries: [u8; 2],
  // DirectoryEntries Follows...
  pub directories: Vec<DirectoryEntry>
}
impl ResourceDataDirectory {
  #[allow(dead_code)]
  pub fn new() -> Self {
    Self {
      // address_pointer: 0,
      characteristics: [0; 4],
      time_date_stamp: [0; 4],
      major_version: [0; 2],
      minor_version: [0; 2],
      number_of_named_entries: [0; 2],
      number_of_id_entries: [0; 2],
      directories: Vec::new()
    }
  }
  #[allow(dead_code)]
  pub fn from_buffer(&mut self, buffer:&mut BufReader<File>) {
    *self = Self::to_object(buffer).unwrap();
  }
  #[allow(dead_code)]
  pub fn from_buffer_address(&mut self, buffer:&mut BufReader<File>, address:u64) {
    match buffer.seek(SeekFrom::Start(address)) {
      Ok(..) => { self.from_buffer(buffer); },
      Err(..) => { }
    }
  }
  #[allow(dead_code)]
  pub fn to_object(buffer:&mut BufReader<File>) -> Result<ResourceDataDirectory, Error> {
    let mut data:ResourceDataDirectory = ResourceDataDirectory::new();
    let mut resources_root_address:u64 = 0;

    // data.address_pointer = buffer.stream_position().unwrap();
    // println!("ResourceDataDirectory->address: {}", data.address_pointer);

    // Load the root address for ResourceDirecotry and set it if not already set
    RESOURCES_ROOT_ADDRESS.with(|v| {resources_root_address = *v.borrow();});
    if resources_root_address == 0 {
      RESOURCES_ROOT_ADDRESS.with(|v| { *v.borrow_mut() = buffer.stream_position().unwrap(); });
    }

    buffer.read_exact(&mut data.characteristics)?;
    buffer.read_exact(&mut data.time_date_stamp)?;
    buffer.read_exact(&mut data.major_version)?;
    buffer.read_exact(&mut data.minor_version)?;
    buffer.read_exact(&mut data.number_of_named_entries)?;
    buffer.read_exact(&mut data.number_of_id_entries)?;
    // buffer.read_exact(&mut self.)?;

    for _i in 0..data.number_of_named_entries.as_i32() {
      data.directories.push(DirectoryEntry::to_object(buffer).unwrap());
    }
    for _i in 0..data.number_of_id_entries.as_i32() {
      data.directories.push(DirectoryEntry::to_object(buffer).unwrap());
    }
    return Ok(data);
  }
}

pub struct DirectoryEntry {
  pub resource_type_id: [u8; 4], // ResourceType ID
  pub offset: [u8; 4],
  pub child_resource: Option<ResourceDataDirectory>,
  pub data_entry_resource: Option<ResourceDataEntry>
}
impl DirectoryEntry {
  #[allow(dead_code)]
  pub fn new() -> Self {
    Self {
      resource_type_id: [0; 4],
      offset: [0; 4],
      child_resource: None,
      data_entry_resource: None
    }
  }
  #[allow(dead_code)]
  pub fn from_buffer(&mut self, buffer:&mut BufReader<File>) {
    *self = Self::to_object(buffer).unwrap();
  }
  #[allow(dead_code)]
  pub fn from_buffer_address(&mut self, buffer:&mut BufReader<File>, address:u64) {
    match buffer.seek(SeekFrom::Start(address)) {
      Ok(..) => { self.from_buffer(buffer); },
      Err(..) => { }
    }
  }
  #[allow(dead_code)]
  pub fn to_object(buffer:&mut BufReader<File>) -> Result<DirectoryEntry, Error> {
    let mut data:DirectoryEntry = DirectoryEntry::new();
    let mut resources_root_address:u64 = 0;
    let pointer_save:u64;

    // data.address_pointer = buffer.stream_position().unwrap();

    buffer.read_exact(&mut data.resource_type_id)?;
    buffer.read_exact(&mut data.offset)?;
    // println!("(Name)resource_type_id: {:02x?} \toffset: {:02x?}\toffset bytes: {}", data.resource_type_id, data.offset, data.offset.as_u64());

    pointer_save = buffer.stream_position().unwrap();
    RESOURCES_ROOT_ADDRESS.with(|v| {resources_root_address = *v.borrow();});
    // println!("Pointer: {}", (resources_root_address+data.offset.as_u31() as u64) as i64);

    buffer.seek(SeekFrom::Start( resources_root_address+data.offset.as_u31() as u64 ))?;
    if data.offset.high_bit_set() {
      data.child_resource = Some(ResourceDataDirectory::to_object(buffer).unwrap());
    } else {
      // println!("DataEntry");
      data.data_entry_resource = Some(ResourceDataEntry::to_object(buffer).unwrap());
    }

    buffer.seek(SeekFrom::Start(pointer_save))?;
    // exit(0);

    return Ok(data);
  }
}

#[derive(Copy, Clone)]
pub struct ResourceDataEntry {
  pub offset_to_data: [u8; 4],
  pub size: [u8; 4],
  pub code_page: [u8; 4],
  pub reserved: [u8; 4]
}
impl ResourceDataEntry {
  #[allow(dead_code)]
  pub fn new() -> Self {
    Self {
      offset_to_data: [0; 4],
      size: [0; 4],
      code_page: [0; 4],
      reserved: [0; 4]
    }
  }
  #[allow(dead_code)]
  pub fn from_buffer(&mut self, buffer:&mut BufReader<File>) {
    *self = Self::to_object(buffer).unwrap();
  }
  #[allow(dead_code)]
  pub fn from_buffer_address(&mut self, buffer:&mut BufReader<File>, address:u64) {
    match buffer.seek(SeekFrom::Start(address)) {
      Ok(..) => { self.from_buffer(buffer); },
      Err(..) => { }
    }
  }
  #[allow(dead_code)]
  pub fn to_object(buffer:&mut BufReader<File>) -> Result<ResourceDataEntry, Error> {
    let mut data:ResourceDataEntry = ResourceDataEntry::new();

    buffer.read_exact(&mut data.offset_to_data)?;
    buffer.read_exact(&mut data.size)?;
    buffer.read_exact(&mut data.code_page)?;
    buffer.read_exact(&mut data.reserved)?;

    return Ok(data);
  }
}

pub struct ResourceDirectoryString {
  pub length: [u8; 2],
  pub string: String
}
impl ResourceDirectoryString {
  #[allow(dead_code)]
  pub fn new() -> Self {
    Self {
      length: [0; 2],
      string: String::from("")
    }
  }
  #[allow(dead_code)]
  pub fn from_buffer(&mut self, buffer:&mut BufReader<File>) {
    *self = Self::to_object(buffer).unwrap();
  }
  #[allow(dead_code)]
  pub fn from_buffer_address(&mut self, buffer:&mut BufReader<File>, address:u64) {
    match buffer.seek(SeekFrom::Start(address)) {
      Ok(..) => { self.from_buffer(buffer); },
      Err(..) => { }
    }
  }
  #[allow(dead_code)]
  pub fn to_object(buffer:&mut BufReader<File>) -> Result<ResourceDirectoryString, Error> {
    let mut data:ResourceDirectoryString = ResourceDirectoryString::new();

    buffer.read_exact(&mut data.length)?;


    return Ok(data);
  }
}

pub struct IconDir {
  pub reserved: [u8; 2],
  pub image_type: [u8; 2],
  pub number_of_images: [u8; 2],
  pub icon_entries: Vec<IconDirEntry>
}
impl IconDir {
  #[allow(dead_code)]
  pub fn new() -> Self {
    return Self {
      reserved: [0; 2],
      image_type: [0; 2],
      number_of_images: [0; 2],
      icon_entries: Vec::new()
    };
  }
  #[allow(dead_code)]
  pub fn from_buffer(&mut self, buffer:&mut std::io::Cursor<Vec<u8>>) {
    *self = Self::to_object(buffer).unwrap();
  }
  #[allow(dead_code)]
  pub fn from_buffer_address(&mut self, buffer:&mut std::io::Cursor<Vec<u8>>, address:u64) {
    match buffer.seek(SeekFrom::Start(address)) {
      Ok(..) => { self.from_buffer(buffer); },
      Err(..) => { }
    }
  }
  #[allow(dead_code)]
  pub fn to_object(buffer:&mut std::io::Cursor<Vec<u8>>) -> Result<IconDir, Error> {
    let mut data:IconDir = IconDir::new();

    buffer.read_exact(&mut data.reserved)?;
    buffer.read_exact(&mut data.image_type)?;
    buffer.read_exact(&mut data.number_of_images)?;

    return Ok(data);
  }
}

pub struct IconDirEntry {
  pub width: [u8; 1],
  pub height: [u8; 1],
  pub reserved: [u8; 2],
  pub image_type: [u8; 2],
  pub bits_per_pixel: [u8; 2],
  pub size: [u8; 4],
  pub index: [u8; 2]  // This is the 'index' of the ICON 'DirectoryEntries' array
}
impl IconDirEntry {
  #[allow(dead_code)]
  fn new() -> Self {
    return Self {
      width: [0; 1],
      height: [0; 1],
      reserved: [0; 2],
      image_type: [0; 2],
      bits_per_pixel: [0; 2],
      size: [0; 4],
      index: [0; 2]
    };
  }
  #[allow(dead_code)]
  pub fn from_buffer(&mut self, buffer:&mut std::io::Cursor<Vec<u8>>) {
    *self = Self::to_object(buffer).unwrap();
  }
  #[allow(dead_code)]
  pub fn from_buffer_address(&mut self, buffer:&mut std::io::Cursor<Vec<u8>>, address:u64) {
    match buffer.seek(SeekFrom::Start(address)) {
      Ok(..) => { self.from_buffer(buffer); },
      Err(..) => { }
    }
  }
  #[allow(dead_code)]
  pub fn to_object(buffer:&mut std::io::Cursor<Vec<u8>>) -> Result<IconDirEntry, Error> {
    let mut data:IconDirEntry = IconDirEntry::new();

    buffer.read_exact(&mut data.width)?;
    buffer.read_exact(&mut data.height)?;
    buffer.read_exact(&mut data.reserved)?;
    buffer.read_exact(&mut data.image_type)?;
    buffer.read_exact(&mut data.bits_per_pixel)?;
    buffer.read_exact(&mut data.size)?;
    buffer.read_exact(&mut data.index)?;

    return Ok(data);
  }
}

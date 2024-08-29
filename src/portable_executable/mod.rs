use std::fs::File;
use std::io::{
  Cursor, Read, Write, Seek, SeekFrom,
  BufReader,
  Error
};

use std::cell::RefCell;

mod types;
use types::*;
mod structures;
use structures::*;

thread_local!{
  static RESOURCES_ROOT_ADDRESS:RefCell<u64> = RefCell::new(0);
  static SECTIONS:RefCell<Vec<OptionalHeaderSection>> = RefCell::new(Vec::new());
}

pub fn get_icon(file_path:&str, icon_index:usize, size:Option<i32>) -> Vec<u8> {

  let mut buf_reader = BufReader::new(File::open(file_path).unwrap());

  let mut dos_header:DosHeader = DosHeader::new();
  let mut e_lfanew:ELfanew = ELfanew::new();
  let mut resources:ResourceDataDirectory = ResourceDataDirectory::new();
  let rsrc_data:&OptionalHeaderSection;

  RESOURCES_ROOT_ADDRESS.with(|v| { *v.borrow_mut() = 0 });

  // Load PE structure, but not the actual data
  dos_header.from_buffer(&mut buf_reader);
  e_lfanew.from_buffer_address(&mut buf_reader, dos_header.e_lfanew.as_u64());
  rsrc_data = e_lfanew.sections.iter().find(|x| x.name == SECTION_RSRC.as_bytes()).unwrap();
  resources.from_buffer_address(&mut buf_reader, rsrc_data.pointer_to_raw_data.as_u64());

  SECTIONS.with(|v| { *v.borrow_mut() = (*e_lfanew.sections).to_vec() });

  let icon_resource_directories:Vec<Vec<u8>> = get_dataentry(&resources.directories, &mut buf_reader, RT_ICON);
  let icon_group:Vec<Vec<u8>> = get_dataentry(&resources.directories, &mut buf_reader, RT_ICON_GROUP);

  match size {
    Some(i) => {
      return icon_size_from_data(&icon_group[icon_index], &icon_resource_directories, i);
    },
    None => {
      return icon_from_data(&icon_group[icon_index], &icon_resource_directories);
    }
  }
}
pub fn save_icon(source_file:&str, icon_index:usize, destination_file:&str, size:Option<i32>) {
  let mut file:File;
  file = File::create(destination_file).unwrap();
  file.write_all(&get_icon(source_file, icon_index, size)).unwrap();
}

fn icon_from_data(icon_group:&Vec<u8>, icon_resource_directories:&Vec<Vec<u8>>) -> Vec<u8>{
  let mut cursor:Cursor<Vec<u8>> = Cursor::new(Vec::new());
  let mut icon_dir:IconDir = IconDir::new();
  let mut raw_icon:Vec<u8> = Vec::new();
  let mut offset:i32;
  let mut index:usize;

  cursor.write_all(&icon_group).unwrap();
  icon_dir.from_buffer_address(&mut cursor, 0);

  // Get Icon directory entry data and calculate initial offset
  for _i in 0..icon_dir.number_of_images.as_i64() {
    icon_dir.icon_entries.push(IconDirEntry::to_object(&mut cursor).unwrap());
  }
  offset = 6 + (16*icon_dir.number_of_images.as_i32());

  // Icon file header
  raw_icon.write(&icon_dir.reserved).unwrap();
  raw_icon.write(&icon_dir.image_type).unwrap();
  raw_icon.write(&icon_dir.number_of_images).unwrap();
  // Add each Icon properties, in header
  for entry in icon_dir.icon_entries.iter() {
    raw_icon.write(&entry.width).unwrap();
    raw_icon.write(&entry.height).unwrap();
    raw_icon.write(&[0; 2]).unwrap(); // Colors and reserved
    raw_icon.write(&entry.image_type).unwrap();
    raw_icon.write(&entry.bits_per_pixel).unwrap();
    raw_icon.write(&entry.size).unwrap();
    raw_icon.write(&offset.to_le_bytes()).unwrap();

    offset = offset + entry.size.as_i32();
  }
  // Copy icon image data
  for entry in icon_dir.icon_entries.iter() {
    index = (entry.index.as_i32()-1) as usize;
    raw_icon.write(&icon_resource_directories[index]).unwrap();
  }

  return raw_icon;
}

fn icon_size_from_data(icon_group:&Vec<u8>, icon_resource_directories:&Vec<Vec<u8>>, size:i32) -> Vec<u8>{
  let mut cursor:Cursor<Vec<u8>> = Cursor::new(Vec::new());
  let mut icon_dir:IconDir = IconDir::new();
  let mut raw_icon:Vec<u8> = Vec::new();
  let offset:i32 = 22;
  let mut index:usize = 0;
  let mut best_icon:usize = 0 ;
  let mut best_size:i32 = 65535;

  cursor.write_all(&icon_group).unwrap();
  icon_dir.from_buffer_address(&mut cursor, 0);

  // Get Icon directory entry data and calculate initial offset
  for _i in 0..icon_dir.number_of_images.as_i64() {
    icon_dir.icon_entries.push(IconDirEntry::to_object(&mut cursor).unwrap());
  }
  // offset = 22; //6 + (16*icon_dir.number_of_images.as_i32());

  // Icon file header
  raw_icon.write(&icon_dir.reserved).unwrap();
  raw_icon.write(&icon_dir.image_type).unwrap();
  raw_icon.write(&[1, 0]).unwrap();
  // Add each Icon properties, in header
  for entry in icon_dir.icon_entries.iter() {
    if
      entry.width.as_i32() >= size &&
      entry.width.as_i32() < best_size
    {
      best_icon = index;
      best_size = entry.width.as_i32();
    }
    index = index + 1;
  }

  let entry = &icon_dir.icon_entries[best_icon];
  raw_icon.write(&entry.width).unwrap();
  raw_icon.write(&entry.height).unwrap();
  raw_icon.write(&[0; 2]).unwrap(); // Colors and reserved
  raw_icon.write(&entry.image_type).unwrap();
  raw_icon.write(&entry.bits_per_pixel).unwrap();
  raw_icon.write(&entry.size).unwrap();
  raw_icon.write(&offset.to_le_bytes()).unwrap();
  raw_icon.write(&icon_resource_directories[(&entry.index.as_i32()-1) as usize]).unwrap();

  return raw_icon;
}

fn get_dataentry(directories:&Vec<DirectoryEntry>, buffer:&mut BufReader<File>, resource_type:i64) -> Vec<Vec<u8>> {

  let mut result:Vec<Vec<u8>> = Vec::new();
  let mut _icons:Vec<IconDirEntry> = Vec::new();

  for directory in directories.iter() {
    if directory.resource_type_id.as_i64() == resource_type {
      match directory.child_resource.as_ref() {
        Some(child_data_directory) => {
          let resource_data_entries:Option<Vec<ResourceDataEntry>> = get_resource_directoryentry_data(child_data_directory);
          for data_entry in resource_data_entries.as_ref().unwrap().iter() {
            match get_resource_dataentry_data(buffer, *data_entry) {
              Ok(resource_data) => {
                // println!("DATA: {:02X?}", resource_data);
                result.push(resource_data);
              }
              Err(..) => { }
            }
          }
        },
        None => { }
      }
    }
  }
  return result;
}

fn get_resource_directoryentry_data(resource_data_directory:&ResourceDataDirectory) -> Option<Vec<ResourceDataEntry>> {

  let mut data:Vec<ResourceDataEntry> = Vec::new();

  for directory in resource_data_directory.directories.iter() {
    match directory.child_resource.as_ref() {
      Some(directory_entry) => {
        // println!("Named directory: {}", directory.resource_type_id.high_bit_set());
        data.push(get_resource_directoryentry_data(directory_entry).unwrap()[0]);
      },
      None => {
        match directory.data_entry_resource.as_ref() {
          Some(data_entry) => {
            return Some(vec![*data_entry]);
          },
          None => { return None; }
        }
      }
    }
  }
  return Some(data);
}

fn get_resource_dataentry_data(buffer:&mut BufReader<File>, resource_data_entry:ResourceDataEntry) -> Result<Vec<u8>, Error> {
  let mut bytes = vec![0; resource_data_entry.size.as_u64() as usize];
  let physical_address:u64 = rva_to_pa(resource_data_entry.offset_to_data.as_u64());

  buffer.seek(SeekFrom::Start(physical_address))?;
  buffer.read_exact(&mut bytes)?;

  return Ok(bytes);
}

// Relative Virtual Address to Physical Address translation (address in raw file BEFORE mapping by loader)
fn rva_to_pa(relative_virtual_address:u64) -> u64 {
  let mut sections:Vec<OptionalHeaderSection> = Vec::new();
  SECTIONS.with(|v| sections = v.borrow().to_vec());
    for section in sections.iter() {
      if relative_virtual_address >= section.virtual_address.as_u64() && relative_virtual_address <= section.virtual_address.as_u64() + section.size_of_raw_data.as_u64() {
        let physical_address:u64 = (relative_virtual_address - section.virtual_address.as_u64()) + section.pointer_to_raw_data.as_u64();
        return physical_address;
      }
    }
    return 0;
}

#[allow(dead_code)]
trait WindowsPortableExecutable {
  fn as_u31(&self) -> u32;
  fn as_u63(&self) -> u64;
  fn as_i32(&self) -> i32;
  fn as_u32(&self) -> u32;
  fn as_i64(&self) -> i64;
  fn as_u64(&self) -> u64;
  fn high_bit_set(&self) -> bool;
}
impl WindowsPortableExecutable for [u8] {
  fn as_u31(&self) -> u32 {
    let mut bytes:[u8; 4] = [0; 4];
    for i in 0..self.len() {
      bytes[3-i] = self[i];
    }
    if 0x80 ^ bytes[0] == 0 {
      bytes[0] = 0x80 ^ bytes[0];
    }
    return u32::from_be_bytes( bytes );
  }
  fn as_u63(&self) -> u64 {
    let mut bytes:[u8; 8] = [0; 8];
    for i in 0..self.len() {
      bytes[7-i] = self[i];
    }
    return u64::from_be_bytes( bytes );
  }
  fn as_i32(&self) -> i32 {
    let mut bytes:[u8; 4] = [0; 4];
    for i in 0..self.len() {
      bytes[3-i] = self[i];
    }
    return i32::from_be_bytes(bytes);
  }
  fn as_u32(&self) -> u32 {
    let mut bytes:[u8; 4] = [0; 4];
    for i in 0..self.len() {
      bytes[3-i] = self[i];
    }
    return u32::from_be_bytes(bytes);
  }
  fn as_i64(&self) -> i64 {
    let mut bytes:[u8; 8] = [0; 8];
    for i in 0..self.len() {
      bytes[7-i] = self[i];
    }
    return i64::from_be_bytes(bytes);
  }
  fn as_u64(&self) -> u64 {
    let mut bytes:[u8; 8] = [0; 8];
    for i in 0..self.len() {
      bytes[7-i] = self[i];
    }
    return u64::from_be_bytes(bytes);
  }
  fn high_bit_set(&self) -> bool {
    let len:usize = self.len();
    if self[len-1] & 0x80 == 0x80 {
      return true;
    }
    return false;
  }
}

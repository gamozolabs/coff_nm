//! Parser for `DI` debug info files. We specfically just parse the COFF
//! data from them to get globals, functions, and line numbers

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, BufReader};
use std::mem::size_of;
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;

/// Wrapper type for `Result`
type Result<T> = std::result::Result<T, Error>;

/// Error types
#[derive(Debug)]
pub enum Error {
    /// Failed to open input file
    Open(PathBuf, std::io::Error),

    /// File was not a debug info file
    NotDebugInfo,

    /// Failed to consume a field from the file
    Consume(&'static str, std::io::Error),

    /// Exported name was not valid UTF-8
    ExportedNameUtf8(std::str::Utf8Error),
    
    /// Symbol table string name was not valid UTF-8
    StringNameUtf8(std::str::Utf8Error),
    
    /// A source filename had an invalid UTF-8 character
    FilenameUtf8(std::str::Utf8Error),

    /// A debug type specified in a [`DebugDirectory`] was invalid
    InvalidDebugType(u32),

    /// Failed to seek to the COFF section
    SeekCoff(std::io::Error),

    /// Failed to skip over exported names
    SkipExportedNames(std::io::Error),

    /// COFF debug referenced out-of-bounds string for symbol name
    SymbolNameOob,

    /// Got a symbol class that was unknown
    UnknownSymbolClass(u8),

    /// Failed to extract a file from the CAB
    ExtractCab(std::io::Error),
}

/// Consume bytes from a reader
macro_rules! consume {
    ($reader:expr, $ty:ty, $field:expr) => {{
        // Create buffer for type
        let mut tmp = [0u8; size_of::<$ty>()];

        // Read the bytes and convert
        $reader.read_exact(&mut tmp).map(|_| {
            <$ty>::from_le_bytes(tmp)
        }).map_err(|x| Error::Consume($field, x))
    }};

    ($reader:expr, $size:expr, $field:expr) => {{
        // Create buffer for type
        let mut tmp = [0u8; $size];

        // Read the bytes and convert
        $reader.read_exact(&mut tmp).map(|_| {
            tmp
        }).map_err(|x| Error::Consume($field, x))
    }};
}

/// Debug directory types
#[derive(Debug)]
#[repr(u32)]
enum DebugType {
    ///  Unknown value, ignored by all tools. 
    Unknown = 0,

    /// COFF debugging information (line numbers, symbol table, and
    /// string table). This type of debugging information is also
    /// pointed to by fields in the file headers. 
    Coff = 1,

    /// CodeView debugging information. The format of the data block is
    /// described by the CodeView 4.0 specification. 
    CodeView = 2,

    /// Frame pointer omission (FPO) information. This information
    /// tells the debugger how to interpret nonstandard stack frames,
    /// which use the EBP register for a purpose other than as a frame
    /// pointer. 
    FramePointerOmission = 3,

    /// Miscellaneous information. 
    Misc = 4,

    /// Exception information. 
    Exception = 5,

    /// Fixup information. 
    Fixup = 6,

    /// Omap to source
    OmapToSrc = 7,

    /// Omap from source
    OmapFromSrc = 8,

    /// Borland debugging information. 
    Borland = 9,
}

impl TryFrom<u32> for DebugType {
    type Error = Error;

    fn try_from(val: u32) -> Result<Self> {
        Ok(match val {
            0 => Self::Unknown,
            1 => Self::Coff,
            2 => Self::CodeView,
            3 => Self::FramePointerOmission,
            4 => Self::Misc,
            5 => Self::Exception,
            6 => Self::Fixup,
            7 => Self::OmapToSrc,
            8 => Self::OmapFromSrc,
            9 => Self::Borland,
            _ => return Err(Error::InvalidDebugType(val)),
        })
    }
}

/// `IMAGE_SECTION_HEADER`
#[derive(Debug)]
#[repr(C)]
struct SectionHeader {
    /// An 8-byte, null-padded UTF-8 string. There is no
    /// terminating null character if the string is exactly eight
    /// characters long. For longer names, this member contains a
    /// forward slash (/) followed by an ASCII representation of a
    /// decimal number that is an offset into the string table.
    /// Executable images do not use a string table and do not
    /// support section names longer than eight characters.
    name: [u8; 8],

    /// The total size of the section when loaded into memory, in
    /// bytes. If this value is greater than the SizeOfRawData
    /// member, the section is filled with zeroes. This field is
    /// valid only for executable images and should be set to 0 for
    /// object files.
    vsize: u32,

    /// The address of the first byte of the section when loaded
    /// into memory, relative to the image base. For object files,
    /// this is the address of the first byte before relocation is
    /// applied.
    vaddr: u32,

    /// The size of the initialized data on disk, in bytes. This
    /// value must be a multiple of the `FileAlignment` member of
    /// the `IMAGE_OPTIONAL_HEADER` structure. If this value is
    /// less than the VirtualSize member, the remainder of the
    /// section is filled with zeroes. If the section contains only
    /// uninitialized data, the member is zero.
    raw_data_sz: u32,

    /// A file pointer to the first page within the COFF file. This
    /// value must be a multiple of the `FileAlignment` member of
    /// the `IMAGE_OPTIONAL_HEADER` structure. If a section
    /// contains only uninitialized data, set this member is zero.
    ptr_raw_data: u32,

    /// A file pointer to the beginning of the relocation entries
    /// for the section. If there are no relocations, this value is
    /// zero.
    ptr_relocation: u32,

    /// A file pointer to the beginning of the line-number entries
    /// for the section. If there are no COFF line numbers, this
    /// value is zero.
    ptr_line_num: u32,

    /// The number of relocation entries for the section. This
    /// value is zero for executable images.
    num_relocs: u16,

    /// The number of line-number entries for the section.
    num_line_num: u16,

    /// The characteristics of the image. The following values are
    /// defined.
    characteristics: u32,
}

/// `IMAGE_DEBUG_DIRECTORY`
#[derive(Debug)]
#[repr(C)]
struct DebugDirectory {
    /// Reserved
    characteristics: u32,

    /// The time and date the debugging information was created.
    timedatestamp: u32,

    /// The major version number of the debugging information format.
    major_version: u16,

    /// The minor version number of the debugging information format.
    minor_version: u16,

    /// The format of the debugging information.
    typ: DebugType,

    /// The size of the debugging information, in bytes. This value
    /// does not include the debug directory itself.
    size_of_data: u32,

    /// The address of the debugging information when the image is
    /// loaded, relative to the image base.
    addr_raw_data: u32,

    /// A file pointer to the debugging information.
    ptr_raw_data: u32,
}

/// `IMAGE_COFF_SYMBOLS_HEADER`
#[derive(Debug)]
#[repr(C)]
struct CoffSymbolsHeader {
    /// The number of symbols.
    num_symbols: u32,

    /// The virtual address of the first symbol.
    lva_first_symbol: u32,

    /// The number of line-number entries.
    num_line_nums: u32,

    /// The virtual address of the first line-number entry.
    lva_first_line: u32,

    /// The relative virtual address of the first byte of code.
    rva_first_code: u32,

    /// The relative virtual address of the last byte of code.
    rva_last_code: u32,

    /// The relative virtual address of the first byte of data.
    rva_first_data: u32,

    /// The relative virtual address of the last byte of data.
    rva_last_data: u32,
}

/// Windows NT `.dbg` file parser
#[derive(Default)]
pub struct DbgFile {
    /// Mapping from RVA to (filename, line number)
    addr_to_line: BTreeMap<u32, (String, u32)>,

    /// Mapping from RVA to function name
    functions: BTreeMap<u32, String>,

    /// Mapping from RVA to global name
    globals: BTreeMap<u32, String>,
}

impl DbgFile {
    /// Parse a debug file at `path`
    pub fn load(mut reader: impl Read + Seek) -> Result<Self> {
        // Make sure it's a debug info file
        if &consume!(reader, 2, "header")? != b"DI" {
            return Err(Error::NotDebugInfo);
        }

        // `IMAGE_SEPARATE_DEBUG_HEADER`
        let _flags           = consume!(reader, u16, "flags")?;
        let _machine         = consume!(reader, u16, "machine")?;
        let _characteristics = consume!(reader, u16, "characteristics")?;
        let _timedatestamp   = consume!(reader, u32, "timedatestamp")?;
        let _checksum        = consume!(reader, u32, "checksum")?;
        let _image_base      = consume!(reader, u32, "image base")?;
        let _size_of_image   = consume!(reader, u32, "size of image")?;
        let num_sections     = consume!(reader, u32, "number of sections")?;
        let exported_namesz  = consume!(reader, u32, "exported names size")?;
        let debug_dirsz      = consume!(reader, u32, "debug directory size")?;
        let _section_align   = consume!(reader, u32, "section alignment")?;
        let _reserved        = consume!(reader, 8,   "reserved")?;

        // Read each `IMAGE_SECTION_HEADER`
        for _ in 0..num_sections {
            // Read the section header
            let _sh = SectionHeader {
                name:            consume!(reader, 8,   "name")?,
                vsize:           consume!(reader, u32, "vsize")?,
                vaddr:           consume!(reader, u32, "vaddr")?,
                raw_data_sz:     consume!(reader, u32, "raw_data_sz")?,
                ptr_raw_data:    consume!(reader, u32, "ptr_raw_data")?,
                ptr_relocation:  consume!(reader, u32, "ptr_relocation")?,
                ptr_line_num:    consume!(reader, u32, "ptr_line_num")?,
                num_relocs:      consume!(reader, u16, "num_relocs")?,
                num_line_num:    consume!(reader, u16, "num_line_num")?,
                characteristics: consume!(reader, u32, "characteristics")?,
            };
        }

        // Skip over the exported names
        reader.seek(SeekFrom::Current(exported_namesz as i64))
            .map_err(Error::SkipExportedNames)?;

        // Create return `Self`
        let mut ret = Self::default();

        // Read each `IMAGE_DEBUG_DIRECTORY`
        for _ in 0..debug_dirsz as usize / size_of::<DebugDirectory>() {
            // Read the section header
            let dd = DebugDirectory {
                characteristics: consume!(reader, u32, "characteristics")?,
                timedatestamp:   consume!(reader, u32, "timedatestamp")?,
                major_version:   consume!(reader, u16, "major_version")?,
                minor_version:   consume!(reader, u16, "minor_version")?,
                typ:             consume!(reader, u32, "typ")?.try_into()?,
                size_of_data:    consume!(reader, u32, "size_of_data")?,
                addr_raw_data:   consume!(reader, u32, "addr_raw_data")?,
                ptr_raw_data:    consume!(reader, u32, "ptr_raw_data")?,
            };

            // Currently we only handle COFF
            if matches!(dd.typ, DebugType::Coff) {
                // Parse COFF debug information
                ret.parse_coff(&mut reader, dd.ptr_raw_data as u64)?;
            }
        }

        Ok(ret)
    }

    /// Parse COFF information, used internally
    ///
    /// Updates the `self` in-place with the newly parsed information
    fn parse_coff(&mut self, reader: &mut (impl Read + Seek), coff_offset: u64)
            -> Result<()> {
        // Save current file location
        let start = reader.stream_position().map_err(Error::SeekCoff)?;

        // Seek to the COFF data header
        reader.seek(SeekFrom::Start(coff_offset)).map_err(Error::SeekCoff)?;

        // Parse COFF symbol header
        let ch = CoffSymbolsHeader {
            num_symbols:      consume!(reader, u32, "num_symbols")?,
            lva_first_symbol:
                consume!(reader, u32, "lva_first_symbol")?,
            num_line_nums:    consume!(reader, u32, "num_line_nums")?,
            lva_first_line:   consume!(reader, u32, "lva_first_line")?,
            rva_first_code:   consume!(reader, u32, "rva_first_code")?,
            rva_last_code:    consume!(reader, u32, "rva_last_code")?,
            rva_first_data:   consume!(reader, u32, "rva_first_data")?,
            rva_last_data:    consume!(reader, u32, "rva_last_data")?,
        };

        // Parse line number table
        let mut line_addrs = Vec::new();
        for _ in 0..ch.num_line_nums {
            #[derive(Debug)]
            struct Line {
                addr: u32,
                line: u16,
            }
            
            // Parse line information
            let line = Line {
                addr: consume!(reader, u32, "addr")?,
                line: consume!(reader, u16, "line")?,
            };

            line_addrs.push(line);
        }

        // Sort by address
        line_addrs.sort_by_key(|x| x.addr);

        // Storage for symbols
        let mut symbols = Vec::new();

        // Parse all symbol entries
        let mut ii = 0;
        while ii < ch.num_symbols as usize {
            /// A COFF symbol table entry
            #[derive(Debug)]
            struct Symbol {
                /// Name of the symbol, represented by union of three
                /// structures. An array of eight bytes is used if the name is
                /// not more than eight bytes long
                name:  [u8; 8],

                /// Value associated with the symbol. The interpretation of
                /// this field depends on Section Number and Storage Class. A
                /// typical meaning is the relocatable address.
                value: u32,

                /// Signed integer identifying the section, using a one-based
                /// index into the Section Table. 
                _num:  i16,

                /// A number representing type. Microsoft tools set this field
                /// to 0x20 (function) or 0x0 (not a function)
                typ:   u16,

                /// Enumerated value representing storage class.
                class: u8,

                /// Number of auxiliary symbol table entries that follow this
                /// record.
                aux:   u8,
            }

            // Parse the symbol
            let symbol = Symbol {
                name:  consume!(reader, 8,   "name")?,
                value: consume!(reader, u32, "value")?,
                _num:  consume!(reader, i16, "num")?,
                typ:   consume!(reader, u16, "typ")?,
                class: consume!(reader, u8,  "class")?,
                aux:   consume!(reader, u8,  "aux")?,
            };
          
            // Read the AUX data
            // There are 18 bytes (one `Symbol` worth) for each `aux` specified
            // This keeps the file always `Symbol` aligned, and actually makes
            // parsing fairly easy
            let mut aux = vec![0u8; symbol.aux as usize * 18];
            reader.read_exact(&mut aux).map_err(|x| {
                Error::Consume("symbol aux data", x)
            })?;

            // Advance to the next symbol
            ii += 1 + symbol.aux as usize;

            // Save the symbol
            symbols.push((symbol, aux));
        }

        // Get string table size
        let string_table_sz =
            consume!(reader, u32, "string table size")?;

        // Read the string table add 4 to leave room for the 4-byte
        // string table size
        let mut string_table =
            vec![0u8; 4 + string_table_sz as usize];
        reader.read_exact(&mut string_table[4..]).map_err(|x| {
            Error::Consume("string table", x)
        })?;

        // Storage for the most recently observed FILE class
        let mut cur_file: Option<String> = None;

        // Now that we've read everything from the file, parse the structures
        for (symbol, aux) in symbols {
            // Check if the symbol name is a pointer
            let name_is_ptr = &symbol.name[..4] == b"\0\0\0\0";
            let name = if name_is_ptr {
                // Unwrap is fine because the size is constant
                let ptr = u32::from_le_bytes(
                    symbol.name[4..].try_into().unwrap());

                // Inside unwrap is fine, `split` always returns at least one
                // iterated value
                String::from_utf8_lossy(
                    string_table.get(ptr as usize..).map(|x| {
                        x.split(|x| *x == 0).next().unwrap()
                    }).ok_or(Error::SymbolNameOob)?)
            } else {
                // Inside unwrap is fine, `split` always returns at least one
                // iterated value
                String::from_utf8_lossy(
                    symbol.name.split(|x| *x == 0).next().unwrap())
            };

            // If the class is a public symbol, private symbol, or
            // an alias (duplicate tag)
            if matches!(symbol.class, 2 | 3 | 105) {
                if symbol.typ == 0x20 {
                    self.functions.insert(symbol.value, name.to_string());
                } else {
                    self.globals.insert(symbol.value, name.to_string());
                }

                // Chcek if it's a static class with an aux, if so, we'll look
                // at the section boundaries and try to find matching source
                // lines
                if symbol.class == 3 && aux.len() >= 4 && cur_file.is_some() {
                    // Get the section length, unwrap is okay due to checked
                    // aux size.
                    let slen = u32::from_le_bytes(
                        aux[0..4].try_into().unwrap());

                    // Get start and end RVAs for this
                    let start = symbol.value; // inclusive
                    let end   = start + slen; // exclusive

                    // Search for `start` in `line_addrs`
                    let idx = match 
                        line_addrs.binary_search_by_key(&start,
                            |line| line.addr) {
                        Ok(idx)  => idx,
                        Err(idx) => idx,
                    };

                    // Go through each line from `start` until we are
                    // out of bounds of `end`
                    if let Some(line_addrs) = line_addrs.get(idx..) {
                        for line in line_addrs {
                            // Break if we're past our address
                            if line.addr >= end {
                                break;
                            }

                            // Save the line information
                            // Unwrap is fine since `cur_file` was checked
                            // to be `Some`
                            self.addr_to_line.insert(line.addr,
                                (cur_file.as_ref().unwrap().clone(),
                                 line.line as u32));
                        }
                    }
                }
            } else if matches!(symbol.class, 103) {
                // Latch the filename from AUX data, split at the null
                // terminator.
                // Unwrap is fine due to `next` always having at least one
                // return on `split`
                let filename = std::str::from_utf8(
                    aux.split(|x| *x == 0).next().unwrap())
                    .map_err(Error::FilenameUtf8)?;
                cur_file = Some(filename.to_string());
            } else {
                return Err(Error::UnknownSymbolClass(symbol.class));
            }
        }

        // Seek back to where we were
        reader.seek(SeekFrom::Start(start)).map_err(Error::SeekCoff)?;

        Ok(())
    }
}

/// Dump information about `path` to `stdout`
fn dump_info(reader: impl Read + Seek) -> Result<()> {
    // Parse the debug file
    let dbg = DbgFile::load(reader)?;

    // Print functions
    for (rva, name) in dbg.functions.iter() {
        println!("F {:08x} {}", rva, name);
    }
    
    // Print globals
    for (rva, name) in dbg.globals.iter() {
        println!("G {:08x} {}", rva, name);
    }
    
    // Print source lines
    for (rva, (source, line)) in dbg.addr_to_line.iter() {
        println!("S {:08x} {}:{}", rva, source, line);
    }

    Ok(())
}

fn main() -> Result<()> {
    // Get arguments
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("Usage: dbgparse <file1.dbg | file1.cab> ...");
        return Ok(());
    }

    for file in &args[1..] {
        // Open the file
        let fd = File::open(file).map_err(|x| {
            Error::Open(Path::new(file).to_path_buf(), x)
        })?;

        // Attempt to parse as a cabinet file
        if let Ok(mut cabinet) = cab::Cabinet::new(fd) {
            let mut cab_files = Vec::new();

            // Go through all files and folders
            for folder in cabinet.folder_entries() {
                for file in folder.file_entries() {
                    cab_files.push(file.name().to_string());
                }
            }
            
            // Extract the files and parse them
            for filename in cab_files {
                let reader = cabinet.read_file(&filename)
                    .map_err(Error::ExtractCab)?;
                dump_info(reader)?;
            }
        } else {
            // Didn't seem to be a CAB, attempt to parse as `DI`
            dump_info(BufReader::new(File::open(file).map_err(|x| {
                Error::Open(Path::new(file).to_path_buf(), x)
            })?))?;
        }
    }

    Ok(())
}


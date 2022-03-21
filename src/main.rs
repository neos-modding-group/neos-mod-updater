use std::{fs, io};
use std::convert::TryInto;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

use pelite::pe32::{Pe, PeFile};
use sha2::{Digest, Sha256, Sha512};

fn main() {
    clr_parse();
}

fn clr_parse() {
    let path = r"C:\Program Files (x86)\Steam\steamapps\common\NeosVR\Neos_Data\Managed\FrooxEngine.dll";
    let map = pelite::FileMap::open(path).unwrap();
    let pe = PeFile::from_bytes(&map).unwrap();
    let headers = pe.section_headers();

    let text_header = headers.by_name(".text").unwrap();
    let text_bytes = pe.get_section_bytes(text_header).unwrap();
    println!("headers: {:?}", headers);
    println!("len: {}", text_bytes.len());
    parse_clr_header(text_header.VirtualAddress, text_bytes);

    //let mut f = fs::File::create("text.dat").unwrap();
    //f.write_all(text_bytes).unwrap();
}

fn parse_clr_header(virtual_address: u32, text_section: &[u8]) {
    // 0x00..0x07 8 bytes: CLR loader stub and are discarded

    // 0x08..0x0C 4 bytes: the CLR header length (72). Header INCLUDES the length, so there are only 68 bytes left.
    let size = u32::from_le_bytes(text_section[0x8 .. 0xC].try_into().unwrap());
    assert_eq!(size, 72, "CLR header should be 72 bytes long");

    // 0x0C..0x0E 2 bytes: CLR major version (2)
    // 0x0E..0x10 2 bytes: CLR minor version (5)
    // 0x10..0x14 4 bytes: metadata header RVA
    let metadata_rva = u32::from_le_bytes(text_section[0x10..0x14].try_into().unwrap());
    println!("metadata rva:  {:08X}", metadata_rva);
    let metadata_offset = metadata_rva - virtual_address;
    let metadata_offset: usize = metadata_offset.try_into().unwrap();
    println!("metadata off:  {:08X}", metadata_offset);

    // 0x14..0x18 4 bytes: metadata header length
    let metadata_size = u32::from_le_bytes(text_section[0x14..0x18].try_into().unwrap());
    println!("metadata size: {:08X}", metadata_size);

    // 0x18..0x0C 4 bytes: various flags
    // 0x0C..0x20 4 bytes: token pointing to entry point of assembly
    // 0x0C..0x50 filled in according to flags, zero otherwise
    // 0x28..0x2C RVA of the strong name signature hash (SHA-1)

    // metadata time, all offsets are relative to metadata_offset now
    // 0x00..0x04 4 bytes: magic number 42534A42
    // 0x04..0x06 2 bytes: CLR metadata major version (1)
    // 0x06..0x08 2 bytes: CLR metadata minor version (1)
    // 0x08..0x0C 4 bytes: reserved
    // 0x0C..0x10 4 bytes: length of CLR version string
    let clr_version_string_size: usize = u32::from_le_bytes(text_section[(0x0C + metadata_offset)..(0x10 + metadata_offset)].try_into().unwrap()).try_into().unwrap();
    println!("clr vstring len: {:08X}", clr_version_string_size);
    let current_offset = metadata_offset + clr_version_string_size + 0x10;
    println!("offset: {:08X}", current_offset);

    // 0x10..0x10+len len bytes: ASCII of CLR version string
    // 2 bytes: reserved
    // 2 bytes: number of streams
    let stream_count = u16::from_le_bytes(text_section[(current_offset + 0x02)..(current_offset + 0x04)].try_into().unwrap());
    println!("stream count: {}", stream_count);
    let mut current_offset = current_offset + 0x04;
    const STREAM_HEADER_LENGTH: usize = 4;
    // stream headers
    for stream_index in 0..stream_count {
        // 4 byte: offset relative to metadata_offset
        // 4 byte: stream size
        // stream name, as a null-terminated string 0-padded to a multiple of 4 bytes

    }

    // #~ offset is 6C, length is very long
    // 0038DD50
    // +     6C
    // 0038DDBC

    // #~ header
    // 4 bytes: reserved
    // 1 bytes: major
    // 1 bytes: minor
    // 1 bytes: heap offset size flag: bit 1 for #Strings, bit 2 for #GUID, bit 3 for #Blob. 0 indicates 2 bytes, 1 indicates 4 bytes
    // 1 bytes: reserved, always 0x01.
    // 8 bytes: table presence bit vector
    // 8 bytes: table sorted bit vector

    // table presence: 00 00 1E 09 3F B6 DF 57
    // 0001: 45
    // 1110: 41, 42, 43
    // 0000:
    // 1001: 32, 35
    // 0011: 28, 29
    // 1111: 24, 25, 26, 27
    // 1011: 20, 21, 23
    // 0110: 17, 18
    // 1101: 12, 14, 15
    // 1111: 8, 9, 10, 11
    // 0101: 4, 6
    // 0111: 0, 1, 2

    // Assembly is 0x20 = 32, II.22.2
    // AssemblyRef is 0x23 = 35 II.22.5
    // we have BOTH
    // we'll try the Assembly table first
    // we need to read the string heap, though

    // B6 DF 57 means there are 3+2+3+4+2+3=17 tables before the one we want (CustomAttribute)
    // we're table 18
    // 00 00 1E 09 3F means there are an additional 1+3+0+2+2+3=11 tables
    // 17+1+11 = 29

    // hmm we have which tables
    // 1, 2, 3,

    // n u32s where n is the number of tables: size of each table

    let value: u8 = 5;
    value.count_ones();

    // we need to be able to read:
    // * the blob heap
    // * the custom attribute table
    // * either the MethodDef or MemberRef table
    //   * MemberRef has name
    //   * MethodDef has name

    // ECMA-335_6th_edition_june_2012.pdf is the actual spec

}


fn hash_test() {

    let path = r"C:\Program Files (x86)\Steam\steamapps\common\NeosVR\Libraries\NeosModLoader.dll";
    let mut buf = [0u8; 128];

    {
        println!("warmup 1...");
        let mut file = fs::File::open(path).unwrap();
        let mut hasher = Sha512::new();
        let now = Instant::now();
        let n = io::copy(&mut file, &mut hasher).unwrap();
        let output = hasher.finalize();
        let elapsed = now.elapsed().as_secs_f64();
        let hash = base16ct::lower::encode_str(&output, &mut buf).unwrap();
        println!("in {}s hashed {} bytes of {} to get {:?}", elapsed, n, path, hash);
    }

    {
        println!("warmup 2...");
        let mut file = fs::File::open(path).unwrap();
        let mut hasher = Sha256::new();
        let now = Instant::now();
        let n = io::copy(&mut file, &mut hasher).unwrap();
        let output = hasher.finalize();
        let elapsed = now.elapsed().as_secs_f64();
        let hash = base16ct::lower::encode_str(&output, &mut buf).unwrap();
        println!("in {}s hashed {} bytes of {} to get {:?}", elapsed, n, path, hash);
    }

    {
        println!("warmup 3...");
        let mut file = fs::File::open(path).unwrap();
        let mut hasher = Sha512::new();
        let now = Instant::now();
        let n = io::copy(&mut file, &mut hasher).unwrap();
        let output = hasher.finalize();
        let elapsed = now.elapsed().as_secs_f64();
        let hash = base16ct::lower::encode_str(&output, &mut buf).unwrap();
        println!("in {}s hashed {} bytes of {} to get {:?}", elapsed, n, path, hash);
    }

    {
        println!("sha 256 test...");
        let mut file = fs::File::open(path).unwrap();
        let mut hasher = Sha256::new();
        let now = Instant::now();
        let n = io::copy(&mut file, &mut hasher).unwrap();
        let output = hasher.finalize();
        let elapsed = now.elapsed().as_secs_f64();
        let hash = base16ct::lower::encode_str(&output, &mut buf).unwrap();
        println!("in {}s hashed {} bytes of {} to get {:?}", elapsed, n, path, hash);
    }

    {
        println!("sha 512 test...");
        let mut file = fs::File::open(path).unwrap();
        let mut hasher = Sha512::new();
        let now = Instant::now();
        let n = io::copy(&mut file, &mut hasher).unwrap();
        let output = hasher.finalize();
        let elapsed = now.elapsed().as_secs_f64();
        let hash = base16ct::lower::encode_str(&output, &mut buf).unwrap();
        println!("in {}s hashed {} bytes of {} to get {}", elapsed, n, path, hash);
    }
}

fn version_test() {
    // this works
    println!("NML: {:?}", get_version(r"C:\Program Files (x86)\Steam\steamapps\common\NeosVR\Libraries\NeosModLoader.dll"));

    // this does not work
    println!("FE:  {:?}", get_version(r"C:\Program Files (x86)\Steam\steamapps\common\NeosVR\Neos_Data\Managed\FrooxEngine.dll"));
}

fn get_version<P: AsRef<Path> + ?Sized>(path: &P) -> String {
    let map = pelite::FileMap::open(path).unwrap();
    let pe = PeFile::from_bytes(&map).unwrap();
    let res = pe.resources().unwrap();
    let v = res.version_info().unwrap();
    format!("{:?}", v)
}

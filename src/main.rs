use std::{fs, io};
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

    let mut f = fs::File::create("text.dat").unwrap();
    f.write_all(text_bytes).unwrap();
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

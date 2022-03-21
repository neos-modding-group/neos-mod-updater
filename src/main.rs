use std::path::Path;
use pelite::FileMap;
use pelite::pe32::{Pe, PeFile};

fn main() {
    // this works
    println!("NML: {:?}", get_version(r"C:\Program Files (x86)\Steam\steamapps\common\NeosVR\Libraries\NeosModLoader.dll"));

    // this does not work
    println!("FE:  {:?}", get_version(r"C:\Program Files (x86)\Steam\steamapps\common\NeosVR\Neos_Data\Managed\FrooxEngine.dll"));
}

fn get_version<P: AsRef<Path> + ?Sized>(path: &P) -> String {
    let map = FileMap::open(path).unwrap();
    let pe = PeFile::from_bytes(&map).unwrap();
    let res = pe.resources().unwrap();
    let v = res.version_info().unwrap();
    format!("{:?}", v)
}

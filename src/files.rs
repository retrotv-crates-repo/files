use std::io::Result;
use std::fs::metadata;

pub fn rm() -> Result<()> {
    let path = "C:\\Users\\yjj83\\OneDrive\\12. 표준프레임워크를 활용한 MSA 구현.pdf";
    println!("{}", is_directory(path));
    println!("{}", is_file(path));
    Ok(())
}

fn is_directory(path: &str) -> bool {
    let md = metadata(path).unwrap();
    md.is_dir()
}

fn is_file(path: &str) -> bool {
    let md = metadata(path).unwrap();
    md.is_file()
}

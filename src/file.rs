use std::fs::{metadata, Metadata};
use std::path::{Path, PathBuf};
use std::io::Result;
use sha2::{Digest, Sha256};

pub struct File {
    path: PathBuf,
}

impl File {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        File {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// 해당 경로의 메타데이터를 반환합니다.
    pub fn metadata(&self) -> Result<Metadata> {
        metadata(&self.path)
    }

    /// 해당 경로의 파일 크기를 반환합니다.
    /// metadata를 사용하므로 파일 혹은 디렉터리가 아니면 오류가 발생합니다.
    pub fn len(&self) -> Result<u64> {
        Ok(self.metadata()?.len())
    }

    /// 파일의 SHA-256 해시 값을 반환합니다.
    /// 파일이 아니거나 오류가 발생하면 빈 문자열을 반환합니다.
    pub fn hash(&self) -> String {
        if !self.is_file() {
            return String::new();
        }

        match std::fs::read(&self.path) {
            Ok(content) => {
                let mut hasher = Sha256::new();
                hasher.update(&content);
                format!("{:x}", hasher.finalize())
            }

            Err(_) => String::new(),
        }
    }

    /// 다른 파일과 해시 값을 비교하여 일치하는지 확인합니다.
    pub fn is_match(&self, other: &File) -> bool {
        if !self.is_file() || !other.is_file() {
            return false;
        }

        self.hash() == other.hash()
    }

    /// 다른 파일과 Byte 단위로 비교하여 일치하는지 확인합니다.
    pub fn is_deep_match(&self, other: &File) -> bool {
        if !self.is_file() || !other.is_file() {
            return false;
        }

        let self_content = std::fs::read(&self.path);
        let other_content = std::fs::read(&other.path);

        match (self_content, other_content) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }

    /// 경로가 파일을 가리키는지 확인합니다.
    pub fn is_file(&self) -> bool {
        self.path.is_file()
    }

    /// 경로가 디렉토리를 가리키는지 확인합니다.
    pub fn is_directory(&self) -> bool {
        self.path.is_dir()
    }

    /// 경로가 존재하는지 확인합니다.
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// 해당 경로의 파일 및 디렉터리를 삭제합니다.
    pub fn rm(&self) -> Result<()> {
        if self.is_file() {
            std::fs::remove_file(&self.path)?;
        } else if self.is_directory() {
            std::fs::remove_dir_all(&self.path)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // 테스트용 임시 디렉터리 경로를 생성하고 정리합니다.
    fn setup_test_env(test_name: &str) -> PathBuf {
        let temp_dir = std::env::temp_dir().join("files_test").join(test_name);
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }
        fs::create_dir_all(&temp_dir).unwrap();
        temp_dir
    }

    #[test]
    fn test_file_creation_and_checks() {
        let test_dir = setup_test_env("test_file_creation_and_checks");
        let file_path = test_dir.join("test_file.txt");
        let dir_path = test_dir.join("test_dir");
        fs::File::create(&file_path).unwrap();
        fs::create_dir(&dir_path).unwrap();

        let file = File::new(&file_path);
        let dir = File::new(&dir_path);
        let non_existent = File::new("non_existent_file.txt");

        // exists, is_file, is_dir 테스트
        assert!(file.exists());
        assert!(file.is_file());
        assert!(!file.is_directory());

        assert!(dir.exists());
        assert!(dir.is_directory());
        assert!(!dir.is_file());

        assert!(!non_existent.exists());
    }

    #[test]
    fn test_rm_file() {
        let test_dir = setup_test_env("test_rm_file");
        let file_path = test_dir.join("file_to_delete.txt");
        fs::File::create(&file_path).unwrap();

        let file = File::new(&file_path);
        assert!(file.exists());

        // rm 메서드로 파일 삭제
        file.rm().unwrap();
        assert!(!file.exists());
    }

    #[test]
    fn test_rm_dir() {
        let test_dir = setup_test_env("test_rm_dir");
        let dir_path = test_dir.join("dir_to_delete");
        fs::create_dir_all(&dir_path).unwrap();
        // 디렉터리 안에 파일 생성
        fs::File::create(dir_path.join("inner_file.txt")).unwrap();

        let dir = File::new(&dir_path);
        assert!(dir.exists());

        // rm 메서드로 디렉터리 재귀적으로 삭제
        dir.rm().unwrap();
        assert!(!dir.exists());
    }

    #[test]
    fn test_rm_non_existent() {
        let non_existent_file = File::new("path/that/does/not/exist.tmp");
        // 존재하지 않는 파일을 삭제 시도 시 에러가 발생하지 않고 Ok(())를 반환해야 합니다.
        // is_file()과 is_dir()가 모두 false이므로 rm()은 아무 작업도 하지 않습니다.
        assert!(non_existent_file.rm().is_ok());
    }

    #[test]
    fn test_file_len() {
        let test_dir = setup_test_env("test_file_len");
        let file_path = test_dir.join("file_with_content.txt");
        let content = b"Hello, World!";
        fs::write(&file_path, content).unwrap();

        let file = File::new(&file_path);
        assert_eq!(file.len().unwrap(), content.len() as u64);
    }

    #[test]
    fn test_is_match() {
        let test_dir = setup_test_env("test_is_match");
        let file1_path = test_dir.join("file1.txt");
        let file2_path = test_dir.join("file2.txt");
        let file3_path = test_dir.join("file3.txt");

        fs::write(&file1_path, b"Hello, World!").unwrap();
        fs::write(&file2_path, b"Hello, World!").unwrap();
        fs::write(&file3_path, b"Different content").unwrap();

        let file1 = File::new(&file1_path);
        let file2 = File::new(&file2_path);
        let file3 = File::new(&file3_path);

        assert!(file1.is_match(&file2));
        assert!(!file1.is_match(&file3));
    }

    #[test]
    fn test_is_deep_match() {
        let test_dir = setup_test_env("test_is_deep_match");
        let file1_path = test_dir.join("file1.txt");
        let file2_path = test_dir.join("file2.txt");
        let file3_path = test_dir.join("file3.txt");

        fs::write(&file1_path, b"Hello, World!").unwrap();
        fs::write(&file2_path, b"Hello, World!").unwrap();
        fs::write(&file3_path, b"Different content").unwrap();

        let file1 = File::new(&file1_path);
        let file2 = File::new(&file2_path);
        let file3 = File::new(&file3_path);

        assert!(file1.is_deep_match(&file2));
        assert!(!file1.is_deep_match(&file3));
    }
}

use std::{
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
    process::Command,
    str::FromStr,
};

pub struct TempFile {
    pub path: PathBuf,
}

impl TempFile {
    pub fn with_content(content: &str) -> Self {
        let path = PathBuf::from_str(
            std::str::from_utf8(
                &(Command::new("mktemp")
                    .output()
                    .expect("failed to call mktemp")
                    .stdout)
                    .split_last()
                    .expect("failed to split mktemp output")
                    .1,
            )
            .expect("failed to convert mktemp output to utf8"),
        )
        .expect("failed to create PathBuf");

        let mut file = File::create(&path).expect("failed to create file");
        file.write_all(content.as_bytes())
            .expect("failed to write content to file");

        TempFile { path }
    }

    pub fn reader(&mut self) -> impl BufRead {
        BufReader::new(File::open(&self.path).expect("failed to open file"))
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        fs::remove_file(&self.path).expect("failed to delete file");
    }
}

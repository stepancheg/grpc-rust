//! API to generate `.rs` files.

#![deny(intra_doc_link_resolution_failure)]
#![deny(missing_docs)]

use protoc::Protoc;
use protoc_rust::Customize;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Error type alias.
pub type Error = io::Error;
/// Result type alias.
pub type Result<T> = io::Result<T>;

/// Utility to generate `.rs` files.
#[derive(Debug, Default)]
pub struct Codegen {
    protoc: Option<Protoc>,
    /// --lang_out= param
    out_dir: PathBuf,
    /// -I args
    includes: Vec<PathBuf>,
    /// List of .proto files to compile
    inputs: Vec<PathBuf>,
    /// Generate rust-protobuf files along with rust-gprc
    rust_protobuf: bool,
    /// Customize rust-protobuf codegen
    rust_protobuf_customize: protoc_rust::Customize,
}

impl Codegen {
    /// Create new codegen object.
    pub fn new() -> Codegen {
        Default::default()
    }

    /// Set `--LANG_out=...` param
    pub fn out_dir(&mut self, out_dir: impl AsRef<Path>) -> &mut Self {
        self.out_dir = out_dir.as_ref().to_owned();
        self
    }

    /// Append a path to `-I` args
    pub fn include(&mut self, include: impl AsRef<Path>) -> &mut Self {
        self.includes.push(include.as_ref().to_owned());
        self
    }

    /// Append multiple paths to `-I` args
    pub fn includes(&mut self, includes: impl IntoIterator<Item = impl AsRef<Path>>) -> &mut Self {
        for include in includes {
            self.include(include);
        }
        self
    }

    /// Append a `.proto` file path to compile
    pub fn input(&mut self, input: impl AsRef<Path>) -> &mut Self {
        self.inputs.push(input.as_ref().to_owned());
        self
    }

    /// Append multiple `.proto` file paths to compile
    pub fn inputs(&mut self, inputs: impl IntoIterator<Item = impl AsRef<Path>>) -> &mut Self {
        for input in inputs {
            self.input(input);
        }
        self
    }

    /// Generate rust-protobuf files along with rust-gprc
    pub fn rust_protobuf(&mut self, rust_protobuf: bool) -> &mut Self {
        self.rust_protobuf = rust_protobuf;
        self
    }

    /// Generate rust-protobuf files along with rust-gprc
    pub fn rust_protobuf_customize(&mut self, rust_protobuf_customize: Customize) -> &mut Self {
        self.rust_protobuf_customize = rust_protobuf_customize;
        self
    }

    /// Run the codegen.
    ///
    /// Generate `_grpc.rs` files, and if [`rust_protobuf_customize`](Codegen::rust_protobuf_customize)
    /// is specified, generate rust-protobuf `.rs` files too.
    ///
    pub fn run(&self) -> Result<()> {
        let protoc = self
            .protoc
            .clone()
            .unwrap_or_else(|| protoc::Protoc::from_env_path());
        let version = protoc.version().expect("protoc version");
        if !version.is_3() {
            panic!("protobuf must have version 3");
        }

        if self.rust_protobuf {
            protoc_rust::Codegen::new()
                .out_dir(&self.out_dir)
                .includes(&self.includes)
                .inputs(&self.inputs)
                .customize(self.rust_protobuf_customize.clone())
                .run()?;
        }

        let temp_dir = tempdir::TempDir::new("protoc-rust")?;
        let temp_file = temp_dir.path().join("descriptor.pbbin");
        let temp_file = temp_file.to_str().expect("utf-8 file name");

        let includes: Vec<&str> = self
            .includes
            .iter()
            .map(|p| p.as_os_str().to_str().unwrap())
            .collect();
        let inputs: Vec<&str> = self
            .inputs
            .iter()
            .map(|p| p.as_os_str().to_str().unwrap())
            .collect();
        protoc.write_descriptor_set(protoc::DescriptorSetOutArgs {
            out: temp_file,
            includes: &includes,
            input: &inputs,
            include_imports: true,
        })?;

        let mut fds = Vec::new();
        let mut file = fs::File::open(temp_file)?;
        file.read_to_end(&mut fds)?;

        drop(file);
        drop(temp_dir);

        let fds: protobuf::descriptor::FileDescriptorSet = protobuf::parse_from_bytes(&fds)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut includes = self.includes.clone();
        if includes.is_empty() {
            includes = vec![PathBuf::from(".")];
        }

        let mut files_to_generate = Vec::new();
        'outer: for file in &self.inputs {
            for include in &includes {
                if let Some(truncated) =
                    remove_path_prefix(file.to_str().unwrap(), include.to_str().unwrap())
                {
                    files_to_generate.push(truncated.to_owned());
                    continue 'outer;
                }
            }

            return Err(Error::new(
                io::ErrorKind::Other,
                format!(
                    "file {:?} is not found in includes {:?}",
                    file, self.includes
                ),
            ));
        }

        let gen_result = grpc_compiler::codegen::gen(fds.get_file(), &files_to_generate);

        for r in gen_result {
            let r: protobuf::compiler_plugin::GenResult = r;
            let file = format!("{}/{}", self.out_dir.display(), r.name);
            let mut file = fs::File::create(&file)?;
            file.write_all(&r.content)?;
            file.flush()?;
        }

        Ok(())
    }
}

fn remove_dot_slash(path: &str) -> &str {
    if path == "." {
        ""
    } else if path.starts_with("./") || path.starts_with(".\\") {
        &path[2..]
    } else {
        path
    }
}

fn remove_path_prefix<'a>(mut path: &'a str, mut prefix: &str) -> Option<&'a str> {
    path = remove_dot_slash(path);
    prefix = remove_dot_slash(prefix);

    if prefix == "" {
        return Some(path);
    }

    if prefix.ends_with("/") || prefix.ends_with("\\") {
        prefix = &prefix[..prefix.len() - 1];
    }

    if !path.starts_with(prefix) {
        return None;
    }

    if path.len() <= prefix.len() {
        return None;
    }

    if path.as_bytes()[prefix.len()] == b'/' || path.as_bytes()[prefix.len()] == b'\\' {
        return Some(&path[prefix.len() + 1..]);
    } else {
        return None;
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn remove_path_prefix() {
        assert_eq!(
            Some("abc.proto"),
            super::remove_path_prefix("xxx/abc.proto", "xxx")
        );
        assert_eq!(
            Some("abc.proto"),
            super::remove_path_prefix("xxx/abc.proto", "xxx/")
        );
        assert_eq!(
            Some("abc.proto"),
            super::remove_path_prefix("../xxx/abc.proto", "../xxx/")
        );
        assert_eq!(
            Some("abc.proto"),
            super::remove_path_prefix("abc.proto", ".")
        );
        assert_eq!(
            Some("abc.proto"),
            super::remove_path_prefix("abc.proto", "./")
        );
        assert_eq!(None, super::remove_path_prefix("xxx/abc.proto", "yyy"));
        assert_eq!(None, super::remove_path_prefix("xxx/abc.proto", "yyy/"));
    }
}

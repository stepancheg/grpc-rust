extern crate protobuf;

use protobuf::compiler_plugin;
use protobuf::descriptor::*;

pub fn gen(file_descriptors: &[FileDescriptorProto], files_to_generate: &[String])
        -> Vec<compiler_plugin::GenResult>
{
    Vec::new()
}

fn main() {
    compiler_plugin::plugin_main(gen);
}

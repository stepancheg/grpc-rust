use std::collections::HashMap;

use protobuf;
use protobuf::compiler_plugin;
use protobuf::descriptor::*;

pub fn gen(file_descriptors: &[FileDescriptorProto], files_to_generate: &[String])
        -> Vec<compiler_plugin::GenResult>
{
    let files_map: HashMap<&str, &FileDescriptorProto> =
        file_descriptors.iter().map(|f| (f.get_name(), f)).collect();

    let mut results = Vec::new();

    for file_name in files_to_generate {
        let file = files_map[&file_name[..]];
        let base = protobuf::proto_path_to_rust_mod(file.get_name());

        if file.get_service().is_empty() {
            continue;
        }

        results.push(compiler_plugin::GenResult {
            name: base + "_grpc.rs",
            content: "// XXX\n".to_string().into_bytes(),
        });
    }

    results
}

pub fn protoc_gen_grpc_rust_main() {
    compiler_plugin::plugin_main(gen);
}

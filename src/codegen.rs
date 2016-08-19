use std::collections::HashMap;

use protobuf;
use protobuf::compiler_plugin;
use protobuf::code_writer::CodeWriter;
use protobuf::descriptor::*;
use protobuf::descriptorx::*;

struct MethodGen<'a> {
    proto: &'a MethodDescriptorProto,
    root_scope: &'a RootScope<'a>,
}

impl<'a> MethodGen<'a> {
    fn new(proto: &'a MethodDescriptorProto, root_scope: &'a RootScope<'a>) -> MethodGen<'a> {
        MethodGen {
            proto: proto,
            root_scope: root_scope,
        }
    }

    fn sig(&self) -> String {
        let input = self.root_scope.find_message(self.proto.get_input_type()).rust_fq_name();
        let output = self.root_scope.find_message(self.proto.get_output_type()).rust_fq_name();

        format!("{}(&self, p: super::{}) -> super::{}", self.proto.get_name(), input, output)
    }

    fn write_intf(&self, w: &mut CodeWriter) {
        w.fn_def(&self.sig())
    }

    fn write_client(&self, w: &mut CodeWriter) {
        let input = self.root_scope.find_message(self.proto.get_input_type()).rust_fq_name();
        let output = self.root_scope.find_message(self.proto.get_output_type()).rust_fq_name();

        w.def_fn(&self.sig(), |w| {
            w.todo("not yet");
        });
    }
}

struct ServiceGen<'a> {
    proto: &'a ServiceDescriptorProto,
    root_scope: &'a RootScope<'a>,
    methods: Vec<MethodGen<'a>>,
    package: String,
}

impl<'a> ServiceGen<'a> {
    fn new(proto: &'a ServiceDescriptorProto, file: &FileDescriptorProto, root_scope: &'a RootScope) -> ServiceGen<'a> {
        ServiceGen {
            proto: proto,
            root_scope: root_scope,
            methods: proto.get_method().into_iter().map(|m| MethodGen::new(m, root_scope)).collect(),
            package: file.get_package().to_string(),
        }
    }

    fn intf_name(&self) -> &str {
        self.proto.get_name()
    }

    fn client_name(&self) -> String {
        format!("{}Client", self.proto.get_name())
    }

    fn server_name(&self) -> String {
        format!("{}Server", self.proto.get_name())
    }

    fn write_intf(&self, w: &mut CodeWriter) {
        w.pub_trait(&self.intf_name(), |w| {
            for (i, method) in self.methods.iter().enumerate() {
                if i != 0 {
                    w.write_line("");
                }

                method.write_intf(w);
            }
        });
    }

    fn write_client(&self, w: &mut CodeWriter) {
        w.pub_struct(&self.client_name(), |w| {
        });

        w.write_line("");

        w.impl_for_block(self.intf_name(), &self.client_name(), |w| {
            for (i, method) in self.methods.iter().enumerate() {
                if i != 0 {
                    w.write_line("");
                }

                method.write_client(w);
            }
        });
    }

    fn service_path(&self) -> String {
        format!("/{}.{}", self.package, self.proto.get_name())
    }

    fn write_server(&self, w: &mut CodeWriter) {
        w.pub_struct(&self.server_name(), |w| {
            w.field_decl("server", "::grpc::server::GrpcServer");
        });

        w.write_line("");

        w.impl_self_block(&self.server_name(), |w| {
            w.pub_fn(format!("new<H : {} + 'static + Sync + Send>(h: H) -> {}", self.intf_name(), self.server_name()), |w| {
                w.write_line("let handler_arc = ::std::sync::Arc::new(h);");
                w.block("let service_definition = ::std::sync::Arc::new(::grpc::method::ServerServiceDefinition::new(", "));", |w| {
                    w.block("vec![", "],", |w| {
                        for method in &self.methods {
                            w.block("::grpc::method::ServerMethod::new(", "),", |w| {
                                w.block("::grpc::method::MethodDescriptor {", "},", |w| {
                                    w.field_entry("name", format!("\"{}/{}\".to_string()", self.service_path(), method.proto.get_name()));
                                    w.field_entry("input_streaming", "false"); // TODO
                                    w.field_entry("output_streaming", "false"); // TODO
                                    w.field_entry("req_marshaller", "Box::new(::grpc::grpc_protobuf::MarshallerProtobuf)");
                                    w.field_entry("resp_marshaller", "Box::new(::grpc::grpc_protobuf::MarshallerProtobuf)");
                                });
                                w.block("{", "},", |w| {
                                    w.write_line("let handler_copy = handler_arc.clone();");
                                    w.write_line(format!("::grpc::method::MethodHandlerFn::new(move |p| handler_copy.{}(p))", method.proto.get_name()));
                                });
                            });
                        }
                    });
                });

                w.expr_block(self.server_name(), |w| {
                    w.field_entry("server", "::grpc::server::GrpcServer::new(service_definition)");
                });
            });

            w.write_line("");

            w.pub_fn("run(&mut self)", |w| {
                w.write_line("self.server.run()")
            });
        });
    }

    fn write(&self, w: &mut CodeWriter) {
        self.write_intf(w);
        w.write_line("");
        self.write_client(w);
        w.write_line("");
        self.write_server(w);
    }
}

fn gen_file(
    file: &FileDescriptorProto,
    root_scope: &RootScope,
) -> Option<compiler_plugin::GenResult>
{
    if file.get_service().is_empty() {
        return None;
    }

    let base = protobuf::descriptorx::proto_path_to_rust_mod(file.get_name());

    let mut v = Vec::new();
    {
        let mut w = CodeWriter::new(&mut v);
        w.write_generated();
        w.write_line("");

        for service in file.get_service() {
            w.write_line("");
            ServiceGen::new(service, file, root_scope).write(&mut w);
        }
    }

    Some(compiler_plugin::GenResult {
        name: base + "_grpc.rs",
        content: v,
    })
}

pub fn gen(file_descriptors: &[FileDescriptorProto], files_to_generate: &[String])
        -> Vec<compiler_plugin::GenResult>
{
    let files_map: HashMap<&str, &FileDescriptorProto> =
        file_descriptors.iter().map(|f| (f.get_name(), f)).collect();

    let root_scope = RootScope { file_descriptors: file_descriptors };

    let mut results = Vec::new();

    for file_name in files_to_generate {
        let file = files_map[&file_name[..]];

        if file.get_service().is_empty() {
            continue;
        }

        results.extend(gen_file(file, &root_scope).into_iter());
    }

    results
}

pub fn protoc_gen_grpc_rust_main() {
    compiler_plugin::plugin_main(gen);
}

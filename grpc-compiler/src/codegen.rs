use std::collections::HashMap;

use protobuf;
use protobuf::compiler_plugin;
use protobuf::code_writer::CodeWriter;
use protobuf::descriptor::*;
use protobuf::descriptorx::*;


/// Adjust method name to follow the rust's style.
fn snake_name(name: &str) -> String {
    let mut snake_method_name = String::with_capacity(name.len());
    let mut chars = name.chars();
    // initial char can be any char except '_'.
    let mut last_char = '.';
    'outer:
    while let Some(c) = chars.next() {
        // Please note that '_' is neither uppercase nor lowercase.
        if !c.is_uppercase() {
            last_char = c;
            snake_method_name.push(c);
            continue;
        }
        let mut can_append_underscore = false;
        // check if it's the first char.
        if !snake_method_name.is_empty() && last_char != '_' {
            snake_method_name.push('_');
        }
        last_char = c;
        // find all continous upper case char and append an underscore before
        // last upper case char if neccessary.
        while let Some(c) = chars.next() {
            if !c.is_uppercase() {
                if can_append_underscore && c != '_' {
                    snake_method_name.push('_');
                }
                snake_method_name.extend(last_char.to_lowercase());
                snake_method_name.push(c);
                last_char = c;
                continue 'outer;
            }
            snake_method_name.extend(last_char.to_lowercase());
            last_char = c;
            can_append_underscore = true;
        }
        snake_method_name.extend(last_char.to_lowercase());
    }
    snake_method_name
}

struct MethodGen<'a> {
    proto: &'a MethodDescriptorProto,
    service_path: String,
    root_scope: &'a RootScope<'a>,
}

impl<'a> MethodGen<'a> {
    fn new(proto: &'a MethodDescriptorProto, service_path: String, root_scope: &'a RootScope<'a>) -> MethodGen<'a> {
        MethodGen {
            proto: proto,
            service_path: service_path,
            root_scope: root_scope,
        }
    }

    fn snake_name(&self) -> String {
        snake_name(self.proto.get_name())
    }

    fn input_message(&self) -> String {
        format!("super::{}", self.root_scope.find_message(self.proto.get_input_type()).rust_fq_name())
    }

    fn output_message(&self) -> String {
        format!("super::{}", self.root_scope.find_message(self.proto.get_output_type()).rust_fq_name())
    }

    fn input(&self) -> String {
        match self.proto.get_client_streaming() {
            false => self.input_message(),
            true  => format!("::grpc::StreamingRequest<{}>", self.input_message()),
        }
    }

    fn output(&self) -> String {
        match self.proto.get_server_streaming() {
            false => format!("::grpc::SingleResponse<{}>", self.output_message()),
            true  => format!("::grpc::StreamingResponse<{}>", self.output_message()),
        }
    }

    fn sig(&self) -> String {
        format!("{}(&self, o: ::grpc::RequestOptions, p: {}) -> {}",
                self.snake_name(), self.input(), self.output())
    }

    fn write_intf(&self, w: &mut CodeWriter) {
        w.fn_def(&self.sig())
    }

    fn descriptor_field_name(&self) -> String {
        format!("method_{}", self.proto.get_name())
    }

    fn descriptor_type(&self) -> String {
        format!("::grpc::method::MethodDescriptor<{}, {}>", self.input_message(), self.output_message())
    }

    fn streaming_upper(&self) -> &'static str {
        match (self.proto.get_client_streaming(), self.proto.get_server_streaming()) {
            (false, false) => "Unary",
            (false, true) => "ServerStreaming",
            (true, false) => "ClientStreaming",
            (true, true) => "Bidi",
        }
    }

    fn streaming_lower(&self) -> &'static str {
        match (self.proto.get_client_streaming(), self.proto.get_server_streaming()) {
            (false, false) => "unary",
            (false, true) => "server_streaming",
            (true, false) => "client_streaming",
            (true, true) => "bidi",
        }
    }

    fn write_client(&self, w: &mut CodeWriter) {
        w.def_fn(&self.sig(), |w| {
            w.write_line(&format!("self.grpc_client.call_{}(o, p, self.{}.clone())",
                self.streaming_lower(),
                self.descriptor_field_name()))
        });
    }

    fn write_descriptor(&self, w: &mut CodeWriter, before: &str, after: &str) {
        w.block(&format!("{}{}", before, "::grpc::method::MethodDescriptor {"), &format!("{}{}", "}", after), |w| {
            w.field_entry("name", &format!("\"{}/{}\".to_string()", self.service_path, self.proto.get_name()));
            w.field_entry("streaming", &format!("::grpc::method::GrpcStreaming::{}", self.streaming_upper()));
            w.field_entry("req_marshaller", "Box::new(::grpc::protobuf::MarshallerProtobuf)");
            w.field_entry("resp_marshaller", "Box::new(::grpc::protobuf::MarshallerProtobuf)");
        });
    }
}

struct ServiceGen<'a> {
    proto: &'a ServiceDescriptorProto,
    _root_scope: &'a RootScope<'a>,
    methods: Vec<MethodGen<'a>>,
    _service_path: String,
    _package: String,
}

impl<'a> ServiceGen<'a> {
    fn new(proto: &'a ServiceDescriptorProto, file: &FileDescriptorProto, root_scope: &'a RootScope) -> ServiceGen<'a> {
        let service_path =
            if file.get_package().is_empty() {
                format!("/{}", proto.get_name())
            } else {
                format!("/{}.{}", file.get_package(), proto.get_name())
            };
        let methods = proto.get_method().into_iter()
            .map(|m| MethodGen::new(m, service_path.clone(), root_scope))
            .collect();

        ServiceGen {
            proto: proto,
            _root_scope: root_scope,
            methods: methods,
            _service_path: service_path,
            _package: file.get_package().to_string(),
        }
    }

    // trait name
    fn intf_name(&self) -> &str {
        self.proto.get_name()
    }

    // client struct name
    fn client_name(&self) -> String {
        format!("{}Client", self.intf_name())
    }

    // server struct name
    fn server_name(&self) -> String {
        format!("{}Server", self.intf_name())
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

    fn write_client_object(&self, grpc_client: &str, w: &mut CodeWriter) {
        w.expr_block(&self.client_name(), |w| {
            w.field_entry("grpc_client", grpc_client);
            for method in &self.methods {
                method.write_descriptor(
                    w,
                    &format!("{}: ::std::sync::Arc::new(", method.descriptor_field_name()),
                    "),");
            }
        });
    }

    fn write_client(&self, w: &mut CodeWriter) {
        w.pub_struct(&self.client_name(), |w| {
            w.field_decl("grpc_client", "::grpc::Client");
            for method in &self.methods {
                w.field_decl(
                    &method.descriptor_field_name(),
                    &format!("::std::sync::Arc<{}>", method.descriptor_type()));
            }
        });

        w.write_line("");

        w.impl_self_block(&self.client_name(), |w| {

            let sig = "with_client(grpc_client: ::grpc::Client) -> Self";
            w.pub_fn(sig, |w| {
                self.write_client_object("grpc_client", w);
            });

            w.write_line("");

            let sig = "new_plain(host: &str, port: u16, conf: ::grpc::ClientConf) -> ::grpc::Result<Self>";
            w.pub_fn(sig, |w| {
                w.write_line("::grpc::Client::new_plain(host, port, conf).map(|c| {");
                w.indented(|w| {
                    w.write_line(&format!("{}::with_client(c)", self.client_name()));
                });
                w.write_line("})");
            });
            
            let sig = "new_tls<C : ::tls_api::TlsConnector>(host: &str, port: u16, conf: ::grpc::ClientConf) -> ::grpc::Result<Self>";
            w.pub_fn(sig, |w| {
                w.write_line("::grpc::Client::new_tls::<C>(host, port, conf).map(|c| {");
                w.indented(|w| {
                    w.write_line(&format!("{}::with_client(c)", self.client_name()));
                });
                w.write_line("})");
            });

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

    fn write_service_definition(&self, before: &str, after: &str, handler: &str, w: &mut CodeWriter) {
        w.block(
            &format!("{}::grpc::server::ServerServiceDefinition::new(", before),
            &format!("){}", after),
            |w| {
                w.block("vec![", "],", |w| {
                    for method in &self.methods {
                        w.block("::grpc::server::ServerMethod::new(", "),", |w| {
                            method.write_descriptor(w, "::std::sync::Arc::new(", "),");
                            w.block("{", "},", |w| {
                                w.write_line(&format!("let handler_copy = {}.clone();", handler));
                                w.write_line(&format!("::grpc::server::MethodHandler{}::new(move |o, p| handler_copy.{}(o, p))",
                                    method.streaming_upper(),
                                    method.snake_name()));
                            });
                        });
                    }
                });
            });
    }

    fn write_server(&self, w: &mut CodeWriter) {
        w.pub_struct(&self.server_name(), |w| {
            w.pub_field_decl("grpc_server", "::grpc::Server");
        });

        w.write_line("");

        w.impl_for_block("::std::ops::Deref", &self.server_name(), |w| {
            w.write_line("type Target = ::grpc::Server;");
            w.write_line("");
            w.def_fn("deref(&self) -> &Self::Target", |w| {
                w.write_line("&self.grpc_server");
            })
        });

        w.write_line("");

        w.impl_self_block(&self.server_name(), |w| {
            let sig = format!(
                "new<A : ::std::net::ToSocketAddrs, H : {} + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::ServerConf, h: H) -> ::grpc::Result<Self>",
                self.intf_name());
            w.pub_fn(&sig, |w| {
                w.write_line(format!("let service_definition = {}::new_service_def(h);", self.server_name()));

                w.block(&format!("Ok({} {{", self.server_name()), "})", |w| {
                    w.field_entry("grpc_server", "::grpc::Server::new_plain(addr, conf, service_definition)?");
                });
            });

            w.write_line("");

            let sig = format!(
                "new_pool<A : ::std::net::ToSocketAddrs, H : {} + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::ServerConf, h: H, cpu_pool: ::futures_cpupool::CpuPool) -> ::grpc::Result<Self>",
                self.intf_name());
            w.pub_fn(&sig, |w| {
                w.write_line(format!("let service_definition = {}::new_service_def(h);", self.server_name()));

                w.block(&format!("Ok({} {{", self.server_name()), "})", |w| {
                    w.field_entry("grpc_server", "::grpc::Server::new_plain_pool(addr, conf, service_definition, cpu_pool)?");
                });
            });

            w.write_line("");

            w.pub_fn(&format!("new_service_def<H : {} + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::server::ServerServiceDefinition", self.intf_name()), |w| {
                w.write_line("let handler_arc = ::std::sync::Arc::new(handler);");

                self.write_service_definition("", "", "handler_arc", w);
            });
        });
    }

    fn write(&self, w: &mut CodeWriter) {
        w.comment("interface");
        w.write_line("");
        self.write_intf(w);
        w.write_line("");
        w.comment("client");
        w.write_line("");
        self.write_client(w);
        w.write_line("");
        w.comment("server");
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

#[cfg(test)]
mod test {
    #[test]
    fn test_snake_name() {
        let cases = vec![
            ("AsyncRequest", "async_request"),
            ("asyncRequest", "async_request"),
            ("async_request", "async_request"),
            ("createID", "create_id"),
            ("CreateIDForReq", "create_id_for_req"),
            ("Create_ID_For_Req", "create_id_for_req"),
            ("ID", "id"),
            ("id", "id"),
        ];

        for (origin, exp) in cases {
            let res = super::snake_name(&origin);
            assert_eq!(res, exp);
        }
    }
}

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

    fn input(&self) -> String {
        format!("super::{}", self.root_scope.find_message(self.proto.get_input_type()).rust_fq_name())
    }

    fn output(&self) -> String {
        format!("super::{}", self.root_scope.find_message(self.proto.get_output_type()).rust_fq_name())
    }

    fn input_async(&self) -> String {
        match self.proto.get_client_streaming() {
            false => self.input(),
            true  => format!("::grpc::futures_grpc::GrpcStreamSend<{}>", self.input()),
        }
    }

    fn output_async(&self) -> String {
        match self.proto.get_server_streaming() {
            false => format!("::grpc::futures_grpc::GrpcFutureSend<{}>", self.output()),
            true  => format!("::grpc::futures_grpc::GrpcStreamSend<{}>", self.output()),
        }
    }

    fn input_sync(&self) -> String {
        match self.proto.get_client_streaming() {
            false => self.input(),
            true  => format!("::grpc::iter::GrpcIterator<{}>", self.input()),
        }
    }

    fn output_sync(&self) -> String {
        match self.proto.get_server_streaming() {
            false => format!("::grpc::result::GrpcResult<{}>", self.output()),
            true  => format!("::grpc::iter::GrpcIterator<{}>", self.output()),
        }
    }

    fn sync_sig(&self, method_name: &str) -> String {
        format!("{}(&self, p: {}) -> {}",
            method_name, self.input_sync(), self.output_sync())
    }

    fn async_sig(&self, method_name: &str) -> String {
        format!("{}(&self, p: {}) -> {}",
                method_name, self.input_async(), self.output_async())
    }

    fn write_sync_intf(&self, w: &mut CodeWriter, method_name: &str) {
        w.fn_def(&self.sync_sig(method_name))
    }

    fn write_async_intf(&self, w: &mut CodeWriter, method_name: &str) {
        w.fn_def(&self.async_sig(method_name))
    }

    fn write_sync_client(&self, w: &mut CodeWriter, method_name: &str) {
        w.def_fn(&self.sync_sig(method_name), |w| {
            let wait = match self.proto.get_server_streaming() {
                false => "::futures::Future::wait",
                true  => "::grpc::rt::stream_to_iter",
            };
            if self.proto.get_client_streaming() {
                w.write_line("let p = ::futures::stream::Stream::boxed(::futures::stream::iter(::std::iter::IntoIterator::into_iter(p)));");
            }
            w.write_line(&format!("{}(self.async_client.{}(p))", wait, method_name));
        });
    }

    fn descriptor_field_name(&self) -> String {
        format!("method_{}", self.proto.get_name())
    }

    fn descriptor_type(&self) -> String {
        format!("::grpc::method::MethodDescriptor<{}, {}>", self.input(), self.output())
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

    fn write_async_client(&self, w: &mut CodeWriter, method_name: &str) {
        w.def_fn(&self.async_sig(method_name), |w| {
            w.write_line(&format!("self.grpc_client.call_{}(p, self.{}.clone())",
                self.streaming_lower(),
                self.descriptor_field_name()))
        });
    }

    fn write_descriptor(&self, w: &mut CodeWriter, before: &str, after: &str) {
        w.block(&format!("{}{}", before, "::grpc::method::MethodDescriptor {"), &format!("{}{}", "}", after), |w| {
            w.field_entry("name", &format!("\"{}/{}\".to_string()", self.service_path, self.proto.get_name()));
            w.field_entry("streaming", &format!("::grpc::method::GrpcStreaming::{}", self.streaming_upper()));
            w.field_entry("req_marshaller", "Box::new(::grpc::grpc_protobuf::MarshallerProtobuf)");
            w.field_entry("resp_marshaller", "Box::new(::grpc::grpc_protobuf::MarshallerProtobuf)");
        });
    }

    fn write_server_sync_to_async_delegate(&self, w: &mut CodeWriter, method_name: &str) {
        w.def_fn(&self.async_sig(method_name), |w| {
            w.write_line("let h = self.handler.clone();");
            w.write_line(format!("::grpc::rt::sync_to_async_{}(&self.cpupool, p, move |p| {{", self.streaming_lower()));
            w.write_line(format!("    h.{}(p)", method_name));
            w.write_line(format!("}})"));
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

    // name of synchronous interface
    fn sync_intf_name(&self) -> &str {
        self.proto.get_name()
    }

    // name of asynchronous interface
    fn async_intf_name(&self) -> String {
        format!("{}Async", self.sync_intf_name())
    }

    fn sync_client_name(&self) -> String {
        format!("{}Client", self.sync_intf_name())
    }

    fn sync_server_name(&self) -> String {
        format!("{}Server", self.sync_intf_name())
    }

    fn sync_handler_to_async_name(&self) -> String {
        format!("{}HandlerToAsync", self.sync_server_name())
    }

    fn async_client_name(&self) -> String {
        format!("{}Client", self.async_intf_name())
    }

    fn async_server_name(&self) -> String {
        format!("{}Server", self.async_intf_name())
    }

    fn write_sync_intf(&self, w: &mut CodeWriter) {
        w.pub_trait(&self.sync_intf_name(), |w| {
            for (i, method) in self.methods.iter().enumerate() {
                if i != 0 {
                    w.write_line("");
                }

                let snake_method_name = snake_name(method.proto.get_name());
                method.write_sync_intf(w, &snake_method_name);
            }
        });
    }

    fn write_async_intf(&self, w: &mut CodeWriter) {
        w.pub_trait(&self.async_intf_name(), |w| {
            for (i, method) in self.methods.iter().enumerate() {
                if i != 0 {
                    w.write_line("");
                }

                let snake_method_name = snake_name(method.proto.get_name());
                method.write_async_intf(w, &snake_method_name);
            }
        });
    }

    fn write_sync_client(&self, w: &mut CodeWriter) {
        w.pub_struct(&self.sync_client_name(), |w| {
            w.field_decl("async_client", &self.async_client_name());
        });

        w.write_line("");

        w.impl_self_block(&self.sync_client_name(), |w| {
            w.pub_fn("new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self>", |w| {
                w.write_line(format!("{}::new(host, port, tls, conf).map(|c| {{", &self.async_client_name()));
                w.indented(|w| {
                    w.expr_block(&self.sync_client_name(), |w| {
                        w.field_entry("async_client", "c");
                    });
                });
                w.write_line("})");
            });
        });

        w.write_line("");

        w.impl_for_block(self.sync_intf_name(), &self.sync_client_name(), |w| {
            for (i, method) in self.methods.iter().enumerate() {
                if i != 0 {
                    w.write_line("");
                }

                let snake_method_name = snake_name(method.proto.get_name());
                method.write_sync_client(w, &snake_method_name);
            }
        });
    }

    fn write_async_client_object(&self, grpc_client: &str, w: &mut CodeWriter) {
        w.expr_block(&self.async_client_name(), |w| {
            w.field_entry("grpc_client", grpc_client);
            for method in &self.methods {
                method.write_descriptor(
                    w,
                    &format!("{}: ::std::sync::Arc::new(", method.descriptor_field_name()),
                    "),");
            }
        });
    }

    fn write_async_client(&self, w: &mut CodeWriter) {
        w.pub_struct(&self.async_client_name(), |w| {
            w.field_decl("grpc_client", "::grpc::client::GrpcClient");
            for method in &self.methods {
                w.field_decl(
                    &method.descriptor_field_name(),
                    &format!("::std::sync::Arc<{}>", method.descriptor_type()));
            }
        });

        w.write_line("");

        w.impl_self_block(&self.async_client_name(), |w| {

            let sig = "with_client(grpc_client: ::grpc::client::GrpcClient) -> Self";
            w.pub_fn(sig, |w| {
                self.write_async_client_object("grpc_client", w);
            });

            w.write_line("");

            let sig = "new(host: &str, port: u16, tls: bool, conf: ::grpc::client::GrpcClientConf) -> ::grpc::result::GrpcResult<Self>";
            w.pub_fn(sig, |w| {
                w.write_line("::grpc::client::GrpcClient::new(host, port, tls, conf).map(|c| {");
                w.indented(|w| {
                    w.write_line(&format!("{}::with_client(c)", self.async_client_name()));
                });
                w.write_line("})");
            });
            
        });

        w.write_line("");

        w.impl_for_block(self.async_intf_name(), &self.async_client_name(), |w| {
            for (i, method) in self.methods.iter().enumerate() {
                if i != 0 {
                    w.write_line("");
                }

                let snake_method_name = snake_name(method.proto.get_name());
                method.write_async_client(w, &snake_method_name);
            }
        });
    }

    fn write_sync_server(&self, w: &mut CodeWriter) {
        w.pub_struct(&self.sync_server_name(), |w| {
            w.field_decl("async_server", &self.async_server_name());
        });

        w.write_line("");

        w.impl_for_block("::std::ops::Deref", &self.sync_server_name(), |w| {
            w.write_line(&format!("type Target = {};", &self.async_server_name()));
            w.write_line("");
            w.def_fn("deref(&self) -> &Self::Target", |w| {
                w.write_line("&self.async_server");
            })
        });

        w.write_line("");

        w.def_struct(&self.sync_handler_to_async_name(), |w| {
            w.field_decl("handler", &format!("::std::sync::Arc<{} + Send + Sync>", self.sync_intf_name()));
            w.field_decl("cpupool", "::futures_cpupool::CpuPool");
        });

        w.write_line("");

        w.impl_for_block(&self.async_intf_name(), &self.sync_handler_to_async_name(), |w| {
            for (i, method) in self.methods.iter().enumerate() {
                if i != 0 {
                    w.write_line("");
                }

                let snake_method_name = snake_name(method.proto.get_name());
                method.write_server_sync_to_async_delegate(w, &snake_method_name);
            }
        });

        w.write_line("");

        w.impl_self_block(&self.sync_server_name(), |w| {
            let sig = &format!(
                "new_plain<A : ::std::net::ToSocketAddrs, H : {} + Send + Sync + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self",
                self.sync_intf_name());
            w.pub_fn(sig, |w| {
                w.stmt_block(&format!("let h = {}", self.sync_handler_to_async_name()), |w| {
                    w.field_entry("cpupool", "::futures_cpupool::CpuPool::new_num_cpus()");
                    w.field_entry("handler", "::std::sync::Arc::new(h)");
                });

                w.expr_block(&self.sync_server_name(), |w| {
                    w.field_entry("async_server", &format!("{}::new(addr, conf, h)", self.async_server_name()));
                });
            });
        });
    }

    fn write_service_definition(&self, before: &str, after: &str, handler: &str, w: &mut CodeWriter) {
        w.block(
            &format!("{}::grpc::server::ServerServiceDefinition::new(", before),
            &format!("){}", after),
            |w| {
                w.block("vec![", "],", |w| {
                    for method in &self.methods {
                        let method_name = snake_name(method.proto.get_name());
                        w.block("::grpc::server::ServerMethod::new(", "),", |w| {
                            method.write_descriptor(w, "::std::sync::Arc::new(", "),");
                            w.block("{", "},", |w| {
                                w.write_line(&format!("let handler_copy = {}.clone();", handler));
                                w.write_line(&format!("::grpc::server::MethodHandler{}::new(move |p| handler_copy.{}(p))",
                                    method.streaming_upper(),
                                    method_name));
                            });
                        });
                    }
                });
            });
    }

    fn write_async_server(&self, w: &mut CodeWriter) {
        w.pub_struct(&self.async_server_name(), |w| {
            w.pub_field_decl("grpc_server", "::grpc::server::GrpcServer");
        });

        w.write_line("");

        w.impl_for_block("::std::ops::Deref", &self.async_server_name(), |w| {
            w.write_line("type Target = ::grpc::server::GrpcServer;");
            w.write_line("");
            w.def_fn("deref(&self) -> &Self::Target", |w| {
                w.write_line("&self.grpc_server");
            })
        });

        w.write_line("");

        w.impl_self_block(&self.async_server_name(), |w| {
            let sig = format!(
                "new<A : ::std::net::ToSocketAddrs, H : {} + 'static + Sync + Send + 'static>(addr: A, conf: ::grpc::server::GrpcServerConf, h: H) -> Self",
                self.async_intf_name());
            w.pub_fn(&sig, |w| {
                w.write_line(format!("let service_definition = {}::new_service_def(h);", self.async_server_name()));

                w.expr_block(&self.async_server_name(), |w| {
                    w.field_entry("grpc_server", "::grpc::server::GrpcServer::new_plain(addr, conf, service_definition)");
                });
            });
            w.write_line("");
            w.pub_fn(&format!("new_service_def<H : {} + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::server::ServerServiceDefinition", self.async_intf_name()), |w| {
                w.write_line("let handler_arc = ::std::sync::Arc::new(handler);");

                self.write_service_definition("", "", "handler_arc", w);
            });
        });
    }

    fn write(&self, w: &mut CodeWriter) {
        w.comment("interface");
        w.write_line("");
        self.write_sync_intf(w);
        w.write_line("");
        self.write_async_intf(w);
        w.write_line("");
        w.comment("sync client");
        w.write_line("");
        self.write_sync_client(w);
        w.write_line("");
        w.comment("async client");
        w.write_line("");
        self.write_async_client(w);
        w.write_line("");
        w.comment("sync server");
        w.write_line("");
        self.write_sync_server(w);
        w.write_line("");
        w.comment("async server");
        w.write_line("");
        self.write_async_server(w);
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

use std::collections::HashMap;

use protobuf;
use protobuf::compiler_plugin;
use protobuf::descriptor::*;
use protobuf::descriptorx::*;

use std::io::Write;

struct CodeWriter<'a> {
    writer: &'a mut (dyn Write + 'a),
    indent: String,
}

impl<'a> CodeWriter<'a> {
    pub fn new(writer: &'a mut dyn Write) -> CodeWriter<'a> {
        CodeWriter {
            writer,
            indent: "".to_string(),
        }
    }

    pub fn write_line<S: AsRef<str>>(&mut self, line: S) {
        (if line.as_ref().is_empty() {
            self.writer.write_all("\n".as_bytes())
        } else {
            let s: String = [self.indent.as_ref(), line.as_ref(), "\n"].concat();
            self.writer.write_all(s.as_bytes())
        })
        .unwrap();
    }

    pub fn write_generated(&mut self) {
        self.write_line("// This file is generated. Do not edit");
        self.write_generated_common();
    }

    fn write_generated_common(&mut self) {
        // https://secure.phabricator.com/T784
        self.write_line("// @generated");

        self.write_line("");
        self.comment("https://github.com/Manishearth/rust-clippy/issues/702");
        self.write_line("#![allow(unknown_lints)]");
        self.write_line("#![allow(clippy::all)]");
        self.write_line("");
        self.write_line("#![cfg_attr(rustfmt, rustfmt_skip)]");
        self.write_line("");
        self.write_line("#![allow(box_pointers)]");
        self.write_line("#![allow(dead_code)]");
        self.write_line("#![allow(missing_docs)]");
        self.write_line("#![allow(non_camel_case_types)]");
        self.write_line("#![allow(non_snake_case)]");
        self.write_line("#![allow(non_upper_case_globals)]");
        self.write_line("#![allow(trivial_casts)]");
        self.write_line("#![allow(unsafe_code)]");
        self.write_line("#![allow(unused_imports)]");
        self.write_line("#![allow(unused_results)]");
    }

    pub fn indented<F>(&mut self, cb: F)
    where
        F: Fn(&mut CodeWriter),
    {
        cb(&mut CodeWriter {
            writer: self.writer,
            indent: format!("{}    ", self.indent),
        });
    }

    #[allow(dead_code)]
    pub fn commented<F>(&mut self, cb: F)
    where
        F: Fn(&mut CodeWriter),
    {
        cb(&mut CodeWriter {
            writer: self.writer,
            indent: format!("// {}", self.indent),
        });
    }

    pub fn block<F>(&mut self, first_line: &str, last_line: &str, cb: F)
    where
        F: Fn(&mut CodeWriter),
    {
        self.write_line(first_line);
        self.indented(cb);
        self.write_line(last_line);
    }

    pub fn expr_block<F>(&mut self, prefix: &str, cb: F)
    where
        F: Fn(&mut CodeWriter),
    {
        self.block(&format!("{} {{", prefix), "}", cb);
    }

    pub fn impl_self_block<S: AsRef<str>, F>(&mut self, name: S, cb: F)
    where
        F: Fn(&mut CodeWriter),
    {
        self.expr_block(&format!("impl {}", name.as_ref()), cb);
    }

    pub fn impl_for_block<S1: AsRef<str>, S2: AsRef<str>, F>(&mut self, tr: S1, ty: S2, cb: F)
    where
        F: Fn(&mut CodeWriter),
    {
        self.expr_block(&format!("impl {} for {}", tr.as_ref(), ty.as_ref()), cb);
    }

    pub fn pub_struct<S: AsRef<str>, F>(&mut self, name: S, cb: F)
    where
        F: Fn(&mut CodeWriter),
    {
        self.expr_block(&format!("pub struct {}", name.as_ref()), cb);
    }

    pub fn pub_trait<F>(&mut self, name: &str, cb: F)
    where
        F: Fn(&mut CodeWriter),
    {
        self.expr_block(&format!("pub trait {}", name), cb);
    }

    pub fn field_entry(&mut self, name: &str, value: &str) {
        self.write_line(&format!("{}: {},", name, value));
    }

    pub fn field_decl(&mut self, name: &str, field_type: &str) {
        self.write_line(&format!("{}: {},", name, field_type));
    }

    pub fn comment(&mut self, comment: &str) {
        if comment.is_empty() {
            self.write_line("//");
        } else {
            self.write_line(&format!("// {}", comment));
        }
    }

    pub fn fn_def(&mut self, sig: &str) {
        self.write_line(&format!("fn {};", sig));
    }

    pub fn fn_block<F>(&mut self, public: bool, sig: &str, cb: F)
    where
        F: Fn(&mut CodeWriter),
    {
        if public {
            self.expr_block(&format!("pub fn {}", sig), cb);
        } else {
            self.expr_block(&format!("fn {}", sig), cb);
        }
    }

    pub fn pub_fn<F>(&mut self, sig: &str, cb: F)
    where
        F: Fn(&mut CodeWriter),
    {
        self.fn_block(true, sig, cb);
    }

    pub fn def_fn<F>(&mut self, sig: &str, cb: F)
    where
        F: Fn(&mut CodeWriter),
    {
        self.fn_block(false, sig, cb);
    }
}

/// Adjust method name to follow the rust's style.
fn snake_name(name: &str) -> String {
    let mut snake_method_name = String::with_capacity(name.len());
    let mut chars = name.chars();
    // initial char can be any char except '_'.
    let mut last_char = '.';
    'outer: while let Some(c) = chars.next() {
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
    fn new(
        proto: &'a MethodDescriptorProto,
        service_path: String,
        root_scope: &'a RootScope<'a>,
    ) -> MethodGen<'a> {
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
        format!(
            "super::{}",
            self.root_scope
                .find_message(self.proto.get_input_type())
                .rust_fq_name()
        )
    }

    fn output_message(&self) -> String {
        format!(
            "super::{}",
            self.root_scope
                .find_message(self.proto.get_output_type())
                .rust_fq_name()
        )
    }

    fn client_resp_type(&self) -> String {
        match self.proto.get_server_streaming() {
            false => format!("::grpc::SingleResponse<{}>", self.output_message()),
            true => format!("::grpc::StreamingResponse<{}>", self.output_message()),
        }
    }

    fn client_sig(&self) -> String {
        let req_param = match self.proto.get_client_streaming() {
            false => format!(", req: {}", self.input_message()),
            true => format!(""),
        };
        let resp_type = self.client_resp_type();
        let return_type = match self.proto.get_client_streaming() {
            false => resp_type,
            true => format!(
                "impl ::std::future::Future<Output=::grpc::Result<(::grpc::ClientRequestSink<{}>, {})>>",
                self.input_message(),
                resp_type
            ),
        };
        format!(
            "{}(&self, o: ::grpc::RequestOptions{}) -> {}",
            self.snake_name(),
            req_param,
            return_type,
        )
    }

    fn server_req_type(&self) -> String {
        match self.proto.get_client_streaming() {
            false => format!("::grpc::ServerRequestSingle<{}>", self.input_message()),
            true => format!("::grpc::ServerRequest<{}>", self.input_message()),
        }
    }

    fn server_resp_type(&self) -> String {
        match self.proto.get_server_streaming() {
            false => format!("::grpc::ServerResponseUnarySink<{}>", self.output_message()),
            true => format!("::grpc::ServerResponseSink<{}>", self.output_message()),
        }
    }

    fn server_sig(&self) -> String {
        format!(
            "{}(&self, o: ::grpc::ServerHandlerContext, req: {}, resp: {}) -> ::grpc::Result<()>",
            self.snake_name(),
            self.server_req_type(),
            self.server_resp_type(),
        )
    }

    fn write_server_intf(&self, w: &mut CodeWriter) {
        w.fn_def(&self.server_sig())
    }

    fn streaming_upper(&self) -> &'static str {
        match (
            self.proto.get_client_streaming(),
            self.proto.get_server_streaming(),
        ) {
            (false, false) => "Unary",
            (false, true) => "ServerStreaming",
            (true, false) => "ClientStreaming",
            (true, true) => "Bidi",
        }
    }

    fn streaming_lower(&self) -> &'static str {
        match (
            self.proto.get_client_streaming(),
            self.proto.get_server_streaming(),
        ) {
            (false, false) => "unary",
            (false, true) => "server_streaming",
            (true, false) => "client_streaming",
            (true, true) => "bidi",
        }
    }

    fn write_client(&self, w: &mut CodeWriter) {
        w.pub_fn(&self.client_sig(), |w| {
            self.write_descriptor(
                w,
                "let descriptor = ::grpc::rt::ArcOrStatic::Static(&",
                ");",
            );

            let req = match self.proto.get_client_streaming() {
                false => ", req",
                true => "",
            };
            w.write_line(&format!(
                "self.grpc_client.call_{}(o{}, descriptor)",
                self.streaming_lower(),
                req,
            ))
        });
    }

    fn write_descriptor(&self, w: &mut CodeWriter, before: &str, after: &str) {
        w.block(
            &format!("{}{}", before, "::grpc::rt::MethodDescriptor {"),
            &format!("{}{}", "}", after),
            |w| {
                w.field_entry(
                    "name",
                    &format!(
                        "::grpc::rt::StringOrStatic::Static(\"{}/{}\")",
                        self.service_path,
                        self.proto.get_name()
                    ),
                );
                w.field_entry(
                    "streaming",
                    &format!("::grpc::rt::GrpcStreaming::{}", self.streaming_upper()),
                );
                w.field_entry(
                    "req_marshaller",
                    "::grpc::rt::ArcOrStatic::Static(&::grpc_protobuf::MarshallerProtobuf)",
                );
                w.field_entry(
                    "resp_marshaller",
                    "::grpc::rt::ArcOrStatic::Static(&::grpc_protobuf::MarshallerProtobuf)",
                );
            },
        );
    }
}

struct ServiceGen<'a> {
    proto: &'a ServiceDescriptorProto,
    _root_scope: &'a RootScope<'a>,
    methods: Vec<MethodGen<'a>>,
    service_path: String,
    _package: String,
}

impl<'a> ServiceGen<'a> {
    fn new(
        proto: &'a ServiceDescriptorProto,
        file: &FileDescriptorProto,
        root_scope: &'a RootScope,
    ) -> ServiceGen<'a> {
        let service_path = if file.get_package().is_empty() {
            format!("/{}", proto.get_name())
        } else {
            format!("/{}.{}", file.get_package(), proto.get_name())
        };
        let methods = proto
            .get_method()
            .into_iter()
            .map(|m| MethodGen::new(m, service_path.clone(), root_scope))
            .collect();

        ServiceGen {
            proto,
            _root_scope: root_scope,
            methods,
            service_path,
            _package: file.get_package().to_string(),
        }
    }

    // trait name
    fn server_intf_name(&self) -> &str {
        self.proto.get_name()
    }

    // client struct name
    fn client_name(&self) -> String {
        format!("{}Client", self.proto.get_name())
    }

    // server struct name
    fn server_name(&self) -> String {
        format!("{}Server", self.proto.get_name())
    }

    fn write_server_intf(&self, w: &mut CodeWriter) {
        w.pub_trait(&self.server_intf_name(), |w| {
            for (i, method) in self.methods.iter().enumerate() {
                if i != 0 {
                    w.write_line("");
                }

                method.write_server_intf(w);
            }
        });
    }

    fn write_client_object(&self, grpc_client: &str, w: &mut CodeWriter) {
        w.expr_block(&self.client_name(), |w| {
            w.field_entry("grpc_client", grpc_client);
        });
    }

    fn write_client(&self, w: &mut CodeWriter) {
        w.pub_struct(&self.client_name(), |w| {
            w.field_decl("grpc_client", "::std::sync::Arc<::grpc::Client>");
        });

        w.write_line("");

        w.impl_for_block("::grpc::ClientStub", &self.client_name(), |w| {
            let sig = "with_client(grpc_client: ::std::sync::Arc<::grpc::Client>) -> Self";
            w.def_fn(sig, |w| {
                self.write_client_object("grpc_client", w);
            });
        });

        w.write_line("");

        w.impl_self_block(&self.client_name(), |w| {
            for (i, method) in self.methods.iter().enumerate() {
                if i != 0 {
                    w.write_line("");
                }

                method.write_client(w);
            }
        });
    }

    fn write_service_definition(
        &self,
        before: &str,
        after: &str,
        handler: &str,
        w: &mut CodeWriter,
    ) {
        w.block(
            &format!("{}::grpc::rt::ServerServiceDefinition::new(\"{}\",",
                before, self.service_path),
            &format!("){}", after),
            |w| {
                w.block("vec![", "],", |w| {
                    for method in &self.methods {
                        w.block("::grpc::rt::ServerMethod::new(", "),", |w| {
                            method.write_descriptor(w, "::grpc::rt::ArcOrStatic::Static(&", "),");
                            w.block("{", "},", |w| {
                                w.write_line(&format!("let handler_copy = {}.clone();", handler));
                                w.write_line(&format!("::grpc::rt::MethodHandler{}::new(move |ctx, req, resp| (*handler_copy).{}(ctx, req, resp))",
                                    method.streaming_upper(),
                                    method.snake_name()));
                            });
                        });
                    }
                });
            });
    }

    fn write_server(&self, w: &mut CodeWriter) {
        w.write_line(&format!("pub struct {};", self.server_name()));

        w.write_line("");

        w.write_line("");

        w.impl_self_block(&self.server_name(), |w| {
            w.pub_fn(&format!("new_service_def<H : {} + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::rt::ServerServiceDefinition", self.server_intf_name()), |w| {
                w.write_line("let handler_arc = ::std::sync::Arc::new(handler);");

                self.write_service_definition("", "", "handler_arc", w);
            });
        });
    }

    fn write(&self, w: &mut CodeWriter) {
        w.comment("server interface");
        w.write_line("");
        self.write_server_intf(w);
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
) -> Option<compiler_plugin::GenResult> {
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

pub fn gen(
    file_descriptors: &[FileDescriptorProto],
    files_to_generate: &[String],
) -> Vec<compiler_plugin::GenResult> {
    let files_map: HashMap<&str, &FileDescriptorProto> =
        file_descriptors.iter().map(|f| (f.get_name(), f)).collect();

    let root_scope = RootScope {
        file_descriptors: file_descriptors,
    };

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

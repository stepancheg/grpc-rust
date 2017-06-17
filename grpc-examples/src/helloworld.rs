// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

#[derive(PartialEq,Clone,Default)]
pub struct HelloRequest {
    // message fields
    pub name: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for HelloRequest {}

impl HelloRequest {
    pub fn new() -> HelloRequest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static HelloRequest {
        static mut instance: ::protobuf::lazy::Lazy<HelloRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const HelloRequest,
        };
        unsafe {
            instance.get(HelloRequest::new)
        }
    }

    // string name = 1;

    pub fn clear_name(&mut self) {
        self.name.clear();
    }

    // Param is passed by value, moved
    pub fn set_name(&mut self, v: ::std::string::String) {
        self.name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_name(&mut self) -> &mut ::std::string::String {
        &mut self.name
    }

    // Take field
    pub fn take_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.name, ::std::string::String::new())
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    fn get_name_for_reflect(&self) -> &::std::string::String {
        &self.name
    }

    fn mut_name_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.name
    }
}

impl ::protobuf::Message for HelloRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.name)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.name.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.name);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.name.is_empty() {
            os.write_string(1, &self.name)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for HelloRequest {
    fn new() -> HelloRequest {
        HelloRequest::new()
    }

    fn descriptor_static(_: ::std::option::Option<HelloRequest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "name",
                    HelloRequest::get_name_for_reflect,
                    HelloRequest::mut_name_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<HelloRequest>(
                    "HelloRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for HelloRequest {
    fn clear(&mut self) {
        self.clear_name();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for HelloRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for HelloRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct HelloReply {
    // message fields
    pub message: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for HelloReply {}

impl HelloReply {
    pub fn new() -> HelloReply {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static HelloReply {
        static mut instance: ::protobuf::lazy::Lazy<HelloReply> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const HelloReply,
        };
        unsafe {
            instance.get(HelloReply::new)
        }
    }

    // string message = 1;

    pub fn clear_message(&mut self) {
        self.message.clear();
    }

    // Param is passed by value, moved
    pub fn set_message(&mut self, v: ::std::string::String) {
        self.message = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_message(&mut self) -> &mut ::std::string::String {
        &mut self.message
    }

    // Take field
    pub fn take_message(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.message, ::std::string::String::new())
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }

    fn get_message_for_reflect(&self) -> &::std::string::String {
        &self.message
    }

    fn mut_message_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.message
    }
}

impl ::protobuf::Message for HelloReply {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.message)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.message.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.message);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.message.is_empty() {
            os.write_string(1, &self.message)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for HelloReply {
    fn new() -> HelloReply {
        HelloReply::new()
    }

    fn descriptor_static(_: ::std::option::Option<HelloReply>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "message",
                    HelloReply::get_message_for_reflect,
                    HelloReply::mut_message_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<HelloReply>(
                    "HelloReply",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for HelloReply {
    fn clear(&mut self) {
        self.clear_message();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for HelloReply {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for HelloReply {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x10helloworld.proto\x12\nhelloworld\"\"\n\x0cHelloRequest\x12\x12\n\
    \x04name\x18\x01\x20\x01(\tR\x04name\"&\n\nHelloReply\x12\x18\n\x07messa\
    ge\x18\x01\x20\x01(\tR\x07message2I\n\x07Greeter\x12>\n\x08SayHello\x12\
    \x18.helloworld.HelloRequest\x1a\x16.helloworld.HelloReply\"\0B6\n\x1bio\
    .grpc.examples.helloworldB\x0fHelloWorldProtoP\x01\xa2\x02\x03HLWJ\xeb\
    \x11\n\x06\x12\x04\x1d\04\x01\n\xe7\x0b\n\x01\x0c\x12\x03\x1d\0\x122\xdc\
    \x0b\x20Copyright\x202015,\x20Google\x20Inc.\n\x20All\x20rights\x20reser\
    ved.\n\n\x20Redistribution\x20and\x20use\x20in\x20source\x20and\x20binar\
    y\x20forms,\x20with\x20or\x20without\n\x20modification,\x20are\x20permit\
    ted\x20provided\x20that\x20the\x20following\x20conditions\x20are\n\x20me\
    t:\n\n\x20\x20\x20\x20\x20*\x20Redistributions\x20of\x20source\x20code\
    \x20must\x20retain\x20the\x20above\x20copyright\n\x20notice,\x20this\x20\
    list\x20of\x20conditions\x20and\x20the\x20following\x20disclaimer.\n\x20\
    \x20\x20\x20\x20*\x20Redistributions\x20in\x20binary\x20form\x20must\x20\
    reproduce\x20the\x20above\n\x20copyright\x20notice,\x20this\x20list\x20o\
    f\x20conditions\x20and\x20the\x20following\x20disclaimer\n\x20in\x20the\
    \x20documentation\x20and/or\x20other\x20materials\x20provided\x20with\
    \x20the\n\x20distribution.\n\x20\x20\x20\x20\x20*\x20Neither\x20the\x20n\
    ame\x20of\x20Google\x20Inc.\x20nor\x20the\x20names\x20of\x20its\n\x20con\
    tributors\x20may\x20be\x20used\x20to\x20endorse\x20or\x20promote\x20prod\
    ucts\x20derived\x20from\n\x20this\x20software\x20without\x20specific\x20\
    prior\x20written\x20permission.\n\n\x20THIS\x20SOFTWARE\x20IS\x20PROVIDE\
    D\x20BY\x20THE\x20COPYRIGHT\x20HOLDERS\x20AND\x20CONTRIBUTORS\n\x20\"AS\
    \x20IS\"\x20AND\x20ANY\x20EXPRESS\x20OR\x20IMPLIED\x20WARRANTIES,\x20INC\
    LUDING,\x20BUT\x20NOT\n\x20LIMITED\x20TO,\x20THE\x20IMPLIED\x20WARRANTIE\
    S\x20OF\x20MERCHANTABILITY\x20AND\x20FITNESS\x20FOR\n\x20A\x20PARTICULAR\
    \x20PURPOSE\x20ARE\x20DISCLAIMED.\x20IN\x20NO\x20EVENT\x20SHALL\x20THE\
    \x20COPYRIGHT\n\x20OWNER\x20OR\x20CONTRIBUTORS\x20BE\x20LIABLE\x20FOR\
    \x20ANY\x20DIRECT,\x20INDIRECT,\x20INCIDENTAL,\n\x20SPECIAL,\x20EXEMPLAR\
    Y,\x20OR\x20CONSEQUENTIAL\x20DAMAGES\x20(INCLUDING,\x20BUT\x20NOT\n\x20L\
    IMITED\x20TO,\x20PROCUREMENT\x20OF\x20SUBSTITUTE\x20GOODS\x20OR\x20SERVI\
    CES;\x20LOSS\x20OF\x20USE,\n\x20DATA,\x20OR\x20PROFITS;\x20OR\x20BUSINES\
    S\x20INTERRUPTION)\x20HOWEVER\x20CAUSED\x20AND\x20ON\x20ANY\n\x20THEORY\
    \x20OF\x20LIABILITY,\x20WHETHER\x20IN\x20CONTRACT,\x20STRICT\x20LIABILIT\
    Y,\x20OR\x20TORT\n\x20(INCLUDING\x20NEGLIGENCE\x20OR\x20OTHERWISE)\x20AR\
    ISING\x20IN\x20ANY\x20WAY\x20OUT\x20OF\x20THE\x20USE\n\x20OF\x20THIS\x20\
    SOFTWARE,\x20EVEN\x20IF\x20ADVISED\x20OF\x20THE\x20POSSIBILITY\x20OF\x20\
    SUCH\x20DAMAGE.\n\n\x08\n\x01\x08\x12\x03\x1f\0\"\n\x0b\n\x04\x08\xe7\
    \x07\0\x12\x03\x1f\0\"\n\x0c\n\x05\x08\xe7\x07\0\x02\x12\x03\x1f\x07\x1a\
    \n\r\n\x06\x08\xe7\x07\0\x02\0\x12\x03\x1f\x07\x1a\n\x0e\n\x07\x08\xe7\
    \x07\0\x02\0\x01\x12\x03\x1f\x07\x1a\n\x0c\n\x05\x08\xe7\x07\0\x03\x12\
    \x03\x1f\x1d!\n\x08\n\x01\x08\x12\x03\x20\04\n\x0b\n\x04\x08\xe7\x07\x01\
    \x12\x03\x20\04\n\x0c\n\x05\x08\xe7\x07\x01\x02\x12\x03\x20\x07\x13\n\r\
    \n\x06\x08\xe7\x07\x01\x02\0\x12\x03\x20\x07\x13\n\x0e\n\x07\x08\xe7\x07\
    \x01\x02\0\x01\x12\x03\x20\x07\x13\n\x0c\n\x05\x08\xe7\x07\x01\x07\x12\
    \x03\x20\x163\n\x08\n\x01\x08\x12\x03!\00\n\x0b\n\x04\x08\xe7\x07\x02\
    \x12\x03!\00\n\x0c\n\x05\x08\xe7\x07\x02\x02\x12\x03!\x07\x1b\n\r\n\x06\
    \x08\xe7\x07\x02\x02\0\x12\x03!\x07\x1b\n\x0e\n\x07\x08\xe7\x07\x02\x02\
    \0\x01\x12\x03!\x07\x1b\n\x0c\n\x05\x08\xe7\x07\x02\x07\x12\x03!\x1e/\n\
    \x08\n\x01\x08\x12\x03\"\0!\n\x0b\n\x04\x08\xe7\x07\x03\x12\x03\"\0!\n\
    \x0c\n\x05\x08\xe7\x07\x03\x02\x12\x03\"\x07\x18\n\r\n\x06\x08\xe7\x07\
    \x03\x02\0\x12\x03\"\x07\x18\n\x0e\n\x07\x08\xe7\x07\x03\x02\0\x01\x12\
    \x03\"\x07\x18\n\x0c\n\x05\x08\xe7\x07\x03\x07\x12\x03\"\x1b\x20\n\x08\n\
    \x01\x02\x12\x03$\x08\x12\n.\n\x02\x06\0\x12\x04'\0*\x01\x1a\"\x20The\
    \x20greeting\x20service\x20definition.\n\n\n\n\x03\x06\0\x01\x12\x03'\
    \x08\x0f\n\x1f\n\x04\x06\0\x02\0\x12\x03)\x025\x1a\x12\x20Sends\x20a\x20\
    greeting\n\n\x0c\n\x05\x06\0\x02\0\x01\x12\x03)\x06\x0e\n\x0c\n\x05\x06\
    \0\x02\0\x02\x12\x03)\x10\x1c\n\x0c\n\x05\x06\0\x02\0\x03\x12\x03)'1\n=\
    \n\x02\x04\0\x12\x04-\0/\x01\x1a1\x20The\x20request\x20message\x20contai\
    ning\x20the\x20user's\x20name.\n\n\n\n\x03\x04\0\x01\x12\x03-\x08\x14\n\
    \x0b\n\x04\x04\0\x02\0\x12\x03.\x02\x12\n\r\n\x05\x04\0\x02\0\x04\x12\
    \x04.\x02-\x16\n\x0c\n\x05\x04\0\x02\0\x05\x12\x03.\x02\x08\n\x0c\n\x05\
    \x04\0\x02\0\x01\x12\x03.\t\r\n\x0c\n\x05\x04\0\x02\0\x03\x12\x03.\x10\
    \x11\n;\n\x02\x04\x01\x12\x042\04\x01\x1a/\x20The\x20response\x20message\
    \x20containing\x20the\x20greetings\n\n\n\n\x03\x04\x01\x01\x12\x032\x08\
    \x12\n\x0b\n\x04\x04\x01\x02\0\x12\x033\x02\x15\n\r\n\x05\x04\x01\x02\0\
    \x04\x12\x043\x022\x14\n\x0c\n\x05\x04\x01\x02\0\x05\x12\x033\x02\x08\n\
    \x0c\n\x05\x04\x01\x02\0\x01\x12\x033\t\x10\n\x0c\n\x05\x04\x01\x02\0\
    \x03\x12\x033\x13\x14b\x06proto3\
";

static mut file_descriptor_proto_lazy: ::protobuf::lazy::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::lazy::Lazy {
    lock: ::protobuf::lazy::ONCE_INIT,
    ptr: 0 as *const ::protobuf::descriptor::FileDescriptorProto,
};

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    unsafe {
        file_descriptor_proto_lazy.get(|| {
            parse_descriptor_proto()
        })
    }
}

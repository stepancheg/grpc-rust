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
pub struct EchoRequest {
    // message fields
    pub payload: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for EchoRequest {}

impl EchoRequest {
    pub fn new() -> EchoRequest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static EchoRequest {
        static mut instance: ::protobuf::lazy::Lazy<EchoRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const EchoRequest,
        };
        unsafe {
            instance.get(EchoRequest::new)
        }
    }

    // string payload = 1;

    pub fn clear_payload(&mut self) {
        self.payload.clear();
    }

    // Param is passed by value, moved
    pub fn set_payload(&mut self, v: ::std::string::String) {
        self.payload = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_payload(&mut self) -> &mut ::std::string::String {
        &mut self.payload
    }

    // Take field
    pub fn take_payload(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.payload, ::std::string::String::new())
    }

    pub fn get_payload(&self) -> &str {
        &self.payload
    }

    fn get_payload_for_reflect(&self) -> &::std::string::String {
        &self.payload
    }

    fn mut_payload_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.payload
    }
}

impl ::protobuf::Message for EchoRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.payload)?;
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
        if !self.payload.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.payload);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.payload.is_empty() {
            os.write_string(1, &self.payload)?;
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

impl ::protobuf::MessageStatic for EchoRequest {
    fn new() -> EchoRequest {
        EchoRequest::new()
    }

    fn descriptor_static(_: ::std::option::Option<EchoRequest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "payload",
                    EchoRequest::get_payload_for_reflect,
                    EchoRequest::mut_payload_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<EchoRequest>(
                    "EchoRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for EchoRequest {
    fn clear(&mut self) {
        self.clear_payload();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for EchoRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for EchoRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct EchoResponse {
    // message fields
    pub payload: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for EchoResponse {}

impl EchoResponse {
    pub fn new() -> EchoResponse {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static EchoResponse {
        static mut instance: ::protobuf::lazy::Lazy<EchoResponse> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const EchoResponse,
        };
        unsafe {
            instance.get(EchoResponse::new)
        }
    }

    // string payload = 2;

    pub fn clear_payload(&mut self) {
        self.payload.clear();
    }

    // Param is passed by value, moved
    pub fn set_payload(&mut self, v: ::std::string::String) {
        self.payload = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_payload(&mut self) -> &mut ::std::string::String {
        &mut self.payload
    }

    // Take field
    pub fn take_payload(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.payload, ::std::string::String::new())
    }

    pub fn get_payload(&self) -> &str {
        &self.payload
    }

    fn get_payload_for_reflect(&self) -> &::std::string::String {
        &self.payload
    }

    fn mut_payload_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.payload
    }
}

impl ::protobuf::Message for EchoResponse {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.payload)?;
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
        if !self.payload.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.payload);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.payload.is_empty() {
            os.write_string(2, &self.payload)?;
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

impl ::protobuf::MessageStatic for EchoResponse {
    fn new() -> EchoResponse {
        EchoResponse::new()
    }

    fn descriptor_static(_: ::std::option::Option<EchoResponse>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "payload",
                    EchoResponse::get_payload_for_reflect,
                    EchoResponse::mut_payload_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<EchoResponse>(
                    "EchoResponse",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for EchoResponse {
    fn clear(&mut self) {
        self.clear_payload();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for EchoResponse {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for EchoResponse {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct CharCountRequest {
    // message fields
    pub part: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for CharCountRequest {}

impl CharCountRequest {
    pub fn new() -> CharCountRequest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static CharCountRequest {
        static mut instance: ::protobuf::lazy::Lazy<CharCountRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const CharCountRequest,
        };
        unsafe {
            instance.get(CharCountRequest::new)
        }
    }

    // string part = 1;

    pub fn clear_part(&mut self) {
        self.part.clear();
    }

    // Param is passed by value, moved
    pub fn set_part(&mut self, v: ::std::string::String) {
        self.part = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_part(&mut self) -> &mut ::std::string::String {
        &mut self.part
    }

    // Take field
    pub fn take_part(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.part, ::std::string::String::new())
    }

    pub fn get_part(&self) -> &str {
        &self.part
    }

    fn get_part_for_reflect(&self) -> &::std::string::String {
        &self.part
    }

    fn mut_part_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.part
    }
}

impl ::protobuf::Message for CharCountRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.part)?;
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
        if !self.part.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.part);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.part.is_empty() {
            os.write_string(1, &self.part)?;
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

impl ::protobuf::MessageStatic for CharCountRequest {
    fn new() -> CharCountRequest {
        CharCountRequest::new()
    }

    fn descriptor_static(_: ::std::option::Option<CharCountRequest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "part",
                    CharCountRequest::get_part_for_reflect,
                    CharCountRequest::mut_part_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<CharCountRequest>(
                    "CharCountRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for CharCountRequest {
    fn clear(&mut self) {
        self.clear_part();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for CharCountRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for CharCountRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct CharCountResponse {
    // message fields
    pub char_count: u64,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for CharCountResponse {}

impl CharCountResponse {
    pub fn new() -> CharCountResponse {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static CharCountResponse {
        static mut instance: ::protobuf::lazy::Lazy<CharCountResponse> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const CharCountResponse,
        };
        unsafe {
            instance.get(CharCountResponse::new)
        }
    }

    // uint64 char_count = 1;

    pub fn clear_char_count(&mut self) {
        self.char_count = 0;
    }

    // Param is passed by value, moved
    pub fn set_char_count(&mut self, v: u64) {
        self.char_count = v;
    }

    pub fn get_char_count(&self) -> u64 {
        self.char_count
    }

    fn get_char_count_for_reflect(&self) -> &u64 {
        &self.char_count
    }

    fn mut_char_count_for_reflect(&mut self) -> &mut u64 {
        &mut self.char_count
    }
}

impl ::protobuf::Message for CharCountResponse {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.char_count = tmp;
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
        if self.char_count != 0 {
            my_size += ::protobuf::rt::value_size(1, self.char_count, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.char_count != 0 {
            os.write_uint64(1, self.char_count)?;
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

impl ::protobuf::MessageStatic for CharCountResponse {
    fn new() -> CharCountResponse {
        CharCountResponse::new()
    }

    fn descriptor_static(_: ::std::option::Option<CharCountResponse>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                    "char_count",
                    CharCountResponse::get_char_count_for_reflect,
                    CharCountResponse::mut_char_count_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<CharCountResponse>(
                    "CharCountResponse",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for CharCountResponse {
    fn clear(&mut self) {
        self.clear_char_count();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for CharCountResponse {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for CharCountResponse {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct RandomStringsRequest {
    // message fields
    pub count: u64,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for RandomStringsRequest {}

impl RandomStringsRequest {
    pub fn new() -> RandomStringsRequest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static RandomStringsRequest {
        static mut instance: ::protobuf::lazy::Lazy<RandomStringsRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const RandomStringsRequest,
        };
        unsafe {
            instance.get(RandomStringsRequest::new)
        }
    }

    // uint64 count = 1;

    pub fn clear_count(&mut self) {
        self.count = 0;
    }

    // Param is passed by value, moved
    pub fn set_count(&mut self, v: u64) {
        self.count = v;
    }

    pub fn get_count(&self) -> u64 {
        self.count
    }

    fn get_count_for_reflect(&self) -> &u64 {
        &self.count
    }

    fn mut_count_for_reflect(&mut self) -> &mut u64 {
        &mut self.count
    }
}

impl ::protobuf::Message for RandomStringsRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.count = tmp;
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
        if self.count != 0 {
            my_size += ::protobuf::rt::value_size(1, self.count, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.count != 0 {
            os.write_uint64(1, self.count)?;
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

impl ::protobuf::MessageStatic for RandomStringsRequest {
    fn new() -> RandomStringsRequest {
        RandomStringsRequest::new()
    }

    fn descriptor_static(_: ::std::option::Option<RandomStringsRequest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                    "count",
                    RandomStringsRequest::get_count_for_reflect,
                    RandomStringsRequest::mut_count_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<RandomStringsRequest>(
                    "RandomStringsRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for RandomStringsRequest {
    fn clear(&mut self) {
        self.clear_count();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for RandomStringsRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for RandomStringsRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct RandomStringsResponse {
    // message fields
    pub s: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for RandomStringsResponse {}

impl RandomStringsResponse {
    pub fn new() -> RandomStringsResponse {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static RandomStringsResponse {
        static mut instance: ::protobuf::lazy::Lazy<RandomStringsResponse> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const RandomStringsResponse,
        };
        unsafe {
            instance.get(RandomStringsResponse::new)
        }
    }

    // string s = 1;

    pub fn clear_s(&mut self) {
        self.s.clear();
    }

    // Param is passed by value, moved
    pub fn set_s(&mut self, v: ::std::string::String) {
        self.s = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_s(&mut self) -> &mut ::std::string::String {
        &mut self.s
    }

    // Take field
    pub fn take_s(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.s, ::std::string::String::new())
    }

    pub fn get_s(&self) -> &str {
        &self.s
    }

    fn get_s_for_reflect(&self) -> &::std::string::String {
        &self.s
    }

    fn mut_s_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.s
    }
}

impl ::protobuf::Message for RandomStringsResponse {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.s)?;
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
        if !self.s.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.s);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.s.is_empty() {
            os.write_string(1, &self.s)?;
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

impl ::protobuf::MessageStatic for RandomStringsResponse {
    fn new() -> RandomStringsResponse {
        RandomStringsResponse::new()
    }

    fn descriptor_static(_: ::std::option::Option<RandomStringsResponse>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "s",
                    RandomStringsResponse::get_s_for_reflect,
                    RandomStringsResponse::mut_s_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<RandomStringsResponse>(
                    "RandomStringsResponse",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for RandomStringsResponse {
    fn clear(&mut self) {
        self.clear_s();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for RandomStringsResponse {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for RandomStringsResponse {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x13long_tests_pb.proto\"'\n\x0bEchoRequest\x12\x18\n\x07payload\x18\
    \x01\x20\x01(\tR\x07payload\"(\n\x0cEchoResponse\x12\x18\n\x07payload\
    \x18\x02\x20\x01(\tR\x07payload\"&\n\x10CharCountRequest\x12\x12\n\x04pa\
    rt\x18\x01\x20\x01(\tR\x04part\"2\n\x11CharCountResponse\x12\x1d\n\nchar\
    _count\x18\x01\x20\x01(\x04R\tcharCount\",\n\x14RandomStringsRequest\x12\
    \x14\n\x05count\x18\x01\x20\x01(\x04R\x05count\"%\n\x15RandomStringsResp\
    onse\x12\x0c\n\x01s\x18\x01\x20\x01(\tR\x01s2\xaa\x01\n\tLongTests\x12#\
    \n\x04echo\x12\x0c.EchoRequest\x1a\r.EchoResponse\x125\n\nchar_count\x12\
    \x11.CharCountRequest\x1a\x12.CharCountResponse(\x01\x12A\n\x0erandom_st\
    rings\x12\x15.RandomStringsRequest\x1a\x16.RandomStringsResponse0\x01J\
    \xd5\x06\n\x06\x12\x04\0\0\"\x01\n\x08\n\x01\x0c\x12\x03\0\0\x12\n\n\n\
    \x02\x04\0\x12\x04\x03\0\x05\x01\n\n\n\x03\x04\0\x01\x12\x03\x03\x08\x13\
    \n\x0b\n\x04\x04\0\x02\0\x12\x03\x04\x04\x17\n\r\n\x05\x04\0\x02\0\x04\
    \x12\x04\x04\x04\x03\x15\n\x0c\n\x05\x04\0\x02\0\x05\x12\x03\x04\x04\n\n\
    \x0c\n\x05\x04\0\x02\0\x01\x12\x03\x04\x0b\x12\n\x0c\n\x05\x04\0\x02\0\
    \x03\x12\x03\x04\x15\x16\n\n\n\x02\x04\x01\x12\x04\x07\0\t\x01\n\n\n\x03\
    \x04\x01\x01\x12\x03\x07\x08\x14\n\x0b\n\x04\x04\x01\x02\0\x12\x03\x08\
    \x04\x17\n\r\n\x05\x04\x01\x02\0\x04\x12\x04\x08\x04\x07\x16\n\x0c\n\x05\
    \x04\x01\x02\0\x05\x12\x03\x08\x04\n\n\x0c\n\x05\x04\x01\x02\0\x01\x12\
    \x03\x08\x0b\x12\n\x0c\n\x05\x04\x01\x02\0\x03\x12\x03\x08\x15\x16\n\n\n\
    \x02\x04\x02\x12\x04\x0b\0\r\x01\n\n\n\x03\x04\x02\x01\x12\x03\x0b\x08\
    \x18\n\x0b\n\x04\x04\x02\x02\0\x12\x03\x0c\x04\x14\n\r\n\x05\x04\x02\x02\
    \0\x04\x12\x04\x0c\x04\x0b\x1a\n\x0c\n\x05\x04\x02\x02\0\x05\x12\x03\x0c\
    \x04\n\n\x0c\n\x05\x04\x02\x02\0\x01\x12\x03\x0c\x0b\x0f\n\x0c\n\x05\x04\
    \x02\x02\0\x03\x12\x03\x0c\x12\x13\n\n\n\x02\x04\x03\x12\x04\x0f\0\x11\
    \x01\n\n\n\x03\x04\x03\x01\x12\x03\x0f\x08\x19\n\x0b\n\x04\x04\x03\x02\0\
    \x12\x03\x10\x04\x1a\n\r\n\x05\x04\x03\x02\0\x04\x12\x04\x10\x04\x0f\x1b\
    \n\x0c\n\x05\x04\x03\x02\0\x05\x12\x03\x10\x04\n\n\x0c\n\x05\x04\x03\x02\
    \0\x01\x12\x03\x10\x0b\x15\n\x0c\n\x05\x04\x03\x02\0\x03\x12\x03\x10\x18\
    \x19\n\n\n\x02\x04\x04\x12\x04\x13\0\x15\x01\n\n\n\x03\x04\x04\x01\x12\
    \x03\x13\x08\x1c\n\x0b\n\x04\x04\x04\x02\0\x12\x03\x14\x04\x15\n\r\n\x05\
    \x04\x04\x02\0\x04\x12\x04\x14\x04\x13\x1e\n\x0c\n\x05\x04\x04\x02\0\x05\
    \x12\x03\x14\x04\n\n\x0c\n\x05\x04\x04\x02\0\x01\x12\x03\x14\x0b\x10\n\
    \x0c\n\x05\x04\x04\x02\0\x03\x12\x03\x14\x13\x14\n\n\n\x02\x04\x05\x12\
    \x04\x17\0\x19\x01\n\n\n\x03\x04\x05\x01\x12\x03\x17\x08\x1d\n\x0b\n\x04\
    \x04\x05\x02\0\x12\x03\x18\x04\x11\n\r\n\x05\x04\x05\x02\0\x04\x12\x04\
    \x18\x04\x17\x1f\n\x0c\n\x05\x04\x05\x02\0\x05\x12\x03\x18\x04\n\n\x0c\n\
    \x05\x04\x05\x02\0\x01\x12\x03\x18\x0b\x0c\n\x0c\n\x05\x04\x05\x02\0\x03\
    \x12\x03\x18\x0f\x10\n\n\n\x02\x06\0\x12\x04\x1b\0\"\x01\n\n\n\x03\x06\0\
    \x01\x12\x03\x1b\x08\x11\n\x19\n\x04\x06\0\x02\0\x12\x03\x1d\x042\x1a\
    \x0c\x20simple\x20RPC\n\n\x0c\n\x05\x06\0\x02\0\x01\x12\x03\x1d\x08\x0c\
    \n\x0c\n\x05\x06\0\x02\0\x02\x12\x03\x1d\x0e\x19\n\x0c\n\x05\x06\0\x02\0\
    \x03\x12\x03\x1d$0\n\x1f\n\x04\x06\0\x02\x01\x12\x03\x1f\x04I\x1a\x12\
    \x20client\x20streaming\n\n\x0c\n\x05\x06\0\x02\x01\x01\x12\x03\x1f\x08\
    \x12\n\x0c\n\x05\x06\0\x02\x01\x05\x12\x03\x1f\x14\x1a\n\x0c\n\x05\x06\0\
    \x02\x01\x02\x12\x03\x1f\x1b+\n\x0c\n\x05\x06\0\x02\x01\x03\x12\x03\x1f6\
    G\n\x1f\n\x04\x06\0\x02\x02\x12\x03!\x04U\x1a\x12\x20server\x20streaming\
    \n\n\x0c\n\x05\x06\0\x02\x02\x01\x12\x03!\x08\x16\n\x0c\n\x05\x06\0\x02\
    \x02\x02\x12\x03!\x18,\n\x0c\n\x05\x06\0\x02\x02\x06\x12\x03!7=\n\x0c\n\
    \x05\x06\0\x02\x02\x03\x12\x03!>Sb\x06proto3\
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

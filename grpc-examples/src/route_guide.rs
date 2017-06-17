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
pub struct Point {
    // message fields
    pub latitude: i32,
    pub longitude: i32,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Point {}

impl Point {
    pub fn new() -> Point {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Point {
        static mut instance: ::protobuf::lazy::Lazy<Point> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Point,
        };
        unsafe {
            instance.get(Point::new)
        }
    }

    // int32 latitude = 1;

    pub fn clear_latitude(&mut self) {
        self.latitude = 0;
    }

    // Param is passed by value, moved
    pub fn set_latitude(&mut self, v: i32) {
        self.latitude = v;
    }

    pub fn get_latitude(&self) -> i32 {
        self.latitude
    }

    fn get_latitude_for_reflect(&self) -> &i32 {
        &self.latitude
    }

    fn mut_latitude_for_reflect(&mut self) -> &mut i32 {
        &mut self.latitude
    }

    // int32 longitude = 2;

    pub fn clear_longitude(&mut self) {
        self.longitude = 0;
    }

    // Param is passed by value, moved
    pub fn set_longitude(&mut self, v: i32) {
        self.longitude = v;
    }

    pub fn get_longitude(&self) -> i32 {
        self.longitude
    }

    fn get_longitude_for_reflect(&self) -> &i32 {
        &self.longitude
    }

    fn mut_longitude_for_reflect(&mut self) -> &mut i32 {
        &mut self.longitude
    }
}

impl ::protobuf::Message for Point {
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
                    let tmp = is.read_int32()?;
                    self.latitude = tmp;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.longitude = tmp;
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
        if self.latitude != 0 {
            my_size += ::protobuf::rt::value_size(1, self.latitude, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.longitude != 0 {
            my_size += ::protobuf::rt::value_size(2, self.longitude, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.latitude != 0 {
            os.write_int32(1, self.latitude)?;
        }
        if self.longitude != 0 {
            os.write_int32(2, self.longitude)?;
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

impl ::protobuf::MessageStatic for Point {
    fn new() -> Point {
        Point::new()
    }

    fn descriptor_static(_: ::std::option::Option<Point>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                    "latitude",
                    Point::get_latitude_for_reflect,
                    Point::mut_latitude_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                    "longitude",
                    Point::get_longitude_for_reflect,
                    Point::mut_longitude_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Point>(
                    "Point",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Point {
    fn clear(&mut self) {
        self.clear_latitude();
        self.clear_longitude();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Point {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Point {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Rectangle {
    // message fields
    lo: ::protobuf::SingularPtrField<Point>,
    hi: ::protobuf::SingularPtrField<Point>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Rectangle {}

impl Rectangle {
    pub fn new() -> Rectangle {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Rectangle {
        static mut instance: ::protobuf::lazy::Lazy<Rectangle> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Rectangle,
        };
        unsafe {
            instance.get(Rectangle::new)
        }
    }

    // .proto.Point lo = 1;

    pub fn clear_lo(&mut self) {
        self.lo.clear();
    }

    pub fn has_lo(&self) -> bool {
        self.lo.is_some()
    }

    // Param is passed by value, moved
    pub fn set_lo(&mut self, v: Point) {
        self.lo = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_lo(&mut self) -> &mut Point {
        if self.lo.is_none() {
            self.lo.set_default();
        }
        self.lo.as_mut().unwrap()
    }

    // Take field
    pub fn take_lo(&mut self) -> Point {
        self.lo.take().unwrap_or_else(|| Point::new())
    }

    pub fn get_lo(&self) -> &Point {
        self.lo.as_ref().unwrap_or_else(|| Point::default_instance())
    }

    fn get_lo_for_reflect(&self) -> &::protobuf::SingularPtrField<Point> {
        &self.lo
    }

    fn mut_lo_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<Point> {
        &mut self.lo
    }

    // .proto.Point hi = 2;

    pub fn clear_hi(&mut self) {
        self.hi.clear();
    }

    pub fn has_hi(&self) -> bool {
        self.hi.is_some()
    }

    // Param is passed by value, moved
    pub fn set_hi(&mut self, v: Point) {
        self.hi = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_hi(&mut self) -> &mut Point {
        if self.hi.is_none() {
            self.hi.set_default();
        }
        self.hi.as_mut().unwrap()
    }

    // Take field
    pub fn take_hi(&mut self) -> Point {
        self.hi.take().unwrap_or_else(|| Point::new())
    }

    pub fn get_hi(&self) -> &Point {
        self.hi.as_ref().unwrap_or_else(|| Point::default_instance())
    }

    fn get_hi_for_reflect(&self) -> &::protobuf::SingularPtrField<Point> {
        &self.hi
    }

    fn mut_hi_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<Point> {
        &mut self.hi
    }
}

impl ::protobuf::Message for Rectangle {
    fn is_initialized(&self) -> bool {
        for v in &self.lo {
            if !v.is_initialized() {
                return false;
            }
        };
        for v in &self.hi {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.lo)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.hi)?;
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
        if let Some(ref v) = self.lo.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if let Some(ref v) = self.hi.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.lo.as_ref() {
            os.write_tag(1, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if let Some(ref v) = self.hi.as_ref() {
            os.write_tag(2, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
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

impl ::protobuf::MessageStatic for Rectangle {
    fn new() -> Rectangle {
        Rectangle::new()
    }

    fn descriptor_static(_: ::std::option::Option<Rectangle>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Point>>(
                    "lo",
                    Rectangle::get_lo_for_reflect,
                    Rectangle::mut_lo_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Point>>(
                    "hi",
                    Rectangle::get_hi_for_reflect,
                    Rectangle::mut_hi_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Rectangle>(
                    "Rectangle",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Rectangle {
    fn clear(&mut self) {
        self.clear_lo();
        self.clear_hi();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Rectangle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Rectangle {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Feature {
    // message fields
    pub name: ::std::string::String,
    location: ::protobuf::SingularPtrField<Point>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Feature {}

impl Feature {
    pub fn new() -> Feature {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Feature {
        static mut instance: ::protobuf::lazy::Lazy<Feature> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Feature,
        };
        unsafe {
            instance.get(Feature::new)
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

    // .proto.Point location = 2;

    pub fn clear_location(&mut self) {
        self.location.clear();
    }

    pub fn has_location(&self) -> bool {
        self.location.is_some()
    }

    // Param is passed by value, moved
    pub fn set_location(&mut self, v: Point) {
        self.location = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_location(&mut self) -> &mut Point {
        if self.location.is_none() {
            self.location.set_default();
        }
        self.location.as_mut().unwrap()
    }

    // Take field
    pub fn take_location(&mut self) -> Point {
        self.location.take().unwrap_or_else(|| Point::new())
    }

    pub fn get_location(&self) -> &Point {
        self.location.as_ref().unwrap_or_else(|| Point::default_instance())
    }

    fn get_location_for_reflect(&self) -> &::protobuf::SingularPtrField<Point> {
        &self.location
    }

    fn mut_location_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<Point> {
        &mut self.location
    }
}

impl ::protobuf::Message for Feature {
    fn is_initialized(&self) -> bool {
        for v in &self.location {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.name)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.location)?;
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
        if let Some(ref v) = self.location.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.name.is_empty() {
            os.write_string(1, &self.name)?;
        }
        if let Some(ref v) = self.location.as_ref() {
            os.write_tag(2, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
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

impl ::protobuf::MessageStatic for Feature {
    fn new() -> Feature {
        Feature::new()
    }

    fn descriptor_static(_: ::std::option::Option<Feature>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "name",
                    Feature::get_name_for_reflect,
                    Feature::mut_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Point>>(
                    "location",
                    Feature::get_location_for_reflect,
                    Feature::mut_location_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Feature>(
                    "Feature",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Feature {
    fn clear(&mut self) {
        self.clear_name();
        self.clear_location();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Feature {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Feature {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct RouteNote {
    // message fields
    location: ::protobuf::SingularPtrField<Point>,
    pub message: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for RouteNote {}

impl RouteNote {
    pub fn new() -> RouteNote {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static RouteNote {
        static mut instance: ::protobuf::lazy::Lazy<RouteNote> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const RouteNote,
        };
        unsafe {
            instance.get(RouteNote::new)
        }
    }

    // .proto.Point location = 1;

    pub fn clear_location(&mut self) {
        self.location.clear();
    }

    pub fn has_location(&self) -> bool {
        self.location.is_some()
    }

    // Param is passed by value, moved
    pub fn set_location(&mut self, v: Point) {
        self.location = ::protobuf::SingularPtrField::some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_location(&mut self) -> &mut Point {
        if self.location.is_none() {
            self.location.set_default();
        }
        self.location.as_mut().unwrap()
    }

    // Take field
    pub fn take_location(&mut self) -> Point {
        self.location.take().unwrap_or_else(|| Point::new())
    }

    pub fn get_location(&self) -> &Point {
        self.location.as_ref().unwrap_or_else(|| Point::default_instance())
    }

    fn get_location_for_reflect(&self) -> &::protobuf::SingularPtrField<Point> {
        &self.location
    }

    fn mut_location_for_reflect(&mut self) -> &mut ::protobuf::SingularPtrField<Point> {
        &mut self.location
    }

    // string message = 2;

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

impl ::protobuf::Message for RouteNote {
    fn is_initialized(&self) -> bool {
        for v in &self.location {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_message_into(wire_type, is, &mut self.location)?;
                },
                2 => {
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
        if let Some(ref v) = self.location.as_ref() {
            let len = v.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        }
        if !self.message.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.message);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if let Some(ref v) = self.location.as_ref() {
            os.write_tag(1, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
        }
        if !self.message.is_empty() {
            os.write_string(2, &self.message)?;
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

impl ::protobuf::MessageStatic for RouteNote {
    fn new() -> RouteNote {
        RouteNote::new()
    }

    fn descriptor_static(_: ::std::option::Option<RouteNote>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_singular_ptr_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Point>>(
                    "location",
                    RouteNote::get_location_for_reflect,
                    RouteNote::mut_location_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "message",
                    RouteNote::get_message_for_reflect,
                    RouteNote::mut_message_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<RouteNote>(
                    "RouteNote",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for RouteNote {
    fn clear(&mut self) {
        self.clear_location();
        self.clear_message();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for RouteNote {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for RouteNote {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct RouteSummary {
    // message fields
    pub point_count: i32,
    pub feature_count: i32,
    pub distance: i32,
    pub elapsed_time: i32,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for RouteSummary {}

impl RouteSummary {
    pub fn new() -> RouteSummary {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static RouteSummary {
        static mut instance: ::protobuf::lazy::Lazy<RouteSummary> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const RouteSummary,
        };
        unsafe {
            instance.get(RouteSummary::new)
        }
    }

    // int32 point_count = 1;

    pub fn clear_point_count(&mut self) {
        self.point_count = 0;
    }

    // Param is passed by value, moved
    pub fn set_point_count(&mut self, v: i32) {
        self.point_count = v;
    }

    pub fn get_point_count(&self) -> i32 {
        self.point_count
    }

    fn get_point_count_for_reflect(&self) -> &i32 {
        &self.point_count
    }

    fn mut_point_count_for_reflect(&mut self) -> &mut i32 {
        &mut self.point_count
    }

    // int32 feature_count = 2;

    pub fn clear_feature_count(&mut self) {
        self.feature_count = 0;
    }

    // Param is passed by value, moved
    pub fn set_feature_count(&mut self, v: i32) {
        self.feature_count = v;
    }

    pub fn get_feature_count(&self) -> i32 {
        self.feature_count
    }

    fn get_feature_count_for_reflect(&self) -> &i32 {
        &self.feature_count
    }

    fn mut_feature_count_for_reflect(&mut self) -> &mut i32 {
        &mut self.feature_count
    }

    // int32 distance = 3;

    pub fn clear_distance(&mut self) {
        self.distance = 0;
    }

    // Param is passed by value, moved
    pub fn set_distance(&mut self, v: i32) {
        self.distance = v;
    }

    pub fn get_distance(&self) -> i32 {
        self.distance
    }

    fn get_distance_for_reflect(&self) -> &i32 {
        &self.distance
    }

    fn mut_distance_for_reflect(&mut self) -> &mut i32 {
        &mut self.distance
    }

    // int32 elapsed_time = 4;

    pub fn clear_elapsed_time(&mut self) {
        self.elapsed_time = 0;
    }

    // Param is passed by value, moved
    pub fn set_elapsed_time(&mut self, v: i32) {
        self.elapsed_time = v;
    }

    pub fn get_elapsed_time(&self) -> i32 {
        self.elapsed_time
    }

    fn get_elapsed_time_for_reflect(&self) -> &i32 {
        &self.elapsed_time
    }

    fn mut_elapsed_time_for_reflect(&mut self) -> &mut i32 {
        &mut self.elapsed_time
    }
}

impl ::protobuf::Message for RouteSummary {
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
                    let tmp = is.read_int32()?;
                    self.point_count = tmp;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.feature_count = tmp;
                },
                3 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.distance = tmp;
                },
                4 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_int32()?;
                    self.elapsed_time = tmp;
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
        if self.point_count != 0 {
            my_size += ::protobuf::rt::value_size(1, self.point_count, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.feature_count != 0 {
            my_size += ::protobuf::rt::value_size(2, self.feature_count, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.distance != 0 {
            my_size += ::protobuf::rt::value_size(3, self.distance, ::protobuf::wire_format::WireTypeVarint);
        }
        if self.elapsed_time != 0 {
            my_size += ::protobuf::rt::value_size(4, self.elapsed_time, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.point_count != 0 {
            os.write_int32(1, self.point_count)?;
        }
        if self.feature_count != 0 {
            os.write_int32(2, self.feature_count)?;
        }
        if self.distance != 0 {
            os.write_int32(3, self.distance)?;
        }
        if self.elapsed_time != 0 {
            os.write_int32(4, self.elapsed_time)?;
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

impl ::protobuf::MessageStatic for RouteSummary {
    fn new() -> RouteSummary {
        RouteSummary::new()
    }

    fn descriptor_static(_: ::std::option::Option<RouteSummary>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                    "point_count",
                    RouteSummary::get_point_count_for_reflect,
                    RouteSummary::mut_point_count_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                    "feature_count",
                    RouteSummary::get_feature_count_for_reflect,
                    RouteSummary::mut_feature_count_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                    "distance",
                    RouteSummary::get_distance_for_reflect,
                    RouteSummary::mut_distance_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeInt32>(
                    "elapsed_time",
                    RouteSummary::get_elapsed_time_for_reflect,
                    RouteSummary::mut_elapsed_time_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<RouteSummary>(
                    "RouteSummary",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for RouteSummary {
    fn clear(&mut self) {
        self.clear_point_count();
        self.clear_feature_count();
        self.clear_distance();
        self.clear_elapsed_time();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for RouteSummary {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for RouteSummary {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x11route_guide.proto\x12\x05proto\"A\n\x05Point\x12\x1a\n\x08latitude\
    \x18\x01\x20\x01(\x05R\x08latitude\x12\x1c\n\tlongitude\x18\x02\x20\x01(\
    \x05R\tlongitude\"G\n\tRectangle\x12\x1c\n\x02lo\x18\x01\x20\x01(\x0b2\
    \x0c.proto.PointR\x02lo\x12\x1c\n\x02hi\x18\x02\x20\x01(\x0b2\x0c.proto.\
    PointR\x02hi\"G\n\x07Feature\x12\x12\n\x04name\x18\x01\x20\x01(\tR\x04na\
    me\x12(\n\x08location\x18\x02\x20\x01(\x0b2\x0c.proto.PointR\x08location\
    \"O\n\tRouteNote\x12(\n\x08location\x18\x01\x20\x01(\x0b2\x0c.proto.Poin\
    tR\x08location\x12\x18\n\x07message\x18\x02\x20\x01(\tR\x07message\"\x93\
    \x01\n\x0cRouteSummary\x12\x1f\n\x0bpoint_count\x18\x01\x20\x01(\x05R\np\
    ointCount\x12#\n\rfeature_count\x18\x02\x20\x01(\x05R\x0cfeatureCount\
    \x12\x1a\n\x08distance\x18\x03\x20\x01(\x05R\x08distance\x12!\n\x0celaps\
    ed_time\x18\x04\x20\x01(\x05R\x0belapsedTime2\xdd\x01\n\nRouteGuide\x12,\
    \n\nGetFeature\x12\x0c.proto.Point\x1a\x0e.proto.Feature\"\0\x124\n\x0cL\
    istFeatures\x12\x10.proto.Rectangle\x1a\x0e.proto.Feature\"\00\x01\x124\
    \n\x0bRecordRoute\x12\x0c.proto.Point\x1a\x13.proto.RouteSummary\"\0(\
    \x01\x125\n\tRouteChat\x12\x10.proto.RouteNote\x1a\x10.proto.RouteNote\"\
    \0(\x010\x01b\x06proto3\
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

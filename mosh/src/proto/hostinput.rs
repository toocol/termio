// This file is generated by rust-protobuf 3.3.0. Do not edit
// .proto file is parsed by protoc 3.19.4
// @generated

// https://github.com/rust-lang/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(unused_attributes)]
#![cfg_attr(rustfmt, rustfmt::skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unused_results)]
#![allow(unused_mut)]

//! Generated file from `hostinput.proto`

/// Generated files are compatible only with the same version
/// of protobuf runtime.
const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_3_7_1;

// @@protoc_insertion_point(message:HostBuffers.HostMessage)
#[derive(PartialEq,Clone,Default,Debug)]
pub struct HostMessage {
    // message fields
    // @@protoc_insertion_point(field:HostBuffers.HostMessage.instruction)
    pub instruction: ::std::vec::Vec<Instruction>,
    // special fields
    // @@protoc_insertion_point(special_field:HostBuffers.HostMessage.special_fields)
    pub special_fields: ::protobuf::SpecialFields,
}

impl<'a> ::std::default::Default for &'a HostMessage {
    fn default() -> &'a HostMessage {
        <HostMessage as ::protobuf::Message>::default_instance()
    }
}

impl HostMessage {
    pub fn new() -> HostMessage {
        ::std::default::Default::default()
    }

    fn generated_message_descriptor_data() -> ::protobuf::reflect::GeneratedMessageDescriptorData {
        let mut fields = ::std::vec::Vec::with_capacity(1);
        let mut oneofs = ::std::vec::Vec::with_capacity(0);
        fields.push(::protobuf::reflect::rt::v2::make_vec_simpler_accessor::<_, _>(
            "instruction",
            |m: &HostMessage| { &m.instruction },
            |m: &mut HostMessage| { &mut m.instruction },
        ));
        ::protobuf::reflect::GeneratedMessageDescriptorData::new_2::<HostMessage>(
            "HostMessage",
            fields,
            oneofs,
        )
    }
}

impl ::protobuf::Message for HostMessage {
    const NAME: &'static str = "HostMessage";

    fn is_initialized(&self) -> bool {
        for v in &self.instruction {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::Result<()> {
        while let Some(tag) = is.read_raw_tag_or_eof()? {
            match tag {
                10 => {
                    self.instruction.push(is.read_message()?);
                },
                tag => {
                    ::protobuf::rt::read_unknown_or_skip_group(tag, is, self.special_fields.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        for value in &self.instruction {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint64_size(len) + len;
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.special_fields.unknown_fields());
        self.special_fields.cached_size().set(my_size as u32);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::Result<()> {
        for v in &self.instruction {
            ::protobuf::rt::write_message_field_with_cached_size(1, v, os)?;
        };
        os.write_unknown_fields(self.special_fields.unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn special_fields(&self) -> &::protobuf::SpecialFields {
        &self.special_fields
    }

    fn mut_special_fields(&mut self) -> &mut ::protobuf::SpecialFields {
        &mut self.special_fields
    }

    fn new() -> HostMessage {
        HostMessage::new()
    }

    fn clear(&mut self) {
        self.instruction.clear();
        self.special_fields.clear();
    }

    fn default_instance() -> &'static HostMessage {
        static instance: HostMessage = HostMessage {
            instruction: ::std::vec::Vec::new(),
            special_fields: ::protobuf::SpecialFields::new(),
        };
        &instance
    }
}

impl ::protobuf::MessageFull for HostMessage {
    fn descriptor() -> ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::Lazy::new();
        descriptor.get(|| file_descriptor().message_by_package_relative_name("HostMessage").unwrap()).clone()
    }
}

impl ::std::fmt::Display for HostMessage {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for HostMessage {
    type RuntimeType = ::protobuf::reflect::rt::RuntimeTypeMessage<Self>;
}

// @@protoc_insertion_point(message:HostBuffers.Instruction)
#[derive(PartialEq,Clone,Default,Debug)]
pub struct Instruction {
    // special fields
    // @@protoc_insertion_point(special_field:HostBuffers.Instruction.special_fields)
    pub special_fields: ::protobuf::SpecialFields,
}

impl<'a> ::std::default::Default for &'a Instruction {
    fn default() -> &'a Instruction {
        <Instruction as ::protobuf::Message>::default_instance()
    }
}

impl Instruction {
    pub fn new() -> Instruction {
        ::std::default::Default::default()
    }

    fn generated_message_descriptor_data() -> ::protobuf::reflect::GeneratedMessageDescriptorData {
        let mut fields = ::std::vec::Vec::with_capacity(0);
        let mut oneofs = ::std::vec::Vec::with_capacity(0);
        ::protobuf::reflect::GeneratedMessageDescriptorData::new_2::<Instruction>(
            "Instruction",
            fields,
            oneofs,
        )
    }
}

impl ::protobuf::Message for Instruction {
    const NAME: &'static str = "Instruction";

    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::Result<()> {
        while let Some(tag) = is.read_raw_tag_or_eof()? {
            match tag {
                tag => {
                    ::protobuf::rt::read_unknown_or_skip_group(tag, is, self.special_fields.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.special_fields.unknown_fields());
        self.special_fields.cached_size().set(my_size as u32);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::Result<()> {
        os.write_unknown_fields(self.special_fields.unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn special_fields(&self) -> &::protobuf::SpecialFields {
        &self.special_fields
    }

    fn mut_special_fields(&mut self) -> &mut ::protobuf::SpecialFields {
        &mut self.special_fields
    }

    fn new() -> Instruction {
        Instruction::new()
    }

    fn clear(&mut self) {
        self.special_fields.clear();
    }

    fn default_instance() -> &'static Instruction {
        static instance: Instruction = Instruction {
            special_fields: ::protobuf::SpecialFields::new(),
        };
        &instance
    }
}

impl ::protobuf::MessageFull for Instruction {
    fn descriptor() -> ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::Lazy::new();
        descriptor.get(|| file_descriptor().message_by_package_relative_name("Instruction").unwrap()).clone()
    }
}

impl ::std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Instruction {
    type RuntimeType = ::protobuf::reflect::rt::RuntimeTypeMessage<Self>;
}

// @@protoc_insertion_point(message:HostBuffers.HostBytes)
#[derive(PartialEq,Clone,Default,Debug)]
pub struct HostBytes {
    // message fields
    // @@protoc_insertion_point(field:HostBuffers.HostBytes.hoststring)
    pub hoststring: ::std::option::Option<::std::vec::Vec<u8>>,
    // special fields
    // @@protoc_insertion_point(special_field:HostBuffers.HostBytes.special_fields)
    pub special_fields: ::protobuf::SpecialFields,
}

impl<'a> ::std::default::Default for &'a HostBytes {
    fn default() -> &'a HostBytes {
        <HostBytes as ::protobuf::Message>::default_instance()
    }
}

impl HostBytes {
    pub fn new() -> HostBytes {
        ::std::default::Default::default()
    }

    // optional bytes hoststring = 4;

    pub fn hoststring(&self) -> &[u8] {
        match self.hoststring.as_ref() {
            Some(v) => v,
            None => &[],
        }
    }

    pub fn clear_hoststring(&mut self) {
        self.hoststring = ::std::option::Option::None;
    }

    pub fn has_hoststring(&self) -> bool {
        self.hoststring.is_some()
    }

    // Param is passed by value, moved
    pub fn set_hoststring(&mut self, v: ::std::vec::Vec<u8>) {
        self.hoststring = ::std::option::Option::Some(v);
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_hoststring(&mut self) -> &mut ::std::vec::Vec<u8> {
        if self.hoststring.is_none() {
            self.hoststring = ::std::option::Option::Some(::std::vec::Vec::new());
        }
        self.hoststring.as_mut().unwrap()
    }

    // Take field
    pub fn take_hoststring(&mut self) -> ::std::vec::Vec<u8> {
        self.hoststring.take().unwrap_or_else(|| ::std::vec::Vec::new())
    }

    fn generated_message_descriptor_data() -> ::protobuf::reflect::GeneratedMessageDescriptorData {
        let mut fields = ::std::vec::Vec::with_capacity(1);
        let mut oneofs = ::std::vec::Vec::with_capacity(0);
        fields.push(::protobuf::reflect::rt::v2::make_option_accessor::<_, _>(
            "hoststring",
            |m: &HostBytes| { &m.hoststring },
            |m: &mut HostBytes| { &mut m.hoststring },
        ));
        ::protobuf::reflect::GeneratedMessageDescriptorData::new_2::<HostBytes>(
            "HostBytes",
            fields,
            oneofs,
        )
    }
}

impl ::protobuf::Message for HostBytes {
    const NAME: &'static str = "HostBytes";

    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::Result<()> {
        while let Some(tag) = is.read_raw_tag_or_eof()? {
            match tag {
                34 => {
                    self.hoststring = ::std::option::Option::Some(is.read_bytes()?);
                },
                tag => {
                    ::protobuf::rt::read_unknown_or_skip_group(tag, is, self.special_fields.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        if let Some(v) = self.hoststring.as_ref() {
            my_size += ::protobuf::rt::bytes_size(4, &v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.special_fields.unknown_fields());
        self.special_fields.cached_size().set(my_size as u32);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::Result<()> {
        if let Some(v) = self.hoststring.as_ref() {
            os.write_bytes(4, v)?;
        }
        os.write_unknown_fields(self.special_fields.unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn special_fields(&self) -> &::protobuf::SpecialFields {
        &self.special_fields
    }

    fn mut_special_fields(&mut self) -> &mut ::protobuf::SpecialFields {
        &mut self.special_fields
    }

    fn new() -> HostBytes {
        HostBytes::new()
    }

    fn clear(&mut self) {
        self.hoststring = ::std::option::Option::None;
        self.special_fields.clear();
    }

    fn default_instance() -> &'static HostBytes {
        static instance: HostBytes = HostBytes {
            hoststring: ::std::option::Option::None,
            special_fields: ::protobuf::SpecialFields::new(),
        };
        &instance
    }
}

impl ::protobuf::MessageFull for HostBytes {
    fn descriptor() -> ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::Lazy::new();
        descriptor.get(|| file_descriptor().message_by_package_relative_name("HostBytes").unwrap()).clone()
    }
}

impl ::std::fmt::Display for HostBytes {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for HostBytes {
    type RuntimeType = ::protobuf::reflect::rt::RuntimeTypeMessage<Self>;
}

// @@protoc_insertion_point(message:HostBuffers.ResizeMessage)
#[derive(PartialEq,Clone,Default,Debug)]
pub struct ResizeMessage {
    // message fields
    // @@protoc_insertion_point(field:HostBuffers.ResizeMessage.width)
    pub width: ::std::option::Option<i32>,
    // @@protoc_insertion_point(field:HostBuffers.ResizeMessage.height)
    pub height: ::std::option::Option<i32>,
    // special fields
    // @@protoc_insertion_point(special_field:HostBuffers.ResizeMessage.special_fields)
    pub special_fields: ::protobuf::SpecialFields,
}

impl<'a> ::std::default::Default for &'a ResizeMessage {
    fn default() -> &'a ResizeMessage {
        <ResizeMessage as ::protobuf::Message>::default_instance()
    }
}

impl ResizeMessage {
    pub fn new() -> ResizeMessage {
        ::std::default::Default::default()
    }

    // optional int32 width = 5;

    pub fn width(&self) -> i32 {
        self.width.unwrap_or(0)
    }

    pub fn clear_width(&mut self) {
        self.width = ::std::option::Option::None;
    }

    pub fn has_width(&self) -> bool {
        self.width.is_some()
    }

    // Param is passed by value, moved
    pub fn set_width(&mut self, v: i32) {
        self.width = ::std::option::Option::Some(v);
    }

    // optional int32 height = 6;

    pub fn height(&self) -> i32 {
        self.height.unwrap_or(0)
    }

    pub fn clear_height(&mut self) {
        self.height = ::std::option::Option::None;
    }

    pub fn has_height(&self) -> bool {
        self.height.is_some()
    }

    // Param is passed by value, moved
    pub fn set_height(&mut self, v: i32) {
        self.height = ::std::option::Option::Some(v);
    }

    fn generated_message_descriptor_data() -> ::protobuf::reflect::GeneratedMessageDescriptorData {
        let mut fields = ::std::vec::Vec::with_capacity(2);
        let mut oneofs = ::std::vec::Vec::with_capacity(0);
        fields.push(::protobuf::reflect::rt::v2::make_option_accessor::<_, _>(
            "width",
            |m: &ResizeMessage| { &m.width },
            |m: &mut ResizeMessage| { &mut m.width },
        ));
        fields.push(::protobuf::reflect::rt::v2::make_option_accessor::<_, _>(
            "height",
            |m: &ResizeMessage| { &m.height },
            |m: &mut ResizeMessage| { &mut m.height },
        ));
        ::protobuf::reflect::GeneratedMessageDescriptorData::new_2::<ResizeMessage>(
            "ResizeMessage",
            fields,
            oneofs,
        )
    }
}

impl ::protobuf::Message for ResizeMessage {
    const NAME: &'static str = "ResizeMessage";

    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::Result<()> {
        while let Some(tag) = is.read_raw_tag_or_eof()? {
            match tag {
                40 => {
                    self.width = ::std::option::Option::Some(is.read_int32()?);
                },
                48 => {
                    self.height = ::std::option::Option::Some(is.read_int32()?);
                },
                tag => {
                    ::protobuf::rt::read_unknown_or_skip_group(tag, is, self.special_fields.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        if let Some(v) = self.width {
            my_size += ::protobuf::rt::int32_size(5, v);
        }
        if let Some(v) = self.height {
            my_size += ::protobuf::rt::int32_size(6, v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.special_fields.unknown_fields());
        self.special_fields.cached_size().set(my_size as u32);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::Result<()> {
        if let Some(v) = self.width {
            os.write_int32(5, v)?;
        }
        if let Some(v) = self.height {
            os.write_int32(6, v)?;
        }
        os.write_unknown_fields(self.special_fields.unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn special_fields(&self) -> &::protobuf::SpecialFields {
        &self.special_fields
    }

    fn mut_special_fields(&mut self) -> &mut ::protobuf::SpecialFields {
        &mut self.special_fields
    }

    fn new() -> ResizeMessage {
        ResizeMessage::new()
    }

    fn clear(&mut self) {
        self.width = ::std::option::Option::None;
        self.height = ::std::option::Option::None;
        self.special_fields.clear();
    }

    fn default_instance() -> &'static ResizeMessage {
        static instance: ResizeMessage = ResizeMessage {
            width: ::std::option::Option::None,
            height: ::std::option::Option::None,
            special_fields: ::protobuf::SpecialFields::new(),
        };
        &instance
    }
}

impl ::protobuf::MessageFull for ResizeMessage {
    fn descriptor() -> ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::Lazy::new();
        descriptor.get(|| file_descriptor().message_by_package_relative_name("ResizeMessage").unwrap()).clone()
    }
}

impl ::std::fmt::Display for ResizeMessage {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ResizeMessage {
    type RuntimeType = ::protobuf::reflect::rt::RuntimeTypeMessage<Self>;
}

// @@protoc_insertion_point(message:HostBuffers.EchoAck)
#[derive(PartialEq,Clone,Default,Debug)]
pub struct EchoAck {
    // message fields
    // @@protoc_insertion_point(field:HostBuffers.EchoAck.echo_ack_num)
    pub echo_ack_num: ::std::option::Option<u64>,
    // special fields
    // @@protoc_insertion_point(special_field:HostBuffers.EchoAck.special_fields)
    pub special_fields: ::protobuf::SpecialFields,
}

impl<'a> ::std::default::Default for &'a EchoAck {
    fn default() -> &'a EchoAck {
        <EchoAck as ::protobuf::Message>::default_instance()
    }
}

impl EchoAck {
    pub fn new() -> EchoAck {
        ::std::default::Default::default()
    }

    // optional uint64 echo_ack_num = 8;

    pub fn echo_ack_num(&self) -> u64 {
        self.echo_ack_num.unwrap_or(0)
    }

    pub fn clear_echo_ack_num(&mut self) {
        self.echo_ack_num = ::std::option::Option::None;
    }

    pub fn has_echo_ack_num(&self) -> bool {
        self.echo_ack_num.is_some()
    }

    // Param is passed by value, moved
    pub fn set_echo_ack_num(&mut self, v: u64) {
        self.echo_ack_num = ::std::option::Option::Some(v);
    }

    fn generated_message_descriptor_data() -> ::protobuf::reflect::GeneratedMessageDescriptorData {
        let mut fields = ::std::vec::Vec::with_capacity(1);
        let mut oneofs = ::std::vec::Vec::with_capacity(0);
        fields.push(::protobuf::reflect::rt::v2::make_option_accessor::<_, _>(
            "echo_ack_num",
            |m: &EchoAck| { &m.echo_ack_num },
            |m: &mut EchoAck| { &mut m.echo_ack_num },
        ));
        ::protobuf::reflect::GeneratedMessageDescriptorData::new_2::<EchoAck>(
            "EchoAck",
            fields,
            oneofs,
        )
    }
}

impl ::protobuf::Message for EchoAck {
    const NAME: &'static str = "EchoAck";

    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream<'_>) -> ::protobuf::Result<()> {
        while let Some(tag) = is.read_raw_tag_or_eof()? {
            match tag {
                64 => {
                    self.echo_ack_num = ::std::option::Option::Some(is.read_uint64()?);
                },
                tag => {
                    ::protobuf::rt::read_unknown_or_skip_group(tag, is, self.special_fields.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u64 {
        let mut my_size = 0;
        if let Some(v) = self.echo_ack_num {
            my_size += ::protobuf::rt::uint64_size(8, v);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.special_fields.unknown_fields());
        self.special_fields.cached_size().set(my_size as u32);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream<'_>) -> ::protobuf::Result<()> {
        if let Some(v) = self.echo_ack_num {
            os.write_uint64(8, v)?;
        }
        os.write_unknown_fields(self.special_fields.unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn special_fields(&self) -> &::protobuf::SpecialFields {
        &self.special_fields
    }

    fn mut_special_fields(&mut self) -> &mut ::protobuf::SpecialFields {
        &mut self.special_fields
    }

    fn new() -> EchoAck {
        EchoAck::new()
    }

    fn clear(&mut self) {
        self.echo_ack_num = ::std::option::Option::None;
        self.special_fields.clear();
    }

    fn default_instance() -> &'static EchoAck {
        static instance: EchoAck = EchoAck {
            echo_ack_num: ::std::option::Option::None,
            special_fields: ::protobuf::SpecialFields::new(),
        };
        &instance
    }
}

impl ::protobuf::MessageFull for EchoAck {
    fn descriptor() -> ::protobuf::reflect::MessageDescriptor {
        static descriptor: ::protobuf::rt::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::rt::Lazy::new();
        descriptor.get(|| file_descriptor().message_by_package_relative_name("EchoAck").unwrap()).clone()
    }
}

impl ::std::fmt::Display for EchoAck {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for EchoAck {
    type RuntimeType = ::protobuf::reflect::rt::RuntimeTypeMessage<Self>;
}

/// Extension fields
pub mod exts {

    pub const hostbytes: ::protobuf::ext::ExtFieldOptional<super::Instruction, super::HostBytes> = ::protobuf::ext::ExtFieldOptional::new(2, ::protobuf::descriptor::field_descriptor_proto::Type::TYPE_MESSAGE);

    pub const resize: ::protobuf::ext::ExtFieldOptional<super::Instruction, super::ResizeMessage> = ::protobuf::ext::ExtFieldOptional::new(3, ::protobuf::descriptor::field_descriptor_proto::Type::TYPE_MESSAGE);

    pub const echoack: ::protobuf::ext::ExtFieldOptional<super::Instruction, super::EchoAck> = ::protobuf::ext::ExtFieldOptional::new(7, ::protobuf::descriptor::field_descriptor_proto::Type::TYPE_MESSAGE);
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x0fhostinput.proto\x12\x0bHostBuffers\"I\n\x0bHostMessage\x12:\n\x0bi\
    nstruction\x18\x01\x20\x03(\x0b2\x18.HostBuffers.InstructionR\x0binstruc\
    tion\"\x17\n\x0bInstruction*\x08\x08\x02\x10\x80\x80\x80\x80\x02\"+\n\tH\
    ostBytes\x12\x1e\n\nhoststring\x18\x04\x20\x01(\x0cR\nhoststring\"=\n\rR\
    esizeMessage\x12\x14\n\x05width\x18\x05\x20\x01(\x05R\x05width\x12\x16\n\
    \x06height\x18\x06\x20\x01(\x05R\x06height\"+\n\x07EchoAck\x12\x20\n\x0c\
    echo_ack_num\x18\x08\x20\x01(\x04R\nechoAckNum:N\n\thostbytes\x18\x02\
    \x20\x01(\x0b2\x16.HostBuffers.HostBytes\x12\x18.HostBuffers.Instruction\
    R\thostbytes:L\n\x06resize\x18\x03\x20\x01(\x0b2\x1a.HostBuffers.ResizeM\
    essage\x12\x18.HostBuffers.InstructionR\x06resize:H\n\x07echoack\x18\x07\
    \x20\x01(\x0b2\x14.HostBuffers.EchoAck\x12\x18.HostBuffers.InstructionR\
    \x07echoackB\x02H\x03\
";

/// `FileDescriptorProto` object which was a source for this generated file
fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    static file_descriptor_proto_lazy: ::protobuf::rt::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::rt::Lazy::new();
    file_descriptor_proto_lazy.get(|| {
        ::protobuf::Message::parse_from_bytes(file_descriptor_proto_data).unwrap()
    })
}

/// `FileDescriptor` object which allows dynamic access to files
pub fn file_descriptor() -> &'static ::protobuf::reflect::FileDescriptor {
    static generated_file_descriptor_lazy: ::protobuf::rt::Lazy<::protobuf::reflect::GeneratedFileDescriptor> = ::protobuf::rt::Lazy::new();
    static file_descriptor: ::protobuf::rt::Lazy<::protobuf::reflect::FileDescriptor> = ::protobuf::rt::Lazy::new();
    file_descriptor.get(|| {
        let generated_file_descriptor = generated_file_descriptor_lazy.get(|| {
            let mut deps = ::std::vec::Vec::with_capacity(0);
            let mut messages = ::std::vec::Vec::with_capacity(5);
            messages.push(HostMessage::generated_message_descriptor_data());
            messages.push(Instruction::generated_message_descriptor_data());
            messages.push(HostBytes::generated_message_descriptor_data());
            messages.push(ResizeMessage::generated_message_descriptor_data());
            messages.push(EchoAck::generated_message_descriptor_data());
            let mut enums = ::std::vec::Vec::with_capacity(0);
            ::protobuf::reflect::GeneratedFileDescriptor::new_generated(
                file_descriptor_proto(),
                deps,
                messages,
                enums,
            )
        });
        ::protobuf::reflect::FileDescriptor::new_generated_2(generated_file_descriptor)
    })
}

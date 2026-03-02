pub mod cpp;
pub mod python;
pub mod rust;

use crate::ast::PrimitiveType;

pub fn rust_type(ty: PrimitiveType) -> &'static str {
    match ty {
        PrimitiveType::Bool => "bool",
        PrimitiveType::Byte | PrimitiveType::Char | PrimitiveType::Uint8 => "u8",
        PrimitiveType::Int8 => "i8",
        PrimitiveType::Uint16 => "u16",
        PrimitiveType::Int16 => "i16",
        PrimitiveType::Uint32 => "u32",
        PrimitiveType::Int32 => "i32",
        PrimitiveType::Uint64 => "u64",
        PrimitiveType::Int64 => "i64",
        PrimitiveType::Float32 => "f32",
        PrimitiveType::Float64 => "f64",
    }
}

pub fn cpp_type(ty: PrimitiveType) -> &'static str {
    match ty {
        PrimitiveType::Bool => "bool",
        PrimitiveType::Byte | PrimitiveType::Char | PrimitiveType::Uint8 => "uint8_t",
        PrimitiveType::Int8 => "int8_t",
        PrimitiveType::Uint16 => "uint16_t",
        PrimitiveType::Int16 => "int16_t",
        PrimitiveType::Uint32 => "uint32_t",
        PrimitiveType::Int32 => "int32_t",
        PrimitiveType::Uint64 => "uint64_t",
        PrimitiveType::Int64 => "int64_t",
        PrimitiveType::Float32 => "float",
        PrimitiveType::Float64 => "double",
    }
}

/// ctypes field type for a primitive (used in `_fields_` list).
pub fn ctypes_type(ty: PrimitiveType) -> &'static str {
    match ty {
        PrimitiveType::Bool => "ctypes.c_bool",
        PrimitiveType::Byte | PrimitiveType::Char | PrimitiveType::Uint8 => "ctypes.c_uint8",
        PrimitiveType::Int8 => "ctypes.c_int8",
        PrimitiveType::Uint16 => "ctypes.c_uint16",
        PrimitiveType::Int16 => "ctypes.c_int16",
        PrimitiveType::Uint32 => "ctypes.c_uint32",
        PrimitiveType::Int32 => "ctypes.c_int32",
        PrimitiveType::Uint64 => "ctypes.c_uint64",
        PrimitiveType::Int64 => "ctypes.c_int64",
        PrimitiveType::Float32 => "ctypes.c_float",
        PrimitiveType::Float64 => "ctypes.c_double",
    }
}

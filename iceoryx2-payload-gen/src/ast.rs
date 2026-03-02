/// A parsed ROS .msg file.
#[derive(Debug, Clone)]
pub struct Message {
    pub name: String,
    pub fields: Vec<Field>,
    pub constants: Vec<Constant>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: FieldType,
}

#[derive(Debug, Clone)]
pub enum FieldType {
    Primitive(PrimitiveType),
    FixedArray(PrimitiveType, usize),
    /// ROS2 bounded string `string<=N` — maps to `StaticString<N>` (zero-copy safe).
    BoundedString(usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrimitiveType {
    Bool,
    Byte,
    Char,
    Int8,
    Uint8,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Int64,
    Uint64,
    Float32,
    Float64,
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub name: String,
    pub ty: PrimitiveType,
    pub value: String,
}

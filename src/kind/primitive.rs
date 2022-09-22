use std::fmt;
use pod::Pod;

use crate::{
    access::{self, Trace, Indirection},
    kind::Kind,
    mem_ribbon::MemRibbon,
};

use super::CType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Primitive {
    Bool, Char,
    U8, U16, U32, U64,
    I8, I16, I32, I64,
             F32, F64,
    Size,
}

impl Primitive {
    pub fn parse_at(&self, ribbon: &MemRibbon, address: usize) -> Option<PrimValue> {
        let mut value = PrimValue::from(*self);

        let dest_slice = value.bytes_mut();
        let source_slice = ribbon.bytes_at(
            address - ribbon.base_adr,
            dest_slice.len(),
        )?;

        for (dst, src) in dest_slice.iter_mut().zip(source_slice) {
            *dst = src;
        }

        Some(value)
    }
}

impl<'kind> CType<'kind> for Primitive {
    fn description(&self) -> &dyn fmt::Display {
        self
    }

    fn size_of(&self) -> u16 {
        use Primitive::*;
        match self {
            Bool => 1,
            Char => 1,
            U8   => 1, U16 => 2, U32 => 4, U64 => 8,
            I8   => 1, I16 => 2, I32 => 4, I64 => 8,
            F32  => 4, F64 => 8,
            Size => 4,
        }
    }

    fn align_of(&self) -> u16 {
        self.size_of()
    }

    fn access_with(&self, indirection: Indirection, trace: Trace<'kind>) -> access::Result<'kind> {
        Err(access::Error::at(
            trace.field_name.clone(),
            access::ErrorKind::Operation { op: indirection.operator(), kind: Kind::Primitive(*self) },
        ))
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Primitive::*;

        f.write_str(match self {
            Bool => "bool",
            Char => "char",
            U8   => "uint8_t", U16 => "uint16_t", U32 => "uint32_t", U64 => "uint64_t",
            I8   => "int8_t",  I16 => "int16_t",  I32 => "int32_t",  I64 => "int64_t",
            F32  => "float",   F64 => "double",
            Size => "size_t",
        })
    }

}

pub enum PrimValue {
    Bool(u8),
    Char(u8),
    U8(u8), U16(u16), U32(u32), U64(u64),
    I8(i8), I16(i16), I32(i32), I64(i64),
                      F32(f32), F64(f64),
    Size(u32),
}

impl PrimValue {
    pub fn bytes(&self) -> &[u8] {
        use PrimValue::*;

        match self {
            Bool(x) | Char(x) | U8(x) => x.as_bytes(),
            U32(x) | Size(x) => x.as_bytes(),
            U16(x) => x.as_bytes(),
            U64(x) => x.as_bytes(),
            I8(x)  => x.as_bytes(),
            I16(x) => x.as_bytes(),
            I32(x) => x.as_bytes(),
            I64(x) => x.as_bytes(),
            F32(x) => x.as_bytes(),
            F64(x) => x.as_bytes(),
        }
    }

    fn bytes_mut(&mut self) -> &mut [u8] {
        use PrimValue::*;

        match self {
            Bool(x) | Char(x) | U8(x) => x.as_bytes_mut(),
            U32(x) | Size(x) => x.as_bytes_mut(),
            U16(x) => x.as_bytes_mut(),
            U64(x) => x.as_bytes_mut(),
            I8(x)  => x.as_bytes_mut(),
            I16(x) => x.as_bytes_mut(),
            I32(x) => x.as_bytes_mut(),
            I64(x) => x.as_bytes_mut(),
            F32(x) => x.as_bytes_mut(),
            F64(x) => x.as_bytes_mut(),
        }
    }
}

impl fmt::Display for PrimValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PrimValue::*;

        match *self {
            Bool(x) => write!(f, "{}", x != 0),
            Char(x) => write!(f, "{:?}", x as char),
            U8(x)   => write!(f, "{}", x),
            U16(x)  => write!(f, "{}", x),
            U32(x)  => write!(f, "{}", x),
            U64(x)  => write!(f, "{}", x),
            I8(x)   => write!(f, "{}", x),
            I16(x)  => write!(f, "{}", x),
            I32(x)  => write!(f, "{}", x),
            I64(x)  => write!(f, "{}", x),
            F32(x)  => write!(f, "{}", x),
            F64(x)  => write!(f, "{}", x),
            Size(x) => write!(f, "{}", x),
        }
    }
}

impl From<Primitive> for PrimValue {
    fn from(prim: Primitive) -> Self {
        use PrimValue::*;
        match prim {
            Primitive::Bool => Bool(0),
            Primitive::Char => Char(0),
            Primitive::U8   => U8  (0),
            Primitive::U16  => U16 (0),
            Primitive::U32  => U32 (0),
            Primitive::U64  => U64 (0),
            Primitive::I8   => I8  (0),
            Primitive::I16  => I16 (0),
            Primitive::I32  => I32 (0),
            Primitive::I64  => I64 (0),
            Primitive::F32  => F32 (0.0),
            Primitive::F64  => F64 (0.0),
            Primitive::Size => Size(0),
        }
    }
}

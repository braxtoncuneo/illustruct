#![allow(dead_code)]

use std::{fmt::{Display, self}, cell::RefCell};
use pod::Pod;

use crate::{
    access::{AccessUnit, AccessTrace, PlaceValue, Error, ErrorKind},
    mem_ribbon::MemRibbon
};

pub mod reference;
pub mod composite;
pub mod array;

use reference::Reference;
use composite::Composite;

use self::{composite::{CompositeMode, Field}, reference::ReferenceMode, array::Array};

pub enum PrimValue {
    Bool(bool),
    Char(u8),
    U8(u8), U16(u16), U32(u32), U64(u64),
    I8(i8), I16(i16), I32(i32), I64(i64),
                      F32(f32), F64(f64),
    Size(u32),
}


impl fmt::Display for PrimValue {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimValue::Bool(x) => write!(f,"{}",x),
            PrimValue::Char(x) => write!(f,"{:?}",*x as char),
            PrimValue::U8(x)   => write!(f,"{}",x),
            PrimValue::U16(x)  => write!(f,"{}",x),
            PrimValue::U32(x)  => write!(f,"{}",x),
            PrimValue::U64(x)  => write!(f,"{}",x),
            PrimValue::I8(x)   => write!(f,"{}",x),
            PrimValue::I16(x)  => write!(f,"{}",x),
            PrimValue::I32(x)  => write!(f,"{}",x),
            PrimValue::I64(x)  => write!(f,"{}",x),
            PrimValue::F32(x)  => write!(f,"{}",x),
            PrimValue::F64(x)  => write!(f,"{}",x),
            PrimValue::Size(x) => write!(f,"{}",x),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Primitive {
    Bool,
    Char,
    U8, U16, U32, U64,
    I8, I16, I32, I64,
             F32, F64,
    Size,
}

impl Primitive {
    pub fn parse_at(&self, ribbon: &MemRibbon, address: usize) -> Option<PrimValue> {
        use Primitive::*;

        let mut value = match self {
            Bool => PrimValue::Bool(false),
            Char => PrimValue::Char(0),
            U8   => PrimValue::U8  (0),
            U16  => PrimValue::U16 (0),
            U32  => PrimValue::U32 (0),
            U64  => PrimValue::U64 (0),
            I8   => PrimValue::I8  (0),
            I16  => PrimValue::I16 (0),
            I32  => PrimValue::I32 (0),
            I64  => PrimValue::I64 (0),
            F32  => PrimValue::F32 (0.0),
            F64  => PrimValue::F64 (0.0),
            Size => PrimValue::Size(0),
        };

        let mut bool_u8: u8 = 0;

        let dst_slice = match &mut value {
            PrimValue::Bool(_) => bool_u8.as_bytes_mut(),
            PrimValue::Char(x) => x.as_bytes_mut(),
            PrimValue::U8  (x) => x.as_bytes_mut(),
            PrimValue::U16 (x) => x.as_bytes_mut(),
            PrimValue::U32 (x) => x.as_bytes_mut(),
            PrimValue::U64 (x) => x.as_bytes_mut(),
            PrimValue::I8  (x) => x.as_bytes_mut(),
            PrimValue::I16 (x) => x.as_bytes_mut(),
            PrimValue::I32 (x) => x.as_bytes_mut(),
            PrimValue::I64 (x) => x.as_bytes_mut(),
            PrimValue::F32 (x) => x.as_bytes_mut(),
            PrimValue::F64 (x) => x.as_bytes_mut(),
            PrimValue::Size(x) => x.as_bytes_mut(),
        };

        let address = address - ribbon.base_adr;
        let bytes = ribbon.bytes_at(address, dst_slice.len())?;

        for (dst, src) in dst_slice.iter_mut().zip(bytes) {
            *dst = src;
        }

        if let PrimValue::Bool(slot) = &mut value {
            *slot = bool_u8 != 0;
        }

        Some(value)
    }

    pub(crate) fn size_of(&self) -> u16 {
        use Primitive::*;
        match self {
            Bool => 1,
            Char => 1,
            U8  =>  1, U16 => 2, U32 => 4, U64 => 8,
            I8  =>  1, I16 => 2, I32 => 4, I64 => 8,
            F32 =>  4, F64 => 8,
            Size => 4,
        }
    }

    fn align_of(&self) -> u16 {
        self.size_of()
    }

    fn access<'kind>(&'kind self, unit: &AccessUnit, trace: &AccessTrace<'kind>) -> Result<PlaceValue<'kind>, Error> {
        Err(Error::at(
            trace.field_name.clone(),
            ErrorKind::Operation { op: unit.op_str(), kind: Kind::Primitive(*self) },
        ))
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Primitive::*;

        f.write_str(match self {
            Bool =>    "bool",
            Char =>    "char",
            U8   => "uint8_t", U16 => "uint16_t", U32 => "uint32_t", U64 => "uint64_t",
            I8   =>  "int8_t", I16 =>  "int16_t", I32 =>  "int32_t", I64 =>  "int64_t",
            F32  =>   "float", F64 =>   "double",
            Size =>  "size_t",
        })
    }

}


#[derive(Clone)]
pub struct Alias<'kind> {
    name: String,
    kind: &'kind Kind<'kind>
}



#[derive(Clone)]
pub enum Kind<'kind> {
    Primitive(Primitive),
    Reference(Reference<'kind>),
    Composite(Composite<'kind>),
    Array    (Array<'kind>),
    Alias    (Alias<'kind>),
}

impl<'kind> Kind<'kind> {
    pub fn prim(value: Primitive) -> Self {
        Kind::Primitive(value)
    }

    pub fn refr(mode: ReferenceMode, kind: &'kind Kind<'kind>) -> Self {
        Kind::Reference(Reference { mode, kind })
    }

    pub fn comp<T: ToString>(name: T, mode: CompositeMode, fields: Vec<Field<'kind>>) -> Self {
        Kind::Composite(Composite {
            name: name.to_string(),
            mode,
            fields: RefCell::new(fields)
        })
    }

    pub fn array(kind: &'kind Kind<'kind>, size: usize) -> Self {
        Kind::Array(Array{kind,size})
    }

    pub fn alias<T: ToString>(name: T, kind: &'kind Kind<'kind>) -> Self {
        Kind::Alias(Alias{name: name.to_string() ,kind})
    }


    pub fn category(&self) -> &dyn Display {
        match self {
            Kind::Primitive(x) => x,
            Kind::Reference(x) => &x.mode,
            Kind::Composite(x) => &x.mode,
            Kind::Array    (x) => x,
            Kind::Alias(x)     => x.kind.category(),
        }
    }

    pub fn size_of(&self) -> u16 {
        match self {
            Kind::Primitive(x) => x.size_of(),
            Kind::Reference(x) => x.size_of(),
            Kind::Composite(x) => x.size_of(),
            Kind::Array(x)     => x.size_of(),
            Kind::Alias(x)     => x.kind.size_of(),
        }
    }

    pub fn align_of(&self) -> u16 {
        match self {
            Kind::Primitive(x) => x.align_of(),
            Kind::Reference(x) => x.align_of(),
            Kind::Composite(x) => x.align_of(),
            Kind::Array    (x) => x.align_of(),
            Kind::Alias(x)     => x.kind.align_of(),
        }
    }


    pub fn align_pad(&self,offset:u16) -> u16 {
        let align = self.align_of();
        let remainder = offset % align;
        if remainder == 0 {
            0
        } else {
            align - remainder
        }
    }

    pub fn base_fields(&self, address: &mut usize) -> Vec<(usize, Primitive)> {
        *address += self.align_pad(*address as u16) as usize;
        match self {
            Kind::Primitive(prim) => vec![(*address,*prim)],
            Kind::Reference(refr) => vec![(*address,Primitive::Size)],
            Kind::Composite(comp) => comp.base_fields(address),
            Kind::Array(x)        => x.base_fields(address),
            Kind::Alias(x)        => x.kind.base_fields(address),
        }
    }

    fn empty_access(&'kind self, trace: &AccessTrace<'kind>) -> Result<PlaceValue<'kind>, Error> {
        let refr = match self {
            Kind::Reference(refr) => refr,
            _ => return Ok(PlaceValue{ kind: self, address: trace.address })
        };

        let field_name = trace.field_name.clone();

        match Primitive::Size.parse_at(trace.ribbon, trace.address) {
            Some(PrimValue::Size(address)) => Ok(PlaceValue {
                kind: refr.kind,
                address: address as usize
            }),
            None => Err(Error::at(field_name, ErrorKind::Deref { old_addr: trace.address })),
            _ => unreachable!(),
        }
    }

    pub fn access(&'kind self, trace: &mut AccessTrace<'kind>) -> Result<PlaceValue<'kind>, Error> {

        if let Kind::Alias(x) = self {
            return x.kind.access(trace);
        }

        let unit = match trace.access.pop_front() {
            Some(unit) => unit,
            None => return self.empty_access(trace),
        };

        match self {
            Kind::Primitive(x) => x.access(&unit, trace),
            Kind::Reference(x) => x.access(&unit, trace),
            Kind::Composite(x) => x.access(&unit, trace),
            Kind::Array    (x) => x.access(&unit, trace),
            Kind::Alias    (_) => unreachable!(),
        }.map_err(|err|
            err.with_context(self.category(), trace.field_name.as_str())
        )
    }

    pub fn into_ribbon(&'kind self) -> MemRibbon<'kind> {
        let mut result = MemRibbon::new(0);
        let mut fields = Vec::new();
        fields.push(Field::anon(self));
        result.span("",fields);
        result
    }

    fn get_fields(&'kind self) -> Option<&'kind RefCell<Vec<Field<'kind>>>> {
        match self {
            Kind::Composite(comp) => Some(&comp.fields),
            _ => panic!("Cannot treat kind '{}' as composite",self),
        }
    }

    pub fn add_field<T:ToString>(&'kind self, name:T, kind: &'kind Kind<'kind>) {
        self.get_fields().unwrap().borrow_mut().push(Field{
            name: Some(name.to_string()),
            kind,
        })
    }

}

impl fmt::Display for Kind<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Primitive(primitive) => write!(f, "{primitive}"),
            Kind::Reference(reference) => write!(f, "{reference}"),
            Kind::Composite(composite) => write!(f, "{composite}"),
            Kind::Array    (array)     => write!(f, "{array}"),
            Kind::Alias    (alias)     => write!(f, "{}",alias.name ),
        }
    }
}

impl From<Primitive> for Kind<'_> {
    fn from(primitive: Primitive) -> Self {
        Self::Primitive(primitive)
    }
}
#![allow(dead_code)]

use std::{fmt::{Display, self}, cell::RefCell};

use crate::{
    access::{AccessTrace, PlaceValue, Error, ErrorKind},
    mem_ribbon::MemRibbon
};

pub mod reference;
pub mod composite;
pub mod array;
pub mod primitive;

use reference::Reference;
use composite::Composite;
use primitive::{Primitive, PrimValue};

use self::{composite::{CompositeMode, Field}, reference::ReferenceMode, array::Array};

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
    Array(Array<'kind>),
    Alias(Alias<'kind>),
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
            Kind::Array(x) => x,
            Kind::Alias(x) => x.kind.category(),
        }
    }

    pub fn size_of(&self) -> u16 {
        match self {
            Kind::Primitive(x) => x.size_of(),
            Kind::Reference(x) => x.size_of(),
            Kind::Composite(x) => x.size_of(),
            Kind::Array(x)  => x.size_of(),
            Kind::Alias(x) => x.kind.size_of(),
        }
    }

    pub fn align_of(&self) -> u16 {
        match self {
            Kind::Primitive(x) => x.align_of(),
            Kind::Reference(x) => x.align_of(),
            Kind::Composite(x) => x.align_of(),
            Kind::Array(x) => x.align_of(),
            Kind::Alias(x) => x.kind.align_of(),
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
            Kind::Primitive(prim) => vec![(*address, *prim)],
            Kind::Reference(_) => vec![(*address, Primitive::Size)],
            Kind::Composite(comp) => comp.base_fields(address),
            Kind::Array(x) => x.base_fields(address),
            Kind::Alias(x) => x.kind.base_fields(address),
        }
    }

    pub fn empty_access(&'kind self, trace: &AccessTrace<'kind>) -> Result<PlaceValue<'kind>, Error> {
        let refr = match self {
            Kind::Reference(refr) => refr,
            _ => return Ok(PlaceValue{ kind: self, address: trace.address })
        };

        match Primitive::Size.parse_at(trace.ribbon, trace.address) {
            Some(PrimValue::Size(address)) => Ok(PlaceValue {
                kind: refr.kind,
                address: address as usize
            }),
            None => Err(Error::at(
                trace.field_name.clone(),
                ErrorKind::Deref { old_addr: trace.address }
            )),
            Some(_) => unreachable!(),
        }
    }

    pub fn access(&'kind self, trace: &mut AccessTrace<'kind>) -> Result<PlaceValue<'kind>, Error> {
        use Kind::*;

        if let Alias(x) = self {
            return x.kind.access(trace);
        }

        let unit = match trace.access.pop_front() {
            Some(unit) => unit,
            None => return self.empty_access(trace),
        };

        let value = match self {
            Primitive(x) => x.access(&unit, trace),
            Reference(x) => x.access(&unit, trace),
            Composite(x) => x.access(&unit, trace),
            Array(x) => x.access(&unit, trace),
            Alias(_) => unreachable!(),
        };
        
        value.map_err(|err| err.with_context(
            self.category(),
            trace.field_name.as_str(),
        ))
    }

    pub fn into_ribbon(&'kind self) -> MemRibbon<'kind> {
        let mut result = MemRibbon::new(0);
        let fields = vec![Field::anon(self)];
        result.span("", fields);
        result
    }

    pub fn get_fields(&'kind self) -> Option<&'kind RefCell<Vec<Field<'kind>>>> {
        match self {
            Kind::Composite(comp) => Some(&comp.fields),
            _ => panic!("Cannot treat kind '{}' as composite",self),
        }
    }

    pub fn add_field<T:ToString>(&'kind self, name:T, kind: &'kind Kind<'kind>) {
        self.get_fields().unwrap().borrow_mut().push(Field::new(name, kind))
    }
}

impl fmt::Display for Kind<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Kind::*;
        match self {
            Primitive(primitive) => write!(f, "{primitive}"),
            Reference(reference) => write!(f, "{reference}"),
            Composite(composite) => write!(f, "{composite}"),
            Array(array) => write!(f, "{array}"),
            Alias(alias) => write!(f, "{}", alias.name ),
        }
    }
}

impl From<Primitive> for Kind<'_> {
    fn from(primitive: Primitive) -> Self {
        Self::Primitive(primitive)
    }
}
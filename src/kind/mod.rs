use std::{fmt::{Display, self}, cell::RefCell};
use enum_dispatch::enum_dispatch;

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

use self::{composite::Field, array::Array};

#[enum_dispatch]
pub trait CType: Sized + Display {
    fn description(&self) -> &dyn Display;
    fn size_of(&self) -> u16;
    fn align_of(&self) -> u16;
    fn display(&self) -> &dyn Display { self }
}

#[derive(Clone)]
#[enum_dispatch(CType)]
pub enum Kind<'kind> {
    Primitive(Primitive),
    Reference(Reference<'kind>),
    Composite(Composite<'kind>),
    Array(Array<'kind>),
    Alias(Alias<'kind>),
}

impl<'kind> Kind<'kind> {
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
            self.description(),
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
        self.display().fmt(f)
    }
}

#[derive(Clone)]
pub struct Alias<'kind> {
    name: String,
    kind: &'kind Kind<'kind>
}

impl<'kind> Alias<'kind> {
    pub fn new(name: impl ToString, kind: &'kind Kind<'kind>) -> Self {
        Self {
            name: name.to_string(),
            kind,
        }
    }
}

impl CType for Alias<'_> {
    fn description(&self) -> &dyn Display {
        self.kind.description()
    }

    fn size_of(&self) -> u16 {
        self.kind.size_of()
    }

    fn align_of(&self) -> u16 {
        self.kind.align_of()
    }
}

impl Display for Alias<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name.as_str())
    }
}

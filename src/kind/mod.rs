use std::fmt::{Display, self};
use enum_dispatch::enum_dispatch;

use crate::{
    access::{self, Indirection, Trace, PlaceValue},
    mem_ribbon::MemRibbon,
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
pub trait CType<'kind>: Sized + Display {
    fn description(&self) -> &dyn Display;
    fn size_of(&self) -> u16;
    fn align_of(&self) -> u16;
    fn access_with(&self, indirection: Indirection, trace: Trace<'kind>) -> access::Result<'kind>;
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
    pub fn align_pad(&self, offset: u16) -> u16 {
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

    pub fn get_place_value(&'kind self, trace: Trace<'kind>) -> access::Result<'kind> {
        let place_value = if let Kind::Reference(refr) = self {
            let parsed_size = Primitive::Size
                .parse_at(trace.ribbon, trace.address)
                .ok_or_else(|| access::Error::at(
                    trace.field_name.clone(),
                    access::ErrorKind::Deref { old_addr: trace.address }
                ))?;

            let address = match parsed_size {
                PrimValue::Size(address) => address,
                _ => unreachable!(),
            };

            PlaceValue {
                kind: refr.kind,
                address: address as usize
            }
        } else {
            PlaceValue{
                kind: self,
                address: trace.address,
            }
        };

        Ok(place_value)
    }

    pub fn access(&'kind self, mut trace: Trace<'kind>) -> access::Result<'kind> {
        match trace.path.pop_front() {
            None => self.get_place_value(trace),
            Some(indirection) => {
                let field_name = trace.field_name.clone();
                self.access_with(indirection, trace).map_err(|err|
                    err.with_context(self.description(), &field_name)
                )
            }
        }
    }

    pub fn into_ribbon(&'kind self) -> MemRibbon<'kind> {
        MemRibbon::new(0).span("", vec![Field::anon(self)])
    }

    pub fn add_field(&self, name: impl ToString, kind: &'kind Kind<'kind>) {
        let composite = match self {
            Kind::Composite(comp) => comp,
            _ => panic!("Cannot treat kind '{self}' as composite"),
        };

        composite.fields.borrow_mut().push(Field::new(name, kind))
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

impl<'kind> CType<'kind> for Alias<'kind> {
    fn description(&self) -> &dyn Display {
        self.kind.description()
    }

    fn size_of(&self) -> u16 {
        self.kind.size_of()
    }

    fn align_of(&self) -> u16 {
        self.kind.align_of()
    }

    fn access_with(&self, indirection: Indirection, trace: Trace<'kind>) -> access::Result<'kind> {
        self.kind.access_with(indirection, trace)
    }
}

impl Display for Alias<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name.as_str())
    }
}

use crate::{
    kind::{PrimValue, Kind, Primitive},
    access::{ErrorKind, Error, PlaceValue, AccessTrace, AccessUnit},
};

use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReferenceMode {
    Ref,
    Ptr,
}

impl fmt::Display for ReferenceMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ReferenceMode::Ref => "reference",
            ReferenceMode::Ptr => "pointer",
        })
    }
}

#[derive(Clone, Copy)]
pub struct Reference<'kind> {
    pub mode: ReferenceMode,
    pub kind: &'kind Kind <'kind>,
}

impl<'kind> Reference<'kind> {
    pub fn align_of(&self) -> u16 {
        Primitive::Size.size_of()
    }

    pub fn size_of(&self) -> u16 {
        Primitive::Size.size_of()
    }

    pub fn access_ref(
        &'kind self,
        unit: &AccessUnit,
        trace: &mut AccessTrace<'kind>,
    ) -> Result<PlaceValue<'_>, Error<'_>>
    {
        use AccessUnit::*;

        match unit {
            Field(_) => {},
            unit => return Err(Error::at(
                trace.field_name.clone(),
                ErrorKind::Operation { op: unit.op_str(), kind: self.kind.clone() },
            )),
        };

        let new_addr = Primitive::Size.parse_at(trace.ribbon, trace.address)
            .ok_or_else(|| Error::at(
                trace.field_name.clone(),
                ErrorKind::Deref { old_addr: trace.address },
            ))?;

        trace.address = match new_addr {
            PrimValue::Size(addr) => addr as usize,
            _ => unreachable!(),
        };

        match self.kind {
            Kind::Composite(comp) => comp.access(unit, trace),
            _ => Err(Error::at(
                trace.field_name.clone(),
                ErrorKind::Operation { op: unit.op_str(), kind: self.kind.clone() },
            )),
        }
    }

    pub fn access_ptr(
        &'kind self,
        unit: &AccessUnit,
        trace: &mut AccessTrace<'kind>,
    ) -> Result<PlaceValue<'_>, Error<'_>>
    {
        use AccessUnit::*;

        let old_addr = trace.address;
        let ptr_val = Primitive::Size.parse_at(trace.ribbon, old_addr);

        trace.address = match ptr_val {
            None => return Err(Error::at(
                trace.field_name.clone(),
                ErrorKind::Deref { old_addr },
            )),
            Some(PrimValue::Size(adr)) => adr as usize,
            Some(_) => unreachable!(),
        };

        let field = match unit {
            Deref => return self.kind.access(trace),
            Index(idx) => {
                trace.address += self.kind.size_of() as usize * idx;
                return self.kind.access(trace)
            },
            Arrow(field) => field.clone(),
            unit => return Err(Error::at(
                trace.field_name.clone(),
                ErrorKind::Operation { op: unit.op_str(), kind: self.kind.clone() },
            )),
        };

        match self.kind {
            Kind::Composite(comp) => comp.access(&Field(field), trace),
            kind => return Err(Error::at(
                trace.field_name.clone(),
                ErrorKind::Arrow { kind: kind.clone() },
            )),
        }
    }

    pub fn access(&'kind self, unit: &AccessUnit, trace: &mut AccessTrace<'kind>) -> Result<PlaceValue<'_>, Error<'_>> {
        match self.mode {
            ReferenceMode::Ref => self.access_ref(unit, trace),
            ReferenceMode::Ptr => self.access_ptr(unit, trace),
        }
    }
}

impl fmt::Display for Reference<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ref_type = match self.mode {
            ReferenceMode::Ref => "&",
            ReferenceMode::Ptr => "*",
        };

        write!(f, "{}{}", self.kind, ref_type)
    }
}

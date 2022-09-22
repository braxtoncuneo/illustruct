use crate::{
    access::{self, Indirection, Trace},
    kind::{PrimValue, Kind, Primitive, CType},
};

use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Ref,
    Ptr,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Mode::Ref => "reference",
            Mode::Ptr => "pointer",
        })
    }
}

#[derive(Clone, Copy)]
pub struct Reference<'kind> {
    pub mode: Mode,
    pub kind: &'kind Kind <'kind>,
}

impl<'kind> Reference<'kind> {
    pub fn new(mode: Mode, kind: &'kind Kind<'kind>) -> Self {
        Self { mode, kind }
    }

    pub fn access_ref(
        &self,
        indirection: Indirection,
        mut trace: Trace<'kind>,
    ) -> access::Result<'kind> {
        if !indirection.is_field() {
            return Err(access::Error::at(
                trace.field_name.clone(),
                access::ErrorKind::Operation { op: indirection.operator(), kind: self.kind.clone() },
            ));
        }

        let new_addr = Primitive::Size.parse_at(trace.ribbon, trace.address)
            .ok_or_else(|| access::Error::at(
                trace.field_name.clone(),
                access::ErrorKind::Deref { old_addr: trace.address },
            ))?;

        trace.address = match new_addr {
            PrimValue::Size(addr) => addr as usize,
            _ => unreachable!(),
        };

        match self.kind {
            Kind::Composite(comp) => comp.access_with(indirection, trace),
            _ => Err(access::Error::at(
                trace.field_name.clone(),
                access::ErrorKind::Operation { op: indirection.operator(), kind: self.kind.clone() },
            )),
        }
    }

    pub fn access_ptr(
        &self,
        indirection: Indirection,
        mut trace: Trace<'kind>,
    ) -> access::Result<'kind> {
        let old_addr = trace.address;
        let ptr_val = Primitive::Size.parse_at(trace.ribbon, old_addr);

        trace.address = match ptr_val {
            None => return Err(access::Error::at(
                trace.field_name.clone(),
                access::ErrorKind::Deref { old_addr },
            )),
            Some(PrimValue::Size(adr)) => adr as usize,
            Some(_) => unreachable!(),
        };

        match indirection {
            Indirection::Arrow(field) => match self.kind {
                Kind::Composite(comp) => comp.access_with(Indirection::Field(field), trace),
                kind => Err(access::Error::at(
                    trace.field_name.clone(),
                    access::ErrorKind::Arrow { kind: kind.clone() },
                )),
            }
            Indirection::Deref => self.kind.access(trace),
            Indirection::Index(idx) => {
                trace.address += self.kind.size_of() as usize * idx;
                self.kind.access(trace)
            }
            Indirection::Field(_) => Err(access::Error::at(
                trace.field_name.clone(),
                access::ErrorKind::Operation { op: indirection.operator(), kind: self.kind.clone() },
            )),
        }
    }
}

impl<'kind> CType<'kind> for Reference<'kind> {
    fn description(&self) -> &dyn fmt::Display {
        &self.mode
    }

    fn align_of(&self) -> u16 {
        Primitive::Size.size_of()
    }

    fn size_of(&self) -> u16 {
        Primitive::Size.size_of()
    }
    
    fn access_with(&self, indirection: Indirection, trace: Trace<'kind>) -> access::Result<'kind> {
        match self.mode {
            Mode::Ref => self.access_ref(indirection, trace),
            Mode::Ptr => self.access_ptr(indirection, trace),
        }
    }
}

impl fmt::Display for Reference<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ref_type = match self.mode {
            Mode::Ref => "&",
            Mode::Ptr => "*",
        };

        write!(f, "{}{}", self.kind, ref_type)
    }
}

use core::fmt;

use crate::{
    access::{self, Indirection, Trace},
    kind::{Kind, primitive::Primitive},
};

use super::CType;

#[derive(Clone, Copy)]
pub struct Array <'kind> {
    pub kind: &'kind Kind<'kind>,
    pub size: usize,
}

impl <'kind> Array <'kind> {
    pub fn base_fields(&self, address: &mut usize) -> Vec<(usize, Primitive)> {
        *address += self.kind.align_pad(*address as u16) as usize;

        (0..self.size)
            .flat_map(|_| self.kind.base_fields(address))
            .collect()
    }
}

impl<'kind> CType<'kind> for Array<'kind> {
    fn description(&self) -> &dyn fmt::Display {
        self
    }

    fn size_of(&self) -> u16 {
        self.kind.size_of() * (self.size as u16)
    }

    fn align_of(&self) -> u16 {
        self.kind.align_of()
    }
    
    fn access_with(&self, indirection: Indirection, mut trace: Trace<'kind>) -> access::Result<'kind> {
        match indirection {
            Indirection::Deref => self.kind.access(trace),
            Indirection::Index(idx) => {
                trace.address += self.kind.size_of() as usize * idx;
                self.kind.access(trace)
            },
            indirection => Err(access::Error::at(
                trace.field_name.clone(),
                access::ErrorKind::Operation { op: indirection.operator(), kind: self.kind.clone() },
            )),
        }
    }
}

impl <'kind> fmt::Display for Array<'kind> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}[{}]",self.kind, self.size)
    }
}

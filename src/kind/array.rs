use core::fmt;
use std::fmt::Display;

use crate::access::{AccessUnit, AccessTrace, PlaceValue, Error, ErrorKind};

use super::{Kind, Primitive};



#[derive(Clone)]
pub struct Array <'kind> {
    pub kind: &'kind Kind<'kind>,
    pub size: usize,
}


impl <'kind> Array <'kind> {

    pub fn size_of(&self) -> u16 {
        self.kind.size_of() * (self.size as u16)
    }

    pub fn align_of(&self) -> u16 {
        self.kind.align_of()
    }

    pub fn base_fields(&self, address: &mut usize) -> Vec<(usize, Primitive)> {
        *address += self.kind.align_pad(*address as u16) as usize;

        let mut result = Vec::new();
        for _ in 0..self.size {
            result.append(&mut self.kind.base_fields(address))
        }

        result
    }



    pub fn access(
        &'kind self,
        unit: &AccessUnit,
        trace: &mut AccessTrace<'kind>,
    ) -> Result<PlaceValue<'_>, Error<'_>>
    {
        use AccessUnit::*;

        match unit {
            Deref => return self.kind.access(trace),
            Index(idx) => {
                trace.address += self.kind.size_of() as usize * idx;
                return self.kind.access(trace)
            },
            unit => return Err(Error::at(
                trace.field_name.clone(),
                ErrorKind::Operation { op: unit.op_str(), kind: self.kind.clone() },
            )),
        };

    }


}


impl <'kind> fmt::Display for Array<'kind> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}[{}]",self.kind,self.size)
    }
}

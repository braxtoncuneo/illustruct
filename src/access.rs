#![allow(dead_code)]

use std::fmt;

use crate::kind::Kind;

use crate::mem_ribbon::MemRibbon;

#[non_exhaustive]
pub enum MemByte {
    Undefined,
    OutOfBounds,
    Byte(u8),
}

impl MemByte {
    pub fn byte(&self) -> Option<u8> {
        match self {
            MemByte::Byte(byte) => Some(*byte),
            _ => None,
        }
    }

    pub fn writable(&mut self) -> &mut u8 {
        *self = MemByte::Byte(0);
        match self {
            MemByte::Byte(val) => val,
            _ => unreachable!(),
        }
    }
}

pub struct PlaceValue<'kind> {
    pub kind: &'kind Kind <'kind>,
    pub address: usize,
}

#[derive(Debug,PartialEq,Eq)]
pub enum AccessUnit {
    Field(String),
    Arrow(String),
    Deref,
    Index(usize),
}

impl AccessUnit {
    pub fn op_str(&self) -> &'static str {
        match self {
            AccessUnit::Field(_) => ".",
            AccessUnit::Arrow(_) => "->",
            AccessUnit::Deref    => "*",
            AccessUnit::Index(_) => "[]",
        }
    }
}

#[derive(Debug)]
pub struct Access {
    sequence: Vec<AccessUnit>,
}

impl Access {
    pub fn new(base: &str) -> Self {
        Self { sequence: vec![AccessUnit::Field(base.into())] }
    }

    pub fn deref(mut self) -> Self {
        self.sequence.push(AccessUnit::Index(0usize));
        self
    }

    pub fn index(mut self,idx: usize) -> Self {
        self.sequence.push(AccessUnit::Index(idx));
        self
    }

    pub fn field(mut self,fname: &str) -> Self {
        self.sequence.push(AccessUnit::Field(fname.to_string()));
        self
    }

    pub fn arrow(self, fname: &str) -> Self {
        self.deref().field(fname)
    }

    pub fn iter<'a>(&'a self) -> AccessIter<'a> {
        AccessIter::over(self)
    }
}

pub struct AccessIter<'kind> {
    access: &'kind Access,
    position: usize,
}

impl<'kind> AccessIter<'kind> {
    fn over(access: &'kind Access) -> Self {
        Self { access, position: 0 }
    }
}

impl <'kind> Iterator for AccessIter <'kind> {
    type Item = &'kind AccessUnit;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.access.sequence.get(self.position);
        self.position += 1;
        result
    }
}

pub struct AccessTrace <'kind> {
    pub ribbon: &'kind MemRibbon <'kind>,
    pub iter: AccessIter <'kind>,
    pub address: usize,
    pub field_name : String,
}

pub struct Error<'a> {
    field_name: String,
    kind: ErrorKind<'a>,
    context: Option<String>,
}

impl<'a> Error<'a> {
    pub fn at(field_name: String, kind: ErrorKind<'a>) -> Self {
        Self { field_name, kind, context: None }
    }

    pub fn with_context(mut self, category: &dyn fmt::Display, field_name: &str) -> Self {
        self.context = Some(format!(" within {} {}", category, field_name));
        self
    }
}

impl fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { field_name, kind, context } = self;

        match kind {
            ErrorKind::Operation { op, kind } =>
                write!(f, "Operation {op} undefined for {kind} {field_name}")?,
            ErrorKind::Arrow { kind } =>
                write!(f, "Arrow operator undefined for pointer {field_name} to non-composite type {kind}")?,
            ErrorKind::Unwind { original, kind } =>
                write!(f, "{original} in {kind} {field_name}")?,
            ErrorKind::Deref { old_addr } =>
                write!(f, "Bad deref of address {old_addr} for reference field {field_name}")?,
            ErrorKind::SubField { name } =>
                write!(f, "Attempted to access non-existant field {name:?} in composite type {field_name}")?,
        }

        if let Some(s) = context {
            f.write_str(s)?;
        }

        Ok(())
    }
}

pub enum ErrorKind<'a> {
    Operation {
        op: &'static str,
        kind: Kind<'a>,
    },
    Arrow {
        kind: Kind<'a>
    },
    Unwind {
        original: String,
        kind: Kind<'a>,
    },
    Deref {
        old_addr: usize,
    },
    SubField {
        name: String,
    },
}

#[cfg(test)]
mod test {
    use super::{Access, AccessUnit};

    #[test]
    fn single(){
        assert_eq!(
            Access::new("abc").sequence,
            vec![AccessUnit::Field("abc".to_string())],
        );
    }


    #[test]
    fn field(){
        let buf = Access::new("abc").field("def");
        assert_eq!(
            buf.sequence,
            vec![
                AccessUnit::Field("abc".to_string()),
                AccessUnit::Field("def".to_string())
            ],
        );
    }


    #[test]
    fn multi_field(){
        let buf = Access::new("abc").field("def").field("ghi");

        assert_eq!(
            buf.sequence,
            vec![
                AccessUnit::Field("abc".to_string()),
                AccessUnit::Field("def".to_string()),
                AccessUnit::Field("ghi".to_string())
            ],
        );
    }


    #[test]
    fn deref(){
        let buf = Access::new("abc").deref();
        assert_eq!(buf.sequence,vec![
            AccessUnit::Field("abc".to_string()),
            AccessUnit::Deref
        ]);
    }


    #[test]
    fn index(){
        let buf = Access::new("abc").index(2usize);
        assert_eq!(buf.sequence,vec![
            AccessUnit::Field("abc".to_string()),
            AccessUnit::Index(2usize)
        ]);
    }


    #[test]
    fn arrow(){
        let buf = Access::new("abc").arrow("def");
        assert_eq!(buf.sequence,vec![
            AccessUnit::Field("abc".to_string()),
            AccessUnit::Arrow("def".to_string())
        ]);
    }

}

// impl FromStr for AccessBuf
// {
//     type Err = String;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let mut rest : Vec<AccessSegment> = Vec::new();
//         for findex, fpath in s.split('.').enumerate()
//         {
//             if findex == 0 {

//             }
//             for iindex, ipath in s.split('@').enumerate()
//             {

//             }
//         }

//     }
// }



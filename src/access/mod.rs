#![allow(dead_code)]

use std::{
    collections::VecDeque,
    fmt,
    str::FromStr,
};

use crate::{
    kind::Kind,
    mem_ribbon::MemRibbon,
};

mod parse;

pub type Result<'kind> = std::result::Result<PlaceValue<'kind>, Error<'kind>>;

pub struct PlaceValue<'kind> {
    pub kind: &'kind Kind <'kind>,
    pub address: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Indirection {
    Field(String),
    Arrow(String),
    Deref,
    Index(usize),
}

impl Indirection {
    pub fn is_field(&self) -> bool {
        matches!(self, Indirection::Field(_))
    }

    pub fn as_field(&self) -> Option<&str> {
        match self {
            Indirection::Field(name) => Some(name),
            _ => None,
        }
    }

    pub fn operator(&self) -> &'static str {
        match self {
            Indirection::Field(_) => ".",
            Indirection::Arrow(_) => "->",
            Indirection::Deref    => "*",
            Indirection::Index(_) => "[]",
        }
    }
}

#[derive(Debug)]
pub struct Path(pub VecDeque<Indirection>);

impl Path {
    pub fn new(base: &str) -> Self {
        let mut sequence = VecDeque::new();
        sequence.push_back(Indirection::Field(base.into()));
        Self(sequence)
    }

    pub fn deref(mut self) -> Self {
        self.0.push_back(Indirection::Index(0usize));
        self
    }

    pub fn index(mut self, idx: usize) -> Self {
        self.0.push_back(Indirection::Index(idx));
        self
    }

    pub fn field(mut self, fname: &str) -> Self {
        self.0.push_back(Indirection::Field(fname.to_string()));
        self
    }

    pub fn arrow(self, fname: &str) -> Self {
        self.deref().field(fname)
    }

    pub fn pop_front(&mut self) -> Option<Indirection> {
        self.0.pop_front()
    }
}

impl<T: Into<VecDeque<Indirection>>> From<T> for Path {
    fn from(collection: T) -> Self {
        Path(collection.into())
    }
}

impl<T> PartialEq<T> for Path
where VecDeque<Indirection>: PartialEq<T> {
    fn eq(&self, other: &T) -> bool {
        self.0 == *other
    }
}

impl FromStr for Path {
    type Err = pom::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        // Bindings required for borrow checker
        let chars = s.chars().collect::<Vec<_>>();
        let parser = parse::access_expr();
        parser.parse(&chars)
    }
}

pub struct Trace<'kind> {
    pub ribbon: &'kind MemRibbon<'kind>,
    pub path: Path,
    pub address: usize,
    pub field_name: String,
}

pub struct Error<'kind> {
    pub field_name: String,
    pub kind: ErrorKind<'kind>,
    pub context: Option<String>,
}

impl<'kind> Error<'kind> {
    pub fn at(field_name: impl ToString, kind: ErrorKind<'kind>) -> Self {
        Self {
            field_name: field_name.to_string(),
            kind,
            context: None,
        }
    }

    pub fn with_context(mut self, description: &dyn fmt::Display, field_name: &str) -> Self {
        self.context = Some(format!(" within {} {}", description, field_name));
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
            ErrorKind::DirectAccess =>
                write!(f, "{field_name} cannot be accessed as a field")?,
            ErrorKind::RibbonOp { op } =>
                write!(f, "Operation {op} undefined for MemoryRibbon")?,
        }

        if let Some(s) = context {
            f.write_str(s)?;
        }

        Ok(())
    }
}

impl fmt::Debug for Error<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}", self)
    }
}

pub enum ErrorKind<'kind> {
    Operation {
        op: &'static str,
        kind: Kind<'kind>,
    },
    Arrow {
        kind: Kind<'kind>
    },
    Unwind {
        original: String,
        kind: Kind<'kind>,
    },
    Deref {
        old_addr: usize,
    },
    SubField {
        name: String,
    },
    DirectAccess,
    RibbonOp {
        op: &'static str,
    },
}

impl<'kind> ErrorKind<'kind> {
    pub fn operation(indirection: &Indirection, kind: Kind<'kind>) -> Self {
        Self::Operation { op: indirection.operator(), kind }
    }

    pub fn ribbon_op(indirection: &Indirection) -> Self {
        Self::RibbonOp { op: indirection.operator() }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn single() {
        assert_eq!(
            Path::new("abc"),
            &[Indirection::Field("abc".to_string())],
        );
    }

    #[test]
    fn field() {
        assert_eq!(
            Path::new("abc").field("def"),
            &[
                Indirection::Field("abc".to_string()),
                Indirection::Field("def".to_string()),
            ],
        );
    }

    #[test]
    fn multi_field() {
        assert_eq!(
            Path::new("abc").field("def").field("ghi"),
            &[
                Indirection::Field("abc".to_string()),
                Indirection::Field("def".to_string()),
                Indirection::Field("ghi".to_string()),
            ],
        );
    }

    #[test]
    fn deref() {
        assert_eq!(
            Path::new("abc").deref(),
            &[
                Indirection::Field("abc".to_string()),
                Indirection::Deref,
            ]
        );
    }

    #[test]
    fn index() {
        assert_eq!(
            Path::new("abc").index(2),
            &[
                Indirection::Field("abc".to_string()),
                Indirection::Index(2),
            ],
        );
    }

    #[test]
    fn arrow() {
        assert_eq!(
            Path::new("abc").arrow("def"),
            &[
                Indirection::Field("abc".to_string()),
                Indirection::Arrow("def".to_string()),
            ],
        );
    }
}
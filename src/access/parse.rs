use std::{collections::VecDeque, cell::Ref};

use pom::parser::{sym, is_a};
use crate::{access::AccessUnit, kind::reference::ReferenceMode};

use super::Access;


fn is_alphunder(ch:char) -> bool {
    char::is_alphabetic(ch) | (ch == '_')
}

fn is_alphnumder(ch:char) -> bool {
    char::is_alphanumeric(ch) | (ch == '_')
}

fn star_space<'a>() -> pom::parser::Parser<'a,char,()> {
    is_a(char::is_whitespace).repeat(0..).discard()
}

fn some_space<'a>() -> pom::parser::Parser<'a,char,()> {
    is_a(char::is_whitespace).repeat(1..).discard()
}


fn label<'a>() -> pom::parser::Parser<'a,char,String> {
    use pom::parser::is_a;

    (is_a(is_alphunder) + is_a(is_alphnumder).repeat(0..))
        .map(|(head,tail)|{
            let mut result = String::new();
            result.push(head);
            result.extend(tail.into_iter());
            result
        })
}

fn integer<'a>() -> pom::parser::Parser<'a,char,usize> {
    (is_a(char::is_numeric).repeat(1..))
        .map(|seq|{
            let mut result = String::new();
            result.extend(seq.into_iter());
            result.parse().unwrap()
        })
}

fn field_expr<'a>() -> pom::parser::Parser<'a,char,AccessUnit> {
    (sym('.') * label())
        .map(|label| AccessUnit::Field(label))
}

fn index_expr<'a>() -> pom::parser::Parser<'a,char,AccessUnit> {
    ( sym('[') * integer() - sym(']') )
        .map(|idx| AccessUnit::Index(idx))
}

fn arrow_expr<'a>() -> pom::parser::Parser<'a,char,AccessUnit> {
    ((sym('-') + sym('>')) * label())
        .map(|label| AccessUnit::Arrow(label))
}

pub fn access_expr<'a>() -> pom::parser::Parser<'a,char,Access> {
    let parser = sym('*').opt()
        + label()
        + ( field_expr() | index_expr() | arrow_expr() ).repeat(0..);

    parser.map(|((deref,head),mut tail)|{
        let mut sequence = vec![AccessUnit::Field(head)];
        sequence.append(&mut tail);
        if deref.is_some() {
            sequence.push(AccessUnit::Deref);
        }
        sequence.into()
    })
}



pub struct RefrDecl {
    label: String,
    mode: ReferenceMode,
}


pub fn field_decl<'a>() -> pom::parser::Parser<'a,char,()> {
    (
          ( label()  - some_space() )
        + ( (sym('*')|sym('&')).opt() - some_space() )
        + ( label() - some_space() )
        + ( (sym('[') - some_space()) + integer() - (some_space()-sym(']')) )
        - sym(';')
    ).map(|(((base_kind,refr),name),arr)|{
        ()
    })
}

pub fn kind_expr<'a>() -> pom::parser::Parser<'a,char,Access> {
    todo!()
}




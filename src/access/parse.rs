#![allow(dead_code)]

use std::iter;
use access::Indirection;
use pom::parser::{sym, is_a};

use crate::{
    access,
    kind::reference,
};

type Parser<'a, O> = pom::parser::Parser<'a, char, O>;

fn is_alphunder(ch:char) -> bool {
    char::is_alphabetic(ch) || (ch == '_')
}

fn is_alphnumder(ch:char) -> bool {
    char::is_alphanumeric(ch) || (ch == '_')
}

fn star_space<'a>() -> Parser<'a, ()> {
    is_a(char::is_whitespace).repeat(0..).discard()
}

fn some_space<'a>() -> Parser<'a, ()> {
    is_a(char::is_whitespace).repeat(1..).discard()
}

fn label<'a>() -> Parser<'a, String> {
    (is_a(is_alphunder) + is_a(is_alphnumder).repeat(0..))
        .map(|(head, tail)| iter::once(head).chain(tail).collect())
}

fn integer<'a>() -> Parser<'a, usize> {
    (is_a(char::is_numeric).repeat(1..))
        .map(|seq| seq.into_iter().collect::<String>())
        .convert(|s| s.parse())
}

fn field_expr<'a>() -> Parser<'a, Indirection> {
    (sym('.') * label())
        .map(Indirection::Field)
}

fn index_expr<'a>() -> Parser<'a, Indirection> {
    (sym('[') * integer() - sym(']'))
        .map(Indirection::Index)
}

fn arrow_expr<'a>() -> Parser<'a, Indirection> {
    ((sym('-') + sym('>')) * label())
        .map(Indirection::Arrow)
}

pub fn access_expr<'a>() -> Parser<'a, access::Path> {
    let parser = sym('*').opt()
        + label()
        + (field_expr() | index_expr() | arrow_expr()).repeat(0..);

    parser.map(|((deref, head), tail)|
        iter::once(Indirection::Field(head))
            .chain(tail)
            .chain(deref.map(|_| Indirection::Deref))
            .collect::<Vec<_>>()
            .into()
    )
}

pub struct RefrDecl {
    label: String,
    mode: reference::Mode,
}

pub fn field_decl<'a>() -> Parser<'a, ()> {
    (
        (label() - some_space())
        + ((sym('*') | sym('&')).opt() - some_space())
        + (label() - some_space())
        + ((sym('[') - some_space()) + integer() - (some_space() - sym(']')))
        - sym(';')
    ).map(drop)
}

pub fn kind_expr<'a>() -> Parser<'a, access::Path> {
    todo!()
}

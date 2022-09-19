use crate::{
    access::{ErrorKind, Error, PlaceValue, AccessTrace, AccessUnit},
    kind::Kind,
    block_draw::{block_plan::BlockDiagPlan, util::{Vec2, Translate}, BlockDrawSpec},
};

use std::{fmt, cell::RefCell};

use super::Primitive;

#[derive(Clone)]
pub struct Field<'kind> {
    pub name: Option<String>,
    pub kind: &'kind Kind<'kind>,
}

impl<'a> Field<'a> {
    pub fn new(name: impl ToString, kind: &'a Kind<'a>) -> Self {
        Self { name: Some(name.to_string()), kind }
    }

    pub fn anon(kind: &'a Kind<'a> ) -> Self {
        Self { name: None, kind }
    }

    pub fn make_plan(&self, spec: &'a BlockDrawSpec, mins: Vec2, width: Option<f32>, notch: bool) -> BlockDiagPlan<'a> {
        let width = width.unwrap_or_else(|| spec.field_width(self, notch));
        let mut plan = spec.make_plan(self.kind, mins, Some(width), notch);

        match self.name.as_deref() {
            None => plan,
            Some(label) => {
                let member_width = spec.member_width(self.kind);
                let left_width = if member_width > 0.0 {
                    member_width + spec.prong_xpad
                } else {
                    0.0
                };
                let label_pos_x : f32 = width - left_width - spec.label_width(label) - spec.label_pads.x;
                // spec.name_width(self.kind) + spec.label_pads.x + spec.text_pads.x * 2f32;
                let label_pos_y : f32 = spec.label_pads.y + spec.fill_inset * 0.5f32;
                let label_svg = spec.draw_label(label)
                    .set("transform", Translate(label_pos_x, label_pos_y));
                plan.head = plan.head.add(label_svg);
                plan
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositeMode {
    Product,
    Sum,
}

impl fmt::Display for CompositeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Product => "struct",
            Self::Sum => "union",
        })
    }
}

#[derive(Clone)]
pub struct Composite<'kind> {
    pub name: String,
    pub mode: CompositeMode,
    pub fields: RefCell<Vec<Field<'kind>>>,
}

impl<'kind> Composite<'kind> {
    pub(crate) fn align_of(&self) -> u16 {
        self.fields.borrow().iter()
            .map(|x| x.kind.size_of())
            .max()
            .unwrap_or_default()
    }

    pub(crate) fn size_of_no_end_pad(&self) -> u16 {
        match self.mode {
            CompositeMode::Product => self.fields.borrow().iter().fold(0, |acc,x|
                acc + x.kind.size_of() + x.kind.align_pad(acc)
            ),
            CompositeMode::Sum => self.fields.borrow().iter()
                .map(|x| x.kind.size_of())
                .max()
                .unwrap_or_default()
        }
    }

    pub(crate) fn size_of(&self) -> u16 {
        let base_size = self.size_of_no_end_pad();
        let align = self.align_of();
        let remainder = base_size % align;
        let padding = if remainder > 0 { align - remainder } else { 0 };

        base_size + padding
    }

    pub(crate) fn offset_of(&self, field_name: &str) -> Option<u16> {
        if self.mode == CompositeMode::Sum {
            return Some(0);
        }

        let mut result = 0;

        for field in self.fields.borrow().iter() {
            result += field.kind.align_pad(result);
            let name = match field.name.as_ref() {
                Some(name) => name,
                None => continue,
            };

            if field_name == name {
                return Some(result)
            }
            result += field.kind.size_of();
        }

        None
    }

    pub fn base_fields(&self, address: &mut usize) -> Vec<(usize,Primitive)> {

        if self.mode == CompositeMode::Sum {
            if self.fields.borrow().is_empty() {
                return Vec::new();
            } else {
                return self.fields.borrow()[0].kind.base_fields(address);
            }
        }

        self.fields.borrow().iter()
            .map(|f| f.kind.base_fields(address))
            .flatten()
            .collect()
    }

    pub(crate) fn type_of(&'kind self, field_name: &str) -> Option<&'kind Kind<'kind>> {
        self.fields.borrow().iter()
            .find(|field| field.name.as_deref() == Some(field_name))
            .map(|field| field.kind)
    }

    pub(crate) fn access(&'kind self, unit: &AccessUnit, trace: &mut AccessTrace<'kind>) -> Result<PlaceValue<'kind>, Error> {
        let subfield = match unit {
            AccessUnit::Field(field) => field,
            _ => return Err(Error::at(
                trace.field_name.clone(),
                ErrorKind::Operation { op: unit.op_str(), kind: Kind::Composite(self.clone()) },
            )),
        };

        match self.offset_of(subfield) {
            Some(offset) => {
                trace.address += offset as usize;
                let field_kind = self.type_of(subfield).unwrap();
                field_kind.access(trace)
            }
            None => Err(Error::at(
                trace.field_name.clone(),
                ErrorKind::SubField { name: subfield.clone() },
            )),
        }
    }
}

impl fmt::Display for Composite<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

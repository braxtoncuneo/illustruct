use crate::{
    access::{self, Indirection, Trace},
    kind::Kind,
    block_draw::{block_plan::BlockDiagPlan, util::{Vec2, Translate}, BlockDrawSpec},
};

use std::{fmt, cell::RefCell};

use super::{Primitive, CType};

#[derive(Clone)]
pub struct Field<'kind> {
    pub name: Option<String>,
    pub kind: &'kind Kind<'kind>,
}

impl<'kind> Field<'kind> {
    pub fn new(name: impl ToString, kind: &'kind Kind<'kind>) -> Self {
        Self { name: Some(name.to_string()), kind }
    }

    pub fn anon(kind: &'kind Kind<'kind> ) -> Self {
        Self { name: None, kind }
    }

    pub fn make_plan(&self, spec: &BlockDrawSpec, mins: Vec2, width: Option<f32>, notch: bool) -> BlockDiagPlan<'kind> {
        let width = width.unwrap_or_else(|| spec.field_width(self, notch));
        let mut plan = spec.make_plan(self.kind, mins, Some(width), notch);

        if let Some(label) = self.name.as_deref() {
            let mut left_width = spec.member_width(self.kind);
            if left_width > 0.0 {
                left_width += spec.prong_xpad
            }
            let label_pos_x = width - left_width - spec.label_width(label) - spec.label_pads.x;
            // spec.name_width(self.kind) + spec.label_pads.x + spec.text_pads.x * 2f32;
            let label_pos_y = spec.label_pads.y + spec.fill_inset * 0.5;
            let label_svg = spec.draw_label(label)
                .set("transform", Translate(label_pos_x, label_pos_y));

            plan.head = plan.head.add(label_svg);
        }
        
        plan
    }

    pub fn size_of(&self) -> u16 {
        self.kind.size_of()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Product,
    Sum,
}

impl fmt::Display for Mode {
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
    pub mode: Mode,
    pub fields: RefCell<Vec<Field<'kind>>>,
}

impl<'kind> Composite<'kind> {
    pub fn new(name: impl ToString, mode: Mode, fields: Vec<Field<'kind>>) -> Self {
        Self {
            name: name.to_string(),
            mode,
            fields: RefCell::new(fields),
        }
    }

    pub fn size_of_no_end_pad(&self) -> u16 {
        match self.mode {
            Mode::Product => self.fields.borrow().iter().fold(0, |acc,x|
                acc + x.kind.size_of() + x.kind.align_pad(acc)
            ),
            Mode::Sum => self.fields.borrow().iter()
                .map(Field::size_of)
                .max()
                .unwrap_or_default()
        }
    }

    pub fn offset_of(&self, field_name: &str) -> Option<u16> {
        if self.mode == Mode::Sum {
            return Some(0);
        }

        let mut result = 0;

        for field in self.fields.borrow().iter() {
            result += field.kind.align_pad(result);

            if field.name.as_deref() == Some(field_name) {
                return Some(result)
            }
            
            result += field.kind.size_of();
        }

        None
    }

    pub fn base_fields(&self, address: &mut usize) -> Vec<(usize,Primitive)> {
        match self.mode {
            Mode::Sum => self.fields.borrow().first()
                .map(|field| field.kind.base_fields(address))
                .unwrap_or_default(),
            Mode::Product => self.fields.borrow().iter()
                .flat_map(|field| field.kind.base_fields(address))
                .collect(),
        }

    }

    pub fn type_of(&self, field_name: &str) -> Option<&'kind Kind<'kind>> {
        self.fields.borrow().iter()
            .find(|field| field.name.as_deref() == Some(field_name))
            .map(|field| field.kind)
    }
}

impl<'kind> CType<'kind> for Composite<'kind> {
    fn description(&self) -> &dyn fmt::Display {
        &self.mode
    }

    fn size_of(&self) -> u16 {
        let base_size = self.size_of_no_end_pad();
        let align = self.align_of();
        let remainder = base_size % align;
        let padding = if remainder > 0 { align - remainder } else { 0 };

        base_size + padding
    }

    fn align_of(&self) -> u16 {
        self.fields.borrow().iter()
            .map(Field::size_of)
            .max()
            .unwrap_or_default()
    }

    fn access_with(&self, indirection: Indirection, mut trace: Trace<'kind>) -> access::Result<'kind> {
        let subfield = indirection.as_field().ok_or_else(|| access::Error::at(
            trace.field_name.clone(),
            access::ErrorKind::Operation { op: indirection.operator(), kind: Kind::from(self.clone()) },
        ))?;

        let offset = self.offset_of(subfield).ok_or_else(|| access::Error::at(
            trace.field_name.clone(),
            access::ErrorKind::SubField { name: subfield.into() },
        ))?;

        trace.address += offset as usize;
        self.type_of(subfield).unwrap().access(trace)
    }
}

impl fmt::Display for Composite<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

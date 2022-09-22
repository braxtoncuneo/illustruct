#![allow(dead_code)]

use std::{iter::Iterator, fmt};
use unicode_width::UnicodeWidthStr;
use svg::node::element::{
    Text, Path, Group,
    path::Data
};

use crate::kind::{Kind, composite::{self, Field, Composite}, array::Array, CType};

pub mod block_plan;
pub mod util;

use util::Vec2;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BlockDrawSpec {
    pub char_dims: Vec2,
    pub text_pads: Vec2,
    pub label_pads: Vec2,
    pub union_xpad: f32,
    pub fill_inset: f32,
    pub prong_width: f32,
    pub prong_xpad: f32,
    pub chamfer_size: f32,
}

impl BlockDrawSpec {
    fn label_height (&self) -> f32 {
        self.char_dims.y + 2.0 * self.text_pads.y
    }

    pub fn line_height (&self) -> f32 {
        self.label_height() + 2.0 * self.label_pads.y
    }

    fn bare_width(&self,text: &str) -> f32 {
        text.width_cjk() as f32 * self.char_dims.x
    }

    pub fn tpad_width(&self,text: &str) -> f32 {
        self.bare_width(text) + 2.0 * self.text_pads.x
    }

    pub fn label_width(&self, text: &str) -> f32 {
        self.tpad_width(text) + 2.0 * self.label_pads.x
    }

    pub fn byte_width (&self) -> f32 {
        self.tpad_width("01234567") + 2.0 * self.prong_width
    }

    pub fn repr_width (&self) -> f32 {
        self.tpad_width("0123456789") + 2.0 * self.prong_width
    }

    pub fn draw_header(&self, title: impl fmt::Display, width: f32, notch: bool) -> Group {
        let text = title.to_string();
        let x = self.tpad_width(&text) / 2.0;

        let half_height = self.line_height() / 2.0;
        let notch_x = notch.then_some(self.prong_width).unwrap_or_default();

        let data = Data::new()
            .move_to((                 0,               0))
            .line_by(( -self.prong_width,     half_height))
            .line_by((  self.prong_width,     half_height))
            .line_by((             width,               0))
            .line_by((          -notch_x,    -half_height))
            .line_by((           notch_x,    -half_height))
            .close();

        let path = Path::new()
            .set("fill", "black")
            .set("stroke", "none")
            .set("d", data);

        let text_node = Text::new()
            .add(svg::node::Text::new(text))
            .set("fill", "white")
            .set("font-family", "monospace")
            .set("font-size", self.char_dims.y)
            .set("dominant-baseline", "middle")
            .set("text-anchor", "middle")
            .set("x", x)
            .set("y", half_height);

        Group::new().add(path).add(text_node)
    }

    pub fn draw_block<'kind>(&self, kind: &Kind, width: f32, notch: bool) -> Option<Group> {
        let size = kind.size_of();
        if size <= 1 { return None; }

        let head_height = self.line_height();
        let half_head = head_height / 2.0;
        let height = self.line_height() * size as f32 + self.fill_inset;
        let chamfer = if size == 1 { 0.0 } else { self.chamfer_size };
        let rem_height = height - head_height - chamfer;

        let width_chopped = width - chamfer;
        let notch_x = notch.then_some(self.prong_width).unwrap_or_default();
        let half_inset = self.fill_inset/2f32;
        let double_inset = self.fill_inset*2f32;

        let data = Data::new()
            .move_to((                 0,                      0))
            .line_by((                 0,                 height))
            .line_by((     width_chopped,                      0))
            .line_by((           chamfer,               -chamfer))
            .line_by((                 0,            -rem_height))
            .line_by((          -notch_x,             -half_head))
            .line_by((           notch_x,             -half_head))
            .close();

        let path = Path::new()
            .set("fill", "black")
            .set("stroke", "none")
            .set("d", data);

        let fill_data = Data::new()
            .move_to((                 self.fill_inset,             self.fill_inset))
            .line_by((                               0,         height-double_inset))
            .line_by((width_chopped - half_inset * 3.0,                           0))
            .line_by((            chamfer - half_inset,       -chamfer + half_inset))
            .line_by((                               0,-rem_height + self.fill_inset))
            .line_by((                        -notch_x,     -half_head - half_inset))
            .line_by((                         notch_x,-half_head + self.fill_inset))
            .close();

        let fill_path = Path::new()
            //.set("fill", "grey")
            .set("stroke", "none")
            .set("d", fill_data);

        Some(Group::new().add(path).add(fill_path))
    }

    fn prong_line(&self) -> Path {
        let head_height = self.line_height();
        let half_height = head_height / 2f32;
        let slope    = half_height / self.prong_width;
        let start_x  = -slope * self.fill_inset;
        let top_xdif = self.prong_width + start_x;
        let top_ydif = half_height - self.fill_inset;
        let px       = self.prong_xpad;

        let data = Data::new()
            .move_to(( start_x+self.fill_inset,  self.fill_inset))
            .line_by((               -top_xdif,         top_ydif))
            .line_by((        self.prong_width,      half_height))
            .line_by((      self.fill_inset+px,                0))
            .line_by((       -self.prong_width,     -half_height))
            .line_by((                top_xdif,        -top_ydif))
            .close();

        Path::new()
            .set("fill", "grey")
            .set("stroke", "none")
            .set("d", data)
    }

    pub fn draw_label(&self, text: &str) -> Group {
        let height = self.label_height();
        let width = self.bare_width(text);

        let data = Data::new()
            .move_to((0,0))
            .elliptical_arc_by((self.text_pads.x,self.text_pads.y,0,0,0,0,height))
            //.line_by((         0,   height))
            .line_by((     width,        0))
            .elliptical_arc_by((self.text_pads.x,self.text_pads.y,0,0,0,0,-height))
            //.line_by((         0,  -height))
            .close();

        let path = Path::new()
            .set("fill", "white")
            .set("stroke", "none")
            .set("d", data);

        let text_node = Text::new()
            .add(svg::node::Text::new(text))
            .set("fill", "black")
            .set("font-family", "monospace")
            .set("font-size", self.char_dims.y)
            .set("dominant-baseline", "middle")
            .set("text-anchor", "middle")
            .set("x",  width/2f32)
            .set("y", height/2f32);

        Group::new().add(path).add(text_node)
    }

    pub fn name_width<'kind>(&self, kind: &Kind<'kind> ) -> f32 {
        self.tpad_width(&kind.to_string())
    }

    pub fn composite_member_width<'kind>(&self,comp: &Composite<'kind>) -> f32 {
        match comp.mode {
            composite::Mode::Product => comp.fields.borrow().iter()
                .enumerate()
                .map(|(i, f)| self.field_width(f, i==0))
                .max_by(f32::total_cmp)
                .unwrap_or(0.0)
                    + self.prong_xpad,
            composite::Mode::Sum => comp.fields.borrow().iter()
                .map(|x| self.field_width(x, false)
                    + self.union_xpad
                    + self.prong_width
                    + self.prong_xpad
                )
                .sum()
        }
    }

    pub fn array_member_width(&self, Array { kind, size }: Array) -> f32 {
        if size == 0 {
            return 0.0;
        }

        let first_field = Field::new(0, kind);
        let first_width = self.field_width(&first_field, true) + self.prong_xpad;

        let last_field = Field::new(size - 1, kind);
        let last_width = self.field_width(&last_field, false) + self.prong_xpad;

        first_width.max(last_width)
    }

    pub fn member_width<'kind>(&self, kind: &Kind<'kind>) -> f32 {
        match kind {
            Kind::Composite(comp) => self.composite_member_width(comp),
            Kind::Array(array) => self.array_member_width(*array),
            _ => 0.0,
        }
    }

    pub fn unlabeled_width(&self, kind: &Kind<'_>, _notch: bool) -> f32 {
        let prong_width = match kind {
            Kind::Composite(_) | Kind::Array(_) => self.prong_width,
            _ => 0.0,
        };

        self.name_width(kind)
            + self.member_width(kind)
            + prong_width
    }

    pub fn height(&self, kind: &Kind<'_>) -> f32 {
        (kind.size_of() as f32) * self.line_height()
    }

    pub fn field_height(&self, field: &Field) -> f32 {
        self.height(field.kind)
    }

    pub fn field_width(&self, field: &Field, notch: bool) -> f32 {
        self.unlabeled_width(field.kind, notch)
            + self.label_width(field.name.as_deref().unwrap_or_default())
            + self.label_pads.x * 2f32
    }

    pub fn plan_primitive<'kind>(
        &self,
        kind: &'kind Kind<'kind>,
        mins:Vec2,
        width:Option<f32>,
        notch:bool,
    ) -> block_plan::BlockDiagPlan<'kind> {
        let dims = Vec2::new(
            width.unwrap_or_else(|| self.unlabeled_width(kind, notch)),
            self.height(kind),
        );

        block_plan::BlockDiagPlan {
            spec: *self,
            head: self.draw_header(
                &kind.to_string(),
                width.unwrap_or_else(|| self.unlabeled_width(kind, notch)),
                notch,
            ),
            head_offset: 0.0,
            body_plan: None,
            relative_pos: None,
            mins,
            maxs: mins + dims,
            kind,
            sub_blocks: Vec::new(),
            graph_index: None,
        }
    }

    pub fn plan_array_fields<'kind>(
        &self,
        Array { size, kind }: Array<'kind>,
        mins: Vec2,
        width: f32,
    ) -> Vec<block_plan::BlockDiagPlan<'kind>>
    {
        let mut field_plans: Vec<block_plan::BlockDiagPlan> = Vec::new();
        let mut offset = 0;
        let mut deltas = Vec2::default();

        for index in 0..size {
            let field = Field::new(index, kind);
            let pad = field.kind.align_pad(offset);
            let pad_height = pad as f32 * self.line_height();
            let size = field.kind.size_of();
            let size_height = size as f32 * self.line_height();

            deltas.y += pad_height;

            let notch = offset == 0;

            let mut f_plan = field.make_plan(
                self,
                mins+deltas,
                Some(width),
                notch
            );

            f_plan.relative_pos  = Some(deltas);

            field_plans.push(f_plan);
            deltas.y += size_height;
            offset += size + pad;
        }

        field_plans
    }

    pub fn plan_product_fields<'kind>(
        &self,
        fields: &Vec<Field<'kind>>,
        mins: Vec2,
        width: f32,
    ) -> Vec<block_plan::BlockDiagPlan<'kind>> {
        let mut field_plans: Vec<block_plan::BlockDiagPlan> = Vec::new();
        let mut offset = 0;
        let mut deltas = Vec2::default();

        for field in fields {
            let pad = field.kind.align_pad(offset);
            let pad_height = pad as f32 * self.line_height();
            let size = field.kind.size_of();
            let size_height = size as f32 * self.line_height();

            deltas.y += pad_height;

            let (x,y) = (deltas.x+mins.x,deltas.y+mins.y);
            println!("({x},{y})");

            let notch = offset == 0;

            let mut f_plan = field.make_plan(
                self,
                mins+deltas,
                Some(width),
                notch
            );

            f_plan.relative_pos  = Some(deltas);

            field_plans.push(f_plan);
            deltas.y += size_height;
            offset += size + pad;
        }

        field_plans
    }

    pub fn plan_sum_fields<'kind>(&self, fields: &Vec<Field<'kind>>, mins: Vec2) -> Vec<block_plan::BlockDiagPlan<'kind>> {
        let mut field_plans: Vec<block_plan::BlockDiagPlan> = Vec::with_capacity(fields.len());
        let mut deltas = Vec2::default();

        for f in fields {
            let w = self.field_width(f, false)
                + self.union_xpad
                + self.prong_width
                + self.prong_xpad;

            let notch = false;

            let mut f_plan = f.make_plan(
                self,
                mins + deltas,
                None,
                notch
            );

            f_plan.relative_pos  = Some(deltas);

            field_plans.push(f_plan);
            deltas.x += w;
        }

        field_plans
    }

    pub fn make_plan<'kind>(
        &self,
        kind: &'kind Kind<'kind>,
        mins: Vec2,
        width: Option<f32>,
        notch: bool,
    ) -> block_plan::BlockDiagPlan<'kind> {
        let block_width = width.unwrap_or_else(|| self.unlabeled_width(kind, notch));
        let block_height = self.height(kind);

        let body_plan = block_plan::BlockBodyPlan {
            block_width,
            notch,
        };

        let fields = match kind {
            Kind::Composite(comp) => match comp.mode {
                composite::Mode::Product => self.plan_product_fields(
                    &comp.fields.borrow(),
                    mins,
                    self.member_width(kind)
                ),
                composite::Mode::Sum => self.plan_sum_fields(
                    &comp.fields.borrow(),
                    mins
                ),
            },
            Kind::Array(array) => self.plan_array_fields(
                    *array,
                    mins,
                    self.member_width(kind)
                ),
            _ => Default::default(),
        };

        let gapped = matches!(kind, Kind::Composite(_) | Kind::Array(_));

        let maxs = mins + Vec2::new(block_width,block_height);

        let member_width = self.member_width(kind);
        let prong_padding = if gapped { self.prong_xpad } else { 0.0 };
        let head_offset = member_width + prong_padding;
        let head = self.draw_header(
            &kind.to_string(),
            block_width - member_width - prong_padding,
            notch,
        );

        block_plan::BlockDiagPlan {
            spec: *self,
            head,
            head_offset,
            relative_pos: None,
            body_plan: Some(body_plan),
            mins,
            maxs,
            kind,
            sub_blocks: fields,
            graph_index: None,
        }
    }

    pub fn make_span_plan<'kind>(
        &self,
        kind: &'kind Kind<'kind>,
        mins: Vec2,
        width: f32,
    ) -> block_plan::BlockDiagPlan<'kind> {
        let block_height = self.height(kind);

        let fields = match kind {
            Kind::Composite(comp) => match comp.mode {
                composite::Mode::Product => self.plan_product_fields(
                    &comp.fields.borrow(),
                    mins,
                    width,
                ),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };

        let gapped = true;

        let maxs = mins + Vec2::new(width,block_height);

        let member_width = self.member_width(kind);
        let prong_padding = if gapped { self.prong_xpad } else { 0.0 };
        let head_offset = member_width + prong_padding;
        let head = Group::new();

        block_plan::BlockDiagPlan {
            spec: *self,
            head,
            head_offset,
            relative_pos: None,
            body_plan: None,
            mins,
            maxs,
            kind,
            sub_blocks: fields,
            graph_index: None,
        }
    }
}

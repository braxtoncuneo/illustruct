#![allow(dead_code)]

use std::{iter::Iterator, fmt};
use unicode_width::UnicodeWidthStr;
use svg::node::element::{
    Text, Path, Group,
    path::Data
};

use crate::kind::{Kind, composite::{CompositeMode, Field}};

use self::util::{Vec2, Translate};

pub mod block_plan;
pub mod util;

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

    pub fn draw_header(&self, title: impl fmt::Display, width: f32, notch: bool) -> Group {
        let text = title.to_string();
        let x = self.tpad_width(&text) / 2.0;

        let half_height = self.line_height() / 2.0;
        let notch_x = notch.then_some(self.prong_width).unwrap_or_default();

        let data = Data::new()
            .move_to((                 0,           0))
            .line_by(( -self.prong_width, half_height))
            .line_by((  self.prong_width, half_height))
            .line_by((             width,           0))
            .line_by((          -notch_x,-half_height))
            .line_by((           notch_x,-half_height))
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

    pub fn draw_block<'kind>(&self, kind: &'kind Kind, width: f32, notch: bool) -> Option<Group> {
        let size = kind.size_of();
        if size <= 1 { return None; }

        let head_height = self.line_height();
        let half_head = head_height / 2.0;
        let height = self.line_height() * size as f32;
        let chamfer = if size == 1 { 0.0 } else { self.chamfer_size };
        let rem_height = height - head_height - chamfer;

        let width_chopped = width - chamfer;
        let notch_x = notch.then_some(self.prong_width).unwrap_or_default();
        let half_inset = self.fill_inset/2f32;

        let data = Data::new()
            .move_to((                 0,                      0))
            .line_by((                 0, height+self.fill_inset))
            .line_by((     width_chopped,                      0))
            .line_by((           chamfer,    -chamfer-half_inset))
            .line_by((                 0, -rem_height-half_inset))
            .line_by((          -notch_x,             -half_head))
            .line_by((           notch_x,             -half_head))
            .close();

        let path = Path::new()
            .set("fill", "black")
            .set("stroke", "none")
            .set("d", data);

        let fill_data = Data::new()
            .move_to((                 self.fill_inset,          self.fill_inset))
            .line_by((                               0,   height-self.fill_inset))
            .line_by((width_chopped - half_inset * 3.0,                        0))
            .line_by((            chamfer - half_inset,                 -chamfer))
            .line_by((                               0, -rem_height + half_inset))
            .line_by((                        -notch_x,  -half_head - half_inset))
            .line_by((                         notch_x,  -half_head + half_inset))
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

    pub fn name_width<'kind>(&self, kind: &'kind Kind<'kind> ) -> f32 {
        self.tpad_width(&kind.to_string())
    }

    pub fn member_width<'kind>(&self,kind: &'kind Kind<'kind>) -> f32 {
        let comp = match kind {
            Kind::Composite(comp) => comp,
            _ => return 0.0,
        };

        match comp.mode {
            CompositeMode::Product => comp.fields.iter()
                .enumerate()
                .map(|(i, f)| self.field_width(f, i==0))
                .max_by(f32::total_cmp)
                .unwrap_or(0.0)
                    + self.prong_xpad,
            CompositeMode::Sum => comp.fields.iter()
                .map(|x| self.field_width(x, false)
                    + self.union_xpad
                    + self.prong_width
                    + self.prong_xpad
                )
                .sum()
        }
    }

    pub fn unlabeled_width(&self, kind: &Kind<'_>, _notch: bool) -> f32 {
        let prong_width = match kind {
            Kind::Composite(_) => self.prong_width,
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
        self.unlabeled_width(field.kind,notch)
            + self.label_width(field.name.as_deref().unwrap_or_default())
            + self.label_pads.x * 2f32
    }

    pub fn plan_primitiv<'kind>(
        &'kind self,
        kind: &'kind Kind<'kind>,
        mins:Vec2,
        width:Option<f32>,
        notch:bool,
    ) -> block_plan::BlockDiagPlan<'kind>
    {
        let dims = Vec2::new(
            width.unwrap_or(self.unlabeled_width(kind,notch)),
            self.height(kind),
        );

        block_plan::BlockDiagPlan {
            spec: &self,
            head: self.draw_header(
                &kind.to_string(),
                width.unwrap_or(self.unlabeled_width(kind, notch)),
                notch,
            ),
            body_plan: None,
            mins,
            maxs: mins+dims,
            kind,
            sub_blocks: Vec::new(),
            graph_index: None,
        }
    }

    pub fn plan_product_fields<'kind>(
        &'kind self,
        fields: &Vec<Field<'kind>>,
        mins: Vec2,
        width: f32,
    ) -> Vec<block_plan::BlockDiagPlan<'kind>>
    {
        let mut field_plans: Vec<block_plan::BlockDiagPlan> = Vec::new();
        let mut offset = 0;
        let mut deltas = Vec2::default();

        for field in fields {
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

            if let Some(body_plan) = &mut f_plan.body_plan {
                _ = body_plan.relative_pos.insert(deltas);
                //g.clone().set("transform", Translate(x, y));
            };

            field_plans.push(f_plan);
            deltas.y += size_height;
            offset += size + pad;
        }

        field_plans
    }

    pub fn plan_sum_fields<'a>(&'a self, fields: &Vec<Field<'a>>, mins: Vec2) -> Vec<block_plan::BlockDiagPlan<'a>> {
        let mut field_plans: Vec<block_plan::BlockDiagPlan> = Vec::with_capacity(fields.len());
        let mut deltas = Vec2::default();

        for f in fields {
            let w = self.field_width(&f,false)
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

            if let Some(body_plan) = &mut f_plan.body_plan {
                _ = body_plan.relative_pos.insert(deltas);
            };

            field_plans.push(f_plan);
            deltas.x += w;
        }

        field_plans
    }

    pub fn make_plan<'kind>(
        &'kind self,
        kind: &'kind Kind<'kind>,
        mins: Vec2,
        width: Option<f32>,
        notch: bool,
    ) -> block_plan::BlockDiagPlan<'kind>
    {
        let block_width = width.unwrap_or(self.unlabeled_width(kind,notch));
        let block_height = self.height(kind);

        let body_plan = block_plan::BlockBodyPlan {
            block_width,
            notch,
            relative_pos: None,
        };

        let fields = match kind {
            Kind::Composite(comp) => match comp.mode {
                CompositeMode::Product => self.plan_product_fields(
                    &comp.fields,
                    mins,
                    self.member_width(kind)
                ),
                CompositeMode::Sum => self.plan_sum_fields(
                    &comp.fields,
                    mins
                ),
            },
            _ => Default::default(),
        };

        let gapped = match kind {
            Kind::Composite(_) => true,
            _ => false,
        };

        let maxs = mins + Vec2::new(block_width,block_height);

        let member_width = self.member_width(kind);
        let prong_padding = if gapped { self.prong_xpad } else { 0.0 };
        let header_offset = member_width + prong_padding;
        let head = self
            .draw_header(
                &kind.to_string(),
                block_width - member_width - prong_padding,
                notch,
            )
            .set("transform", Translate(header_offset, 0.0));

        block_plan::BlockDiagPlan {
            spec: self,
            head,
            body_plan: Some(body_plan),
            mins,
            maxs,
            kind,
            sub_blocks: fields,
            graph_index: None,
        }
    }
}

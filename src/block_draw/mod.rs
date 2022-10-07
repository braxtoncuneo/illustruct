use std::iter::Iterator;
use unicode_width::UnicodeWidthStr;
use svg::node::element::{
    Text, Path, Group,
    path::Data
};

use crate::kind::{Kind, composite::{self, Field, Composite}, array::Array, CType};

pub mod block_plan;
pub mod util;

use util::Vec2;

use self::block_plan::BlockDiagPlan;

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
    fn label_height(&self) -> f32 {
        self.char_dims.y + self.text_pads.y * 2.0
    }

    pub fn line_height(&self) -> f32 {
        self.label_height() + self.label_pads.y * 2.0
    }

    fn bare_width(&self, text: &str) -> f32 {
        text.width_cjk() as f32 * self.char_dims.x
    }

    fn padded_width(&self, text: &str) -> f32 {
        self.bare_width(text) + self.text_pads.x * 2.0
    }

    pub fn label_width(&self, text: &str) -> f32 {
        self.padded_width(text) + self.label_pads.x * 2.0
    }

    pub fn byte_width(&self) -> f32 {
        self.padded_width("01234567") + self.prong_width * 2.0
    }

    pub fn repr_width(&self) -> f32 {
        self.padded_width("0123456789") + self.prong_width * 2.0
    }

    fn draw_header(&self, title: impl ToString, width: f32, notch: bool) -> Group {
        let text = title.to_string();
        let half_height = self.line_height() / 2.0;

        let prong_line = Vec2::new(
            self.prong_width,
            half_height,
        );
        let notch_line = Vec2::new(
            notch.then_some(self.prong_width).unwrap_or_default(),
            half_height,
        );

        let path = Path::new()
            .set("fill", "black")
            .set("stroke", "none")
            .set("d", Data::new()
                .move_to(Vec2::ZERO)
                .line_by(Vec2::Q2 * prong_line)
                .line_by(Vec2::Q1 * prong_line)
                .line_by(Vec2::x(width))
                .line_by(Vec2::Q3 * notch_line)
                .line_by(Vec2::Q4 * notch_line)
                .close(),
            );

        let padded_width = self.padded_width(&text);
        let text = Text::new()
            .add(svg::node::Text::new(text))
            .set("fill", "white")
            .set("font-family", "monospace")
            .set("font-size", self.char_dims.y)
            .set("dominant-baseline", "middle")
            .set("text-anchor", "middle")
            .set("x", padded_width / 2.0)
            .set("y", half_height);

        Group::new().add(path).add(text)
    }

    pub fn draw_block(&self, kind: &Kind, width: f32, notch: bool) -> Option<Group> {
        let size = kind.size_of();
        if size <= 1 { return None; }

        let head_height = self.line_height();
        let half_head = head_height / 2.0;
        let height = self.line_height() * size as f32 + self.fill_inset;
        let rem_height = height - head_height - self.chamfer_size;

        let width_chopped = width - self.chamfer_size;
        let notch_x = notch.then_some(self.prong_width).unwrap_or_default();
        let notch_line = Vec2::new(notch_x, half_head);
        let half_inset = self.fill_inset / 2.0;
        let double_inset = self.fill_inset * 2.0;

        let path = Path::new()
            .set("fill", "black")
            .set("stroke", "none")
            .set("d", Data::new()
                .move_to(Vec2::ZERO)
                .line_by(Vec2::y(height))
                .line_by(Vec2::x(width_chopped))
                .line_by(Vec2::Q4 * self.chamfer_size)
                .line_by(Vec2::y(-rem_height))
                .line_by(Vec2::Q3 * notch_line)
                .line_by(Vec2::Q4 * notch_line)
                .close(),
            );

        let fill_path = Path::new()
            //.set("fill", "grey")
            .set("stroke", "none")
            .set("d", Data::new()
                .move_to(Vec2::squared(self.fill_inset))
                .line_by(Vec2::y(height - double_inset))
                .line_by(Vec2::x(width_chopped - half_inset * 3.0))
                .line_by(Vec2::Q4 * (self.chamfer_size - half_inset))
                .line_by(Vec2::y(-rem_height + self.fill_inset))
                .line_by(Vec2::Q3 * (notch_line - Vec2::y(half_inset)))
                .line_by(Vec2::Q4 * (notch_line + Vec2::y(self.fill_inset)))
                .close(),
            );

        Some(Group::new().add(path).add(fill_path))
    }

    fn prong_line(&self) -> Path {
        let half_height = self.line_height() / 2.0;
        let slope    = half_height / self.prong_width;
        let start_x  = -slope * self.fill_inset;
        let top_dif = Vec2::new(
            self.prong_width + start_x,
            half_height - self.fill_inset,
        );
        Path::new()
            .set("fill", "grey")
            .set("stroke", "none")
            .set("d", Data::new()
                .move_to(Vec2::squared(self.fill_inset) + Vec2::x(start_x))
                .line_by(Vec2::Q2 * top_dif)
                .line_by(Vec2::Q1 * Vec2::new(self.prong_width, half_height))
                .line_by(Vec2::x(self.fill_inset + self.prong_xpad))
                .line_by(Vec2::Q3 * Vec2::new(self.prong_width, half_height))
                .line_by(Vec2::Q4 * top_dif)
                .close(),
            )
    }

    pub fn draw_label(&self, text: &str) -> Group {
        let dims = Vec2::new(
            self.bare_width(text),
            self.label_height(),
        );

        let path = Path::new()
            .set("fill", "white")
            .set("stroke", "none")
            .set("d", Data::new()
                .move_to(Vec2::ZERO)
                .elliptical_arc_by((self.text_pads.x,self.text_pads.y,0,0,0,0,dims.y))
                //.line_by((         0,   height))
                .line_by(Vec2::x(dims.x))
                .elliptical_arc_by((self.text_pads.x,self.text_pads.y,0,0,0,0,-dims.y))
                //.line_by((         0,  -height))
                .close(),
            );

        let text_node = Text::new()
            .add(svg::node::Text::new(text))
            .set("fill", "black")
            .set("font-family", "monospace")
            .set("font-size", self.char_dims.y)
            .set("dominant-baseline", "middle")
            .set("text-anchor", "middle")
            .set("x", dims.x / 2.0)
            .set("y", dims.y / 2.0);

        Group::new().add(path).add(text_node)
    }

    pub fn name_width<'kind>(&self, kind: &Kind<'kind> ) -> f32 {
        self.padded_width(&kind.to_string())
    }

    pub fn composite_member_width<'kind>(&self,comp: &Composite<'kind>) -> f32 {
        match comp.mode {
            composite::Mode::Product => comp.fields.borrow().iter()
                .enumerate()
                .map(|(i, f)| self.field_width(f, i==0))
                .max_by(f32::total_cmp)
                .unwrap_or_default()
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
        if size == 0 { return 0.0; }

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
        self.line_height() * kind.size_of() as f32
    }

    pub fn field_width(&self, field: &Field, notch: bool) -> f32 {
        self.unlabeled_width(field.kind, notch)
            + self.label_width(field.name.as_deref().unwrap_or_default())
            + self.label_pads.x * 2.0
    }

    pub fn plan_primitive<'kind>(
        &self,
        kind: &'kind Kind<'kind>,
        mins: Vec2,
        width: Option<f32>,
        with_notch: bool,
    ) -> BlockDiagPlan<'kind> {
        let dims = Vec2::new(
            width.unwrap_or_else(|| self.unlabeled_width(kind, with_notch)),
            self.height(kind),
        );

        BlockDiagPlan {
            spec: *self,
            head: self.draw_header(
                &kind.to_string(),
                width.unwrap_or_else(|| self.unlabeled_width(kind, with_notch)),
                with_notch,
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
    ) -> Vec<BlockDiagPlan<'kind>> {
        let mut field_plans = Vec::new();
        let mut offset = 0;
        let mut deltas = Vec2::ZERO;

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
                mins + deltas,
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
    ) -> Vec<BlockDiagPlan<'kind>> {
        let mut field_plans = Vec::with_capacity(fields.len());
        let mut offset = 0;
        let mut deltas = Vec2::ZERO;

        for field in fields {
            let pad = field.kind.align_pad(offset);
            let pad_height = pad as f32 * self.line_height();
            let size = field.kind.size_of();
            let size_height = size as f32 * self.line_height();

            deltas.y += pad_height;

            println!("{:?}", deltas + mins);

            let has_notch = offset == 0;

            let mut f_plan = field.make_plan(
                self,
                mins+deltas,
                Some(width),
                has_notch
            );

            f_plan.relative_pos = Some(deltas);

            field_plans.push(f_plan);
            deltas.y += size_height;
            offset += size + pad;
        }

        field_plans
    }

    pub fn plan_sum_fields<'kind>(
        &self,
        fields: &Vec<Field<'kind>>,
        mins: Vec2,
    ) -> Vec<BlockDiagPlan<'kind>> {
        const NOTCH: bool = false;
        let mut field_plans = Vec::with_capacity(fields.len());
        let mut deltas = Vec2::ZERO;

        for field in fields {
            let w = self.field_width(field, NOTCH)
                + self.union_xpad
                + self.prong_width
                + self.prong_xpad;

            let mut f_plan = field.make_plan(
                self,
                mins + deltas,
                None,
                NOTCH
            );

            f_plan.relative_pos = Some(deltas);

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
        with_notch: bool,
    ) -> BlockDiagPlan<'kind> {
        let block_width = width.unwrap_or_else(|| self.unlabeled_width(kind, with_notch));

        let (head, head_offset) = {
            let member_width = self.member_width(kind);
            let prong_padding = match kind {
                Kind::Composite(_) | Kind::Array(_) => self.prong_xpad,
                _ => 0.0,
            };
            let head = self.draw_header(
                &kind.to_string(),
                block_width - member_width - prong_padding,
                with_notch,
            );

            (head, member_width + prong_padding)
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

        BlockDiagPlan {
            spec: *self,
            head,
            head_offset,
            relative_pos: None,
            body_plan: Some(block_plan::BlockBodyPlan {
                block_width,
                notch: with_notch,
            }),
            mins,
            maxs: mins + Vec2::new(block_width, self.height(kind)),
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
    ) -> BlockDiagPlan<'kind> {
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

        let head = Group::new();
        let head_offset = self.member_width(kind) + self.prong_xpad;
        let maxs = mins + Vec2::new(width, self.height(kind));

        BlockDiagPlan {
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

#![allow(dead_code)]

use std::{iter, fmt};
use svg::{node::element::{Group, Text, path::Data, Path as SvgPath}, Document};

use crate::{
    block_draw::{util::{Vec2, Translate, Bounds}, BlockDrawSpec},
    kind::{
        primitive::{Primitive, PrimValue},
        composite::{Field, Composite},
        CType,
    },
    access::{self, Indirection, Trace},
};

pub enum Segment<'kind> {
    Chop(Vec2),
    Skip(usize,bool),
    Span(Composite<'kind>),
}

#[non_exhaustive]
#[derive(Clone)]
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

impl fmt::Display for MemByte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemByte::Undefined   => write!(f, "----"),
            MemByte::OutOfBounds => write!(f, "OOB"),
            MemByte::Byte(x)     => write!(f, "{x:08b}"),
        }
    }
}

pub struct MemRibbon <'kind> {
    pub base_adr: usize,
    pub segments: Vec<Segment<'kind>>,
    pub data: Vec<MemByte>,
}

impl<'kind> MemRibbon<'kind> {
    pub fn new(base_adr: usize) -> Self {
        MemRibbon {
            base_adr,
            segments: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn chop(mut self, offset: Vec2) -> Self {
        self.segments.push(Segment::Chop(offset));
        self
    }

    pub fn skip(&mut self, offset: usize) {
        println!("Skipping by {offset}");
        self.segments.push(Segment::Skip(offset,false));
        self.data.extend(iter::repeat(MemByte::Undefined).take(offset));
    }

    pub fn ellipse(mut self, offset: usize) -> Self {
        println!("Skipping by {offset}");
        self.segments.push(Segment::Skip(offset,true));
        self.data.extend(iter::repeat(MemByte::Undefined).take(offset));
        self
    }

    pub fn span(mut self, name: impl ToString, fields: Vec<Field<'kind>>) -> Self {
        let end_adr = self.base_adr + self.data.len();

        let comp = Composite::product(name, fields);

        let align = comp.align_of() as usize;
        let align_rem = end_adr % align;

        if align_rem != 0 {
            self.skip(align - align_rem);
        }

        let size = comp.size_of_no_end_pad() as usize;
        self.data.extend(iter::repeat(MemByte::Undefined).take(size));
        self.segments.push(Segment::Span(comp));

        self
    }

    pub fn get(&'kind self, mut path: access::Path) -> access::Result<'kind> {
        let field_name = match path.pop_front() {
            Some(Indirection::Field(field_name)) => Ok(field_name),
            Some(indirection) => Err(access::Error::at(
                "MemRibbon",
                access::ErrorKind::ribbon_op(&indirection),
            )),
            None => Err(access::Error::at(
                "MemRibbon",
                access::ErrorKind::DirectAccess,
            )),
        }?;

        let mut address = self.base_adr;
        let mut span_comp = None;

        for seg in self.segments.iter() {
            address += match seg {
                Segment::Chop(_) => 0,
                Segment::Skip(skip, _) => *skip,
                Segment::Span(comp) =>
                    if comp.name == field_name {
                        span_comp = Some(comp);
                        break;
                    } else {
                        comp.size_of_no_end_pad() as usize
                    }
            }
        }

        let span_comp = span_comp.ok_or_else(|| access::Error::at(
            "MemRibbon",
            access::ErrorKind::SubField { name: field_name },
        ))?;

        let indirection = path.pop_front().ok_or_else(|| access::Error::at(
            "MemRibbonSpan",
            access::ErrorKind::DirectAccess,
        ))?;

        let name = indirection.as_field().ok_or_else(|| access::Error::at(
            "MemRibbonSpan",
            access::ErrorKind::ribbon_op(&indirection),
        ))?;

        span_comp.access_with(
            Indirection::Field(name.into()),
            Trace {
                ribbon: self,
                path,
                address,
                field_name: name.into(),
            },
        )
    }

    pub fn at(&'kind self, access_string: &str) -> access::PlaceValue {
        self.get(access_string.parse().unwrap()).unwrap()
    }

    pub fn size_of(&self, _access: &str) -> Option<usize> {
        todo!()
    }

    pub fn align_of(&self, _access: &str) -> Option<usize> {
        todo!()
    }

    pub fn address_of(&self, _access: &str) -> Option<usize> {
        todo!()
    }

    pub fn bytes_at(&self, address: usize, size: usize) -> Option<Vec<u8>> {
        let address = address.checked_sub(self.base_adr)?;

        self.data.get(address..address+size)?.iter()
            .map(MemByte::byte)
            .collect()
    }

    pub fn write_at(&mut self, address: usize, value: PrimValue) {
        let (ribbon_skip, prim_skip) = if address < self.base_adr {
            (0, self.base_adr - address)
        } else {
            (address - self.base_adr, 0)
        };

        self.data.iter_mut()
            .skip(ribbon_skip)
            // .map(MemByte::writable)
            .zip(value.bytes().iter().skip(prim_skip))
            .for_each(|(dst,src)|*dst.writable() = *src )
    }

    pub fn memcpy(&self, _src: usize, _dst: usize, _size: usize) {
        todo!()
    }

    pub fn draw(
        &self,
        position: Vec2,
        spec: &BlockDrawSpec,
        show_data: bool,
        show_kind: bool,
    ) -> (Group, Bounds) {
        let mut nozzle = Nozzle {
            address: self.base_adr,
            position, // Vec2::ZERO,
            bounds: Bounds::closed_at(position),
            show_data,
            show_kind,
        };

        let width = self.segments.iter()
            .map(|seg| match seg {
                Segment::Span(comp) => spec.composite_member_width(comp),
                _ => 0.0,
            })
            .max_by(f32::total_cmp)
            .unwrap_or_default();

        let result = self.segments.iter()
            .map(|seg| match seg {
                Segment::Chop(offset) => nozzle.draw_chop(*offset),
                Segment::Skip(offset, ellipse) => nozzle.draw_skip(spec, *offset, *ellipse),
                Segment::Span(comp) => nozzle.draw_span(self, spec, comp, width),
            })
            .fold(Group::new(), Group::add);

        (result, nozzle.bounds)
    }

    pub fn save_svg(
        &self,
        file_name: impl AsRef<std::path::Path>,
        spec: &BlockDrawSpec,
        show_data: bool,
        show_kind: bool,
    ) {
        let (group, bounds) = self.draw(Vec2::ZERO, spec, show_data, show_kind);

        let document = Document::new()
            .set("viewBox", bounds.svg_viewbox())
            .add(group);

        svg::save(file_name, &document).unwrap();
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Nozzle {
    position: Vec2,
    bounds: Bounds,
    address: usize,
    show_data: bool,
    show_kind: bool,
}

fn rectangle(position: Vec2, size: Vec2) -> Data {
    let Vec2 { x: x1, y: y1 } = position;
    let Vec2 { x: x2, y: y2 } = position + size;
    Data::new()
        .move_to((x1, y1))
        .line_to((x1, y2))
        .line_to((x2, y2))
        .line_to((x2, y1))
        .close()
}

impl Nozzle {
    fn subnozzle(&self, position: Vec2) -> Self {
        Self { position, ..*self }
    }

    pub fn draw_box(&mut self, dims: Vec2, spec: &BlockDrawSpec, text: String) -> Group {
        let inside = dims - Vec2::y(spec.fill_inset);
        let outside = dims + Vec2::new(2.0, 1.0) * spec.fill_inset;

        let path = SvgPath::new()
            .set("fill", "black")
            .set("stroke", "none")
            .set("d", rectangle(Vec2::ZERO, outside));

        let fill_path = SvgPath::new()
            .set("fill", "white")
            .set("stroke", "none")
            .set("d", rectangle(Vec2::squared(spec.fill_inset), inside));

        let text_node = Text::new()
            .add(svg::node::Text::new(text))
            .set("fill", "black")
            .set("font-family", "monospace")
            .set("font-size", spec.char_dims.y)
            .set("dominant-baseline", "middle")
            .set("text-anchor", "middle")
            .set("x", outside.x / 2.0)
            .set("y", outside.y / 2.0);

        let result = Group::new()
            .add(path)
            .add(fill_path)
            .add(text_node)
            .set("transform", Translate::from(self.position));

        self.move_by(Vec2::y(dims.y));

        result
    }

    pub fn draw_flag(&mut self, dims: Vec2, spec: &BlockDrawSpec, text: String) -> Group {
        let inside = dims - Vec2::y(spec.fill_inset);
        let outside = dims + Vec2::new(2.0, 1.0) * spec.fill_inset;
        let prong_line = Vec2::new(spec.prong_width, spec.line_height() / 2.0);

        let path = SvgPath::new()
            .set("fill", "black")
            .set("stroke", "none")
            .set("d", Data::new()
                .move_to(Vec2::ZERO)
                .elliptical_arc_by((spec.text_pads.x,outside.y / 2.0,0,0,0,0,outside.y))
                //.elliptical_arc_by((hho,hho,0,0,0,0,ho))
                //.line_by((  0,         ho))
                .line_by(Vec2::x(outside.x))
                //.line_by((  p, -ho*0.5f32))
                //.line_by(( -p, -ho*0.5f32))
                .line_by(Vec2::Q2 * Vec2::y(outside.y + spec.line_height()))
                .line_by(Vec2::Q4 * prong_line)
                .line_by(Vec2::Q3 * prong_line)
                .close(),
            );

        let fill_path = SvgPath::new()
            .set("fill", "#EEE")
            .set("stroke", "none")
            .set("d", Data::new()
                .move_to(Vec2::squared(spec.fill_inset))
                .elliptical_arc_by((spec.text_pads.x, inside.y / 2.0,0,0,0,0,inside.y))
                //.elliptical_arc_by((hhi,hhi,0,0,0,0,hi))
                //.line_by((  0,         hi))
                .line_by(Vec2::x(inside.x))
                //.line_by((  p, -hi*0.5f32))
                //.line_by(( -p, -hi*0.5f32))
                .line_by(Vec2::y(-inside.y + spec.line_height() - spec.fill_inset * 1.5))
                .line_by(Vec2::Q4 * (prong_line - Vec2::squared(spec.fill_inset / 2.0)))
                .line_by(Vec2::Q3 * (prong_line - Vec2::squared(spec.fill_inset)))
                .close(),
            );

        let text_node = Text::new()
            .add(svg::node::Text::new(text))
            .set("fill", "black")
            .set("font-family", "Noto Serif")
            .set("font-size", spec.char_dims.y)
            .set("dominant-baseline", "middle")
            .set("text-anchor", "middle")
            .set("x", outside.x / 2.0)
            .set("y", outside.y / 2.0);

        let result = Group::new()
            .add(path)
            .add(fill_path)
            .add(text_node)
            .set("transform", Translate::from(self.position));

        self.move_by(Vec2::y(dims.y));

        result
    }

    pub fn draw_byte(&mut self, spec: &BlockDrawSpec, text: String) -> Group {
        let dims = Vec2::new(
            spec.byte_width(),
            spec.line_height(),
        );

        self.draw_box(dims, spec, text)
    }

    pub fn draw_chop(&mut self, offset: Vec2) -> Group {
        self.move_by(offset);
        Group::new()
    }

    pub fn draw_skip(&mut self, spec: &BlockDrawSpec, offset: usize, ellipse: bool) -> Group {
        self.increment_address(offset);

        if ellipse {
            self.draw_byte(spec, "...".to_string())
        } else {
            self.draw_byte(spec, format!("({offset}B)"))
        }
    }

    pub fn draw_repr(&mut self, ribbon: &MemRibbon, address: usize, prim: Primitive, spec: &BlockDrawSpec) -> Group {
        let dims = Vec2::new(
            spec.repr_width(),
            spec.line_height() * prim.size_of() as f32,
        );
        let text = prim.parse_at(ribbon, address)
            .map(|x| x.to_string())
            .unwrap_or_else(|| "???".to_string());

        self.draw_flag(dims, spec, text)
    }

    pub fn draw_span(
        &mut self,
        ribbon: &MemRibbon,
        spec: &BlockDrawSpec,
        comp: &Composite,
        width: f32,
    ) -> Group {
        let mut start_address = self.address;
        let mut result = Group::new();

        let mut byte_noz = self.subnozzle(self.position);

        // let right_grp = spec.make_span_plan(kind, mins, width);

        for field in comp.fields.borrow().iter() {
            let padding = field.kind.align_pad(start_address as u16);
            let field_address = start_address + padding as usize;

            let vertical_offset = (field_address - self.address) as f32 * spec.line_height();
            let kind_tform = Vec2::new(spec.byte_width(), vertical_offset) + self.position;

            // let width = spec.composite_member_width(comp);
            let kind_grp = field.make_plan(spec, kind_tform, Some(width), false)
                .into_svg()
                .set("transform",Translate::from(kind_tform));

            let mut repr_group = Group::new();
            let mut repr_address = field_address;
            let mut sub_noz = self.subnozzle(Vec2::y(vertical_offset) + self.position);
            for (adr, prim) in field.kind.base_fields(&mut repr_address) {
                let prim_group = sub_noz.draw_repr(ribbon, adr, prim, spec);
                repr_group = repr_group.add(prim_group);
            }
            repr_group = repr_group
                .set("transform", Translate::from(Vec2::x(-spec.repr_width() - spec.fill_inset)));

            let span_size = (padding + field.kind.size_of()) as usize;
            result = ribbon.data.iter()
                .skip(start_address - ribbon.base_adr)
                .take(span_size)
                .map(|byte| byte_noz.draw_byte(spec, byte.to_string()))
                .fold(result, Group::add);

            start_address += span_size;

            if self.show_data {
                result = result.add(repr_group);
            }

            if self.show_kind {
                result = result.add(kind_grp);
            }
        }

        let skip = start_address - self.address;
        self.increment_address(skip);

        let y_skip = skip as f32 * spec.line_height();

        let data_width = self.show_data.then_some(
            spec.repr_width()
                + spec.line_height() / 2.0
                + spec.fill_inset * 1.5
        ).unwrap_or_default();

        let kind_width = self.show_kind.then_some(width).unwrap_or_default();

        self.bounds.expand(Bounds {
            min: self.position - Vec2::x(data_width),
            max: self.position + Vec2::new(
                spec.byte_width() + kind_width,
                y_skip + spec.fill_inset,
            ),
        });

        // result = result.set("transform",Translate::from(self.position));
        self.move_by(Vec2::y(y_skip));
        result
    }

    fn move_by(&mut self, delta: Vec2) {
        self.position += delta;
    }

    fn increment_address(&mut self, delta: usize) {
        self.address += delta;
    }
}
#![allow(dead_code)]

use std::{iter, cell::RefCell};

use svg::{node::element::{Group, Text, Path, path::Data,}, Document};

use crate::{
    block_draw::{util::{Vec2, Translate}, BlockDrawSpec},
    kind::{primitive::{Primitive, PrimValue}, composite::{Field, Composite, CompositeMode}},
    access::{self, MemByte, PlaceValue, AccessTrace, AccessPath},
};


pub enum MemRibbonSegment<'kind> {
    Chop(Vec2),
    Skip(usize,bool),
    Span(Composite<'kind>),
}




pub struct MemRibbon <'kind> {
    pub base_adr: usize,
    pub segments: Vec<MemRibbonSegment<'kind>>,
    pub data: Vec<MemByte>,
}

impl <'kind> MemRibbon <'kind> {
    pub fn new(base_adr: usize) -> Self {
        MemRibbon { base_adr, segments: Vec::new(), data: Vec::new() }
    }

    pub fn chop(&mut self, offset: Vec2) {
        self.segments.push(MemRibbonSegment::Chop(offset));
    }

    pub fn skip(&mut self, offset: usize) {
        println!("Skipping by {offset}");
        self.segments.push(MemRibbonSegment::Skip(offset,false));
        self.data.extend(iter::repeat(MemByte::Undefined).take(offset));
    }

    pub fn ellipse(&mut self, offset: usize) {
        println!("Skipping by {offset}");
        self.segments.push(MemRibbonSegment::Skip(offset,true));
        self.data.extend(iter::repeat(MemByte::Undefined).take(offset));
    }

    pub fn span<T: ToString>(&mut self, name: T, fields: Vec<Field<'kind>>) {
        let end_adr = self.base_adr + self.data.len();

        let comp = Composite {
            name: name.to_string(),
            mode: CompositeMode::Product,
            fields: RefCell::new(fields),
        };

        let align = comp.align_of() as usize;
        let align_rem = end_adr % align;

        if align_rem != 0 {
            self.skip(align-align_rem);
        }

        let size = comp.size_of_no_end_pad() as usize;
        self.data.extend(iter::repeat(MemByte::Undefined).take(size));
        self.segments.push(MemRibbonSegment::Span(comp));
    }

    pub fn get(&'kind self, access: AccessPath) -> Result<PlaceValue, access::Error> {
        use MemRibbonSegment::*;
        use access::AccessUnit::*;

        let mut access = access.0;

        let span_name = match access.pop_front() {
            Some(Field(field_name)) => field_name,
            Some(op) => return Err(access::Error {
                field_name: "MemRibbon".to_string(),
                kind: access::ErrorKind::RibbonOp { op: op.op_str() },
                context: None,
            }),
            None => return Err(access::Error {
                field_name: "MemRibbon".to_string(),
                kind: access::ErrorKind::DirectAccess,
                context: None,
            }),
        };

        let mut address = self.base_adr;
        let mut span_comp = None;

        for seg in self.segments.iter() {
            address += match seg {
                Chop(_) => 0,
                Skip(skip,_) => *skip,
                Span(comp) => 
                    if comp.name == span_name {
                        span_comp = Some(comp);
                        break;
                    } else {
                        comp.size_of_no_end_pad() as usize
                    }
            }
        }

        let span_comp = match span_comp {
            Some(comp) => comp,
            None => return Err(access::Error{
                field_name: "MemRibbon".to_string(),
                kind: access::ErrorKind::SubField { name: span_name },
                context: None,
            }),
        };


        let unit = match access.pop_front() {
            Some(unit) => unit,
            None => return Err(access::Error{
                field_name: "MemRibbonSpan".to_string(),
                kind: access::ErrorKind::DirectAccess,
                context: None,
            }),
        };

        let (field,name) = match &unit {
            field @ Field(name) => (field,name.clone()),
            op => return Err(access::Error{
                field_name: "MemRibbonSpan".to_string(),
                kind: access::ErrorKind::RibbonOp { op: op.op_str() },
                context: None,
            }),
        };

        let mut trace = AccessTrace {
            ribbon: self,
            access,
            address,
            field_name: name,
        };

        span_comp.access(field, &mut trace)
    }

    pub fn at(&'kind self, access_string: &str) -> PlaceValue {
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
        &'kind self, 
        position: Vec2, 
        spec: &'kind BlockDrawSpec, 
        show_data: bool, 
        show_kind: bool,
    ) -> (Group, (Vec2, Vec2)) {
        use MemRibbonSegment::*;

        let mut nozzle = Nozzle {
            address: self.base_adr, 
            position, // Vec2::default(),
            mins: position,
            maxs: position,
            show_data,
            show_kind,
        };

        let width = self.segments.iter()
            .map(|seg| match seg {
                Chop(_)    => 0.0f32,
                Skip(_,_)  => 0.0f32,
                Span(comp) => spec.composite_member_width(comp)
            })
            .fold( 0.0f32, |acc,val| acc.max(val) );

        let result = self.segments.iter()
            .map(|seg| match seg {
                &Chop(offset) => nozzle.draw_chop(offset),
                &Skip(offset,ellipse) => nozzle.draw_skip(spec, offset, ellipse),
                Span(comp) => nozzle.draw_span(self, spec, comp, width),
            })
            .fold(Group::new(), Group::add);

        (result, (nozzle.mins, nozzle.maxs))
    }

    pub fn save_svg<T: ToString>(&'kind self,file_name: T, spec:&'kind BlockDrawSpec, show_data: bool, show_kind: bool) {
        let (group,bounds) = self.draw(Vec2::default(), spec, show_data, show_kind);

        let document = Document::new()
            .set("viewBox", (bounds.0.x, bounds.0.y, bounds.1.x-bounds.0.x, bounds.1.y-bounds.0.y))
            .add(group);

        svg::save(file_name.to_string(), &document).unwrap();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
struct Nozzle {
    position: Vec2,
    mins: Vec2,
    maxs: Vec2,
    address: usize,
    show_data: bool,
    show_kind: bool,
}

impl Nozzle {
    fn subnozzle(&self, position: Vec2) -> Self {
        Self { position, ..*self }
    }

    pub fn draw_box(&mut self, w: f32, h: f32, spec: &BlockDrawSpec, text: String) -> Group {
        let i = spec.fill_inset;
        let hi = h - i;
        let wi = w;
        let ho = h + i;
        let wo = w + i * 2f32;

        let data = Data::new()
            .move_to(( 0,  0))
            .line_by(( 0, ho))
            .line_by((wo,  0))
            .line_by(( 0,-ho))
            .close();

        let path = Path::new()
            .set("fill", "black")
            .set("stroke", "none")
            .set("d", data);

        let fill_data = Data::new()
            .move_to(( i,  i))
            .line_by(( 0,  hi))
            .line_by((wi,  0))
            .line_by(( 0,-hi))
            .close();

        let fill_path = Path::new()
            .set("fill", "white")
            .set("stroke", "none")
            .set("d", fill_data);

        let text_node = Text::new()
            .add(svg::node::Text::new(text))
            .set("fill", "black")
            .set("font-family", "monospace")
            .set("font-size", spec.char_dims.y)
            .set("dominant-baseline", "middle")
            .set("text-anchor", "middle")
            .set("x", wo / 2.0)
            .set("y", ho / 2.0);

        let result = Group::new()
            .add(path)
            .add(fill_path)
            .add(text_node)
            .set("transform", Translate::from(self.position));

        self.move_by(Vec2::y(h));

        result
    }

    pub fn draw_flag(&mut self, w: f32, h: f32, spec: &BlockDrawSpec, text: String) -> Group {
        let p = spec.prong_width;
        let i = spec.fill_inset;
        let hi = h - i;
        let wi = w;
        let ho = h + i;
        let wo = w + i * 2.0;
        let l  = spec.line_height();

        let hho = ho * 0.5;
        let hhi = hi * 0.5;

        let data = Data::new()
            .move_to((  0,          0))
            .elliptical_arc_by((spec.text_pads.x,hho,0,0,0,0,ho))
            //.elliptical_arc_by((hho,hho,0,0,0,0,ho))
            //.line_by((  0,         ho))
            .line_by(( wo,          0))
            //.line_by((  p, -ho*0.5f32))
            //.line_by(( -p, -ho*0.5f32))
            .line_by((  0, -ho+l))
            .line_by((  p, -l*0.5f32))
            .line_by(( -p, -l*0.5f32))
            .close();

        let path = Path::new()
            .set("fill", "black")
            .set("stroke", "none")
            .set("d", data);

        let fill_data = Data::new()
            .move_to((  i,          i))
            .elliptical_arc_by((spec.text_pads.x,hhi,0,0,0,0,hi))
            //.elliptical_arc_by((hhi,hhi,0,0,0,0,hi))
            //.line_by((  0,         hi))
            .line_by(( wi,          0))
            //.line_by((  p, -hi*0.5f32))
            //.line_by(( -p, -hi*0.5f32))
            .line_by((  0, -hi+l-i*1.5f32))
            .line_by((  p-i*0.5f32, -l*0.5f32+i*0.5f32))
            .line_by(( -p+i, -l*0.5f32+i))
            .close();

        let fill_path = Path::new()
            .set("fill", "#EEE")
            .set("stroke", "none")
            .set("d", fill_data);

        let text_node = Text::new()
            .add(svg::node::Text::new(text))
            .set("fill", "black")
            .set("font-family", "Noto Serif")
            .set("font-size", spec.char_dims.y)
            .set("dominant-baseline", "middle")
            .set("text-anchor", "middle")
            .set("x", wo / 2.0)
            .set("y", ho / 2.0);

        let result = Group::new()
            .add(path)
            .add(fill_path)
            .add(text_node)
            .set("transform", Translate::from(self.position));

        self.move_by(Vec2::y(h));

        result
    }

    pub fn draw_byte(&mut self, spec: &BlockDrawSpec, text: String) -> Group {
        let h = spec.line_height();
        let w = spec.byte_width();

        self.draw_box(w, h, spec, text)
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
        let w = spec.repr_width();
        let h = spec.line_height() * prim.size_of() as f32;
        let text = prim.parse_at(ribbon, address)
            .map(|x| x.to_string())
            .unwrap_or_else(|| "???".to_string());

        self.draw_flag(w,h,spec,text)
    }

    pub fn draw_span<'a>(
        &mut self, 
        ribbon: &MemRibbon,
        spec: &'a BlockDrawSpec, 
        comp: &'a Composite<'a>,
        width: f32,
    ) -> Group {
        let mut start_address = self.address;
        let mut result = Group::new();

        let mut byte_noz = self.subnozzle(self.position);

        // let right_grp = spec.make_span_plan(kind, mins, width);

        for field in comp.fields.borrow().iter() {
            let padding = field.kind.align_pad(start_address as u16);
            let field_address = start_address + padding as usize;
            let span_size = ( padding + field.kind.size_of() ) as usize;


            let vertical_offset = (field_address - self.address) as f32 * spec.line_height();
            let kind_tform = Vec2::new(spec.byte_width(),vertical_offset) + self.position;

            // let width = spec.composite_member_width(comp);
            let kind_grp = field.make_plan(spec, kind_tform, Some(width), false)
                .into_svg()
                .set("transform",Translate::from(kind_tform));

            let mut repr_group = Group::new();
            let mut repr_address = field_address;
            let mut sub_noz = self.subnozzle(Vec2::new(0f32,vertical_offset)+self.position);
            for (adr,prim) in field.kind.base_fields(&mut repr_address).into_iter() {
                let prim_group = sub_noz.draw_repr(ribbon, adr, prim, spec);
                repr_group = repr_group.add(prim_group);
            }
            repr_group = repr_group.set("transform",Translate(-spec.repr_width()-spec.fill_inset,0f32));


            for byte in ribbon.data.iter().skip(start_address-ribbon.base_adr).take(span_size) {
                result = result.add(byte_noz.draw_byte(spec, format!("{}",byte)));
            }

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

        let data_width = if self.show_data {
            spec.repr_width() + spec.line_height() * 0.5 + spec.fill_inset * 1.5 
        } else { 0.0 };

        let mins = self.position - Vec2::new(data_width, 0.0);

        let kind_width = if self.show_kind { width } else { 0.0 };

        let maxs = self.position + Vec2::new(spec.byte_width()+kind_width,y_skip+spec.fill_inset);

        self.mins = self.mins.min(mins);
        self.maxs = self.maxs.max(maxs);
        
        // result = result.set("transform",Translate::from(self.position));
        self.move_by(Vec2::new(0.0, y_skip));
        result
    }

    fn move_by(&mut self, delta: Vec2) {
        self.position += delta;
    }

    fn increment_address(&mut self, delta: usize) {
        self.address += delta;
    }
}
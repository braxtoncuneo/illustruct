use std::ops::Add;

use svg::node::element::Group;

use crate::{
    block_draw::Vec2,
    kind::{
        Field, Kind, PrimValue, Composite,
    }, access::MemByte
};

use pod::Pod;




pub enum MemRibbonSegment
{
    Chop  (    Vec2),
    Skip  (   usize),
    Span  (   usize),
}




pub struct MemRibbon <'kind>{
    pub base_adr: usize,
    pub base_pos: Vec2,
    pub segments: Vec<MemRibbonSegment>,
    pub data:     Vec<MemByte>,
    pub kind:     Composite<'kind>,
}


impl <'kind> MemRibbon <'kind> {

    pub fn chop(&'kind mut self, offset: Vec2) -> &'kind mut Self
    {
        self.segments.push(MemRibbonSegment::Chop(offset));
        self
    }

    pub fn skip(&'kind mut self, offset: usize) -> &'kind mut Self
    {
        self.segments.push(MemRibbonSegment::Skip(offset));
        self
    }

    pub fn span(&'kind mut self, mut fields: Vec<Field<'kind>> ) -> &'kind mut Self
    {
        self.segments.push(MemRibbonSegment::Span(fields.len()));
        self.kind.fields.append(&mut fields);
        self
    }

    pub fn size_of(&self,access: &str) -> Option<usize>
    {
        todo!()
    }

    pub fn align_of(&self,access: &str) -> Option<usize>
    {
        todo!()
    }

    pub fn address_of(&self, access: &str) -> Option<usize>
    {
        todo!()
    }

    pub fn bytes_at(&self, address: usize, size: usize) -> Option<Vec<u8>>
    {

        let address = address - self.base_adr;

        self.data.get(address..address+size)?.iter()
            .map(MemByte::byte)
            .collect()
            
    }


    pub fn write_at(&mut self, address: usize, value: PrimValue)
    {

        let (ribbon_skip, prim_skip) = if address < self.base_adr {
            (0, self.base_adr - address)
        } else {
            (address - self.base_adr, 0)
        };

        let address = address - self.base_adr;

        let bytes = match &value {
            PrimValue::Bool(x) => if *x {&[1]} else {&[0]},
            PrimValue::Char(x) => x.as_bytes(),
            PrimValue::U8  (x) => x.as_bytes(),
            PrimValue::U16 (x) => x.as_bytes(),
            PrimValue::U32 (x) => x.as_bytes(),
            PrimValue::U64 (x) => x.as_bytes(),
            PrimValue::I8  (x)  => x.as_bytes(),
            PrimValue::I16 (x) => x.as_bytes(),
            PrimValue::I32 (x) => x.as_bytes(),
            PrimValue::I64 (x) => x.as_bytes(),
            PrimValue::F32 (x) => x.as_bytes(),
            PrimValue::F64 (x) => x.as_bytes(),
            PrimValue::Size(x) => x.as_bytes(),
        };

        self.data.iter_mut()
            .skip(ribbon_skip)
            .map(MemByte::byte_mut)
            .zip(bytes.iter().skip(prim_skip))
            .for_each(|(dst,src)|*dst = *src )

    }


    pub fn memcpy (&self, src: usize, dst: usize, size: usize)
    {
        todo!()
    }


    pub fn draw() -> Group
    {
        todo!()
    }


}






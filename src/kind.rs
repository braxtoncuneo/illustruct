
use std::{fmt::format, cell::Ref};

use pod::Pod;

use crate::{
    block_draw::{
        Vec2,
        BlockDrawSpec,
        block_plan::BlockDiagPlan,
    },
    access::{
        AccessUnit,
        AccessTrace,
        Access, PlaceValue, base_access_error_string, unwind_access_error_string,
    }, mem_ribbon::MemRibbon
};


pub struct Field<'kind>
{
    pub(crate) name : Option<String>,
    pub(crate) kind : &'kind Kind<'kind>,
}

impl<'a> Field<'a>{
  
  

    pub(crate) fn make_plan (&self, spec: &'a BlockDrawSpec, mins: Vec2, width: Option<f32>, notch: bool) -> BlockDiagPlan<'a> {
        let width = width.unwrap_or(spec.field_width(self,notch));
        let mut plan = spec.make_plan(self.kind, mins, Some(width), notch);

        if let Some(label) = self.name.as_deref() {
            let member_width = spec.member_width(self.kind);
            let left_width = if member_width > 0.0 {
                member_width + spec.prong_xpad
            } else {
                0.0
            };
            let label_pos_x : f32 = width - left_width - spec.label_width(label) - spec.label_pads.x;
            // spec.name_width(self.kind) + spec.label_pads.x + spec.text_pads.x * 2f32;
            let label_pos_y : f32 = spec.label_pads.y;
            let label_svg = spec.draw_label(label)
                .set("transform",format!("translate({label_pos_x},{label_pos_y})"));
            plan.head = plan.head.add(label_svg);
            plan
        } else {
            plan
        }
    }


}

#[derive(Clone,Copy)]
pub enum CompositeMode
{
    Product,
    Sum,
}



pub enum PrimValue {
    Bool(bool),
    Char(u8),
    U8(u8), U16(u16), U32(u32), U64(u64),
    I8(i8), I16(i16), I32(i32), I64(i64),
            F32(f32), F64(f64),
    Size(u32),
}





pub enum Primitive {
    Bool,
    Char,
    U8, U16, U32, U64,
    I8, I16, I32, I64,
             F32, F64,
    Size,
}



impl Primitive
{

    fn parse_at (&self, ribbon: &MemRibbon, address: usize) -> Option<PrimValue>
    {
        use Primitive::*;
        
        let (ribbon_skip, prim_skip) = if address < ribbon.base_adr {
            (0, ribbon.base_adr - address)
        } else {
            (address - ribbon.base_adr, 0)
        };

        let address = address - ribbon.base_adr;

        let mut value = match self {
            Bool => PrimValue::Bool(false),
            Char => PrimValue::Char(0),
            U8   => PrimValue::U8  (0),
            U16  => PrimValue::U16 (0),
            U32  => PrimValue::U32 (0),
            U64  => PrimValue::U64 (0),
            I8   => PrimValue::I8  (0),
            I16  => PrimValue::I16 (0),
            I32  => PrimValue::I32 (0),
            I64  => PrimValue::I64 (0),
            F32  => PrimValue::F32 (0.0),
            F64  => PrimValue::F64 (0.0),
            Size => PrimValue::Size(0),
        };

        let mut bool_u8: u8 = 0;

        let dst_slice = match &mut value {
            PrimValue::Bool(x) => bool_u8.as_bytes_mut(),
            PrimValue::Char(x)   => x.as_bytes_mut(),
            PrimValue::U8  (x)   => x.as_bytes_mut(),
            PrimValue::U16 (x)  => x.as_bytes_mut(),
            PrimValue::U32 (x)  => x.as_bytes_mut(),
            PrimValue::U64 (x)  => x.as_bytes_mut(),
            PrimValue::I8  (x)   => x.as_bytes_mut(),
            PrimValue::I16 (x)  => x.as_bytes_mut(),
            PrimValue::I32 (x)  => x.as_bytes_mut(),
            PrimValue::I64 (x)  => x.as_bytes_mut(),
            PrimValue::F32 (x)  => x.as_bytes_mut(),
            PrimValue::F64 (x)  => x.as_bytes_mut(),
            PrimValue::Size(x)  => x.as_bytes_mut(),
        };

        let bytes = ribbon.bytes_at(address, dst_slice.len())?;

        dst_slice.iter_mut()
            .zip(bytes.iter())
            .for_each(|(dst,src)|*dst = *src );

        match &mut value {
            PrimValue::Bool(v) => *v = (bool_u8 != 0),
            _ => {},
        };

        Some(value)

    }


}




impl Primitive {

    fn name_str(&self) -> &'static str
    {
        use Primitive::*;
        match self {
            Bool =>    "bool",
            Char =>    "char",
            U8   => "uint8_t", U16 => "uint16_t", U32 => "uint32_t", U64 => "uint64_t",
            I8   =>  "int8_t", I16 =>  "int16_t", I32 =>  "int32_t", I64 =>  "int64_t",
            F32  =>   "float", F64 =>   "double",
            Size =>  "size_t",
        }
    }

}


impl Primitive {

    fn name_string(&self) -> String {
        self.name_str().to_string()
    }

    fn size_of(&self) -> u16 {
        use Primitive::*;
        match self {
            Bool => 1,
            Char => 1,
            U8  =>  1, U16 => 2, U32 => 4, U64 => 8,
            I8  =>  1, I16 => 2, I32 => 4, I64 => 8,
            F32 =>  4, F64 => 8,
            Size => 4,
        }
    }

    fn align_of(&self) -> u16 {
        self.size_of()
    }

    fn variant_str(&self) -> &'static str {
        self.name_str()
    }


    fn access <'kind> (&'kind self, unit: &AccessUnit, trace: &'kind mut AccessTrace<'kind>)
        -> Result<PlaceValue<'kind>,String>
    {
        let type_name = self.name_string();
        let field_name = trace.field_name.clone();
        Err(base_access_error_string(unit,type_name,field_name))
    }

}

pub enum ReferenceMode
{
    Ref,
    Ptr,
}


pub struct Reference <'kind>
{
    mode: ReferenceMode,
    kind: &'kind Kind <'kind>,
}




impl <'kind> Reference <'kind>
{

    fn name_string(&self) -> String
    {
        let mut base_name = self.kind.name_string();
        match self.mode {
            ReferenceMode::Ptr => base_name.push_str("*"),
            ReferenceMode::Ref => base_name.push_str("&"),
        }
        base_name
    }

    fn align_of(&self) -> u16
    {
        Primitive::Size.size_of()
    }

    fn size_of(&self) -> u16
    {
        Primitive::Size.size_of()
    }

    fn variant_str(&self) -> &'static str {
        match self.mode {
            ReferenceMode::Ptr => "pointer",
            ReferenceMode::Ref => "reference"
        }
    }


    fn access_ref (&'kind self, unit: &AccessUnit, trace: &'kind mut AccessTrace<'kind>)
        -> Result<PlaceValue<'kind>,String>
    {

        use AccessUnit::*;

        match unit {
            Field(_) => {},
            unit => return Err(base_access_error_string(
                unit,
                self.name_string(),
                trace.field_name.clone()
            )),
        };

        let old_adr = trace.address;
        let new_adr = Primitive::Size.parse_at(trace.ribbon, trace.address);

        let field_name = trace.field_name.clone();

        trace.address = match new_adr {
            Some(PrimValue::Size(adr)) => adr as usize,
            None => return Err(format!(
                    "Bad deref of address {old_adr} \
                    for reference field {field_name}"
                )),
            _ => unreachable!(),
        };

        match self.kind {
            Kind::Composite(comp) => comp.access(unit,trace),
            _ => return Err(base_access_error_string(
                unit,
                self.name_string(),
                trace.field_name.clone()
            )),
        }

    }

    fn access_ptr (&'kind self, unit: &AccessUnit, trace: &'kind mut AccessTrace<'kind>)
        -> Result<PlaceValue<'kind>,String>
    {

        use AccessUnit::*;

        let old_adr = trace.address;
        let ptr_val = Primitive::Size.parse_at(trace.ribbon, old_adr);

        let field_name = trace.field_name.clone();
        let  type_name = self.kind.name_string();

        trace.address = match ptr_val {
            Some(PrimValue::Size(adr)) => adr as usize,
            None => return Err(format!(
                "Attempted bad deref of address {old_adr} \
                for pointer field {field_name}"
            )),
            _ => unreachable!(),
        };

        let field = match unit {
            Deref => return self.kind.access(trace),
            Index(idx) => {
                trace.address += self.kind.size_of() as usize * idx;
                return self.kind.access(trace)
            },
            Arrow(field) => field.clone(),
            unit => return Err(base_access_error_string(
                unit,
                self.name_string(),
                trace.field_name.clone()
            )),
        };

        match self.kind {
            Kind::Composite(comp) => comp.access(&Field(field),trace),
            _ => return Err(format!(
                "Arrow operator undefined for pointer {field_name} to \
                non-composite type {type_name}"
            ))
        }

        
    }

    fn access (&'kind self, unit: &AccessUnit, trace: &'kind mut AccessTrace<'kind>)
        -> Result<PlaceValue<'kind>,String>
    {

        match self.mode {
            ReferenceMode::Ref => self.access_ref(unit,trace),
            ReferenceMode::Ptr => self.access_ptr(unit,trace),
        }

    }

} 




pub struct Composite <'kind>
{
    pub name:   String,
    pub mode:   CompositeMode,
    pub fields: Vec<Field<'kind>>,
}



impl <'kind> Composite <'kind>
{

    fn name_string(&self) -> String
    {
        self.name.clone()
    }

    fn align_of(&self) -> u16
    {
        self.fields.iter()
            .map(|x| x.kind.size_of())
            .max()
            .unwrap_or(0u16)
    }


    fn size_of(&self) -> u16
    {
        let base_size = match self.mode {
            CompositeMode::Product => self.fields.iter()
                    .fold(0u16,|acc,x|
                        acc + x.kind.size_of() + x.kind.align_pad(acc)
                    ),
            CompositeMode::Sum     => self.fields.iter()
                .map(|x|x.kind.size_of())
                .max()
                .unwrap_or(0u16)
        };
        let align = self.align_of();
        let remainder = base_size % align;
        let padding   = if remainder > 0 {align-remainder} else {0};
        base_size + padding
    }

    fn variant_str(&self) -> &'static str {
        match self.mode {
            CompositeMode::Product => "struct",
            CompositeMode::Sum     => "union",
        }
    }


    fn offset_of(&self,field_name: &str) -> Option<u16>
    {

        match self.mode {
            CompositeMode::Sum => return Some(0u16),
            _ => {},
        }

        let mut result = 0u16;

        for field in self.fields.iter() {

            let name = match field.name.as_ref() {
                Some(name) => name,
                None => continue,
            };

            if field_name == name {
                return Some(result)
            }

            result += field.kind.align_pad(result);
            result += field.kind.size_of();

        }

        None

    }


    fn type_of(&'kind self,field_name: &str) -> Option<&'kind Kind<'kind>>
    {


        for field in self.fields.iter() {

            let name = match field.name.as_ref() {
                Some(name) => name,
                None => continue,
            };

            if field_name == name {
                return Some(field.kind)
            }

        }

        None

    }


    fn access (&'kind self, unit: &AccessUnit, trace: &'kind mut AccessTrace<'kind>)
        -> Result<PlaceValue<'kind>,String>
    {

        let subfield = match unit {
            AccessUnit::Field(field) => field,
            _ => return Err(base_access_error_string(
                unit,
                self.name_string(),
                trace.field_name.clone()
            ))
        };

        let field_name = trace.field_name.clone();

        match self.offset_of(subfield) {
            Some(offset) => {
                trace.address += offset as usize;
                let field_kind = self.type_of(subfield).unwrap();
                field_kind.access(trace)
            },
            None => {Err(format!(
                "Attempted to access non-existant field '{subfield}' \
                in composite type {field_name}"
            ))}
        }


    }

}






pub enum Kind <'kind>
{
    Primitive(Primitive),
    Reference(Reference<'kind>),
    Composite(Composite<'kind>),
} 


impl <'kind> Kind <'kind> {

    pub fn prim( value: Primitive ) -> Self
    {
        Kind::Primitive(value)
    }

    pub fn refr( value: Reference<'kind> ) -> Self
    {
        Kind::Reference(value)
    }

    pub fn comp( value: Composite<'kind> ) -> Self
    {
        Kind::Composite(value)
    }

    pub fn variant_str(&self) -> &'static str
    {
        match self {
            Kind::Primitive(x) => x.name_str(),
            Kind::Reference(x) => x.variant_str(),
            Kind::Composite(x) => x.variant_str(),
        }
    }
    
    pub fn name_string(&self) -> String
    {
        match self {
            Kind::Primitive(x) => x.name_string(),
            Kind::Reference(x) => x.name_string(),
            Kind::Composite(x) => x.name_string(),
        }
    }

    pub fn size_of(&self) -> u16
    {
        match self {
            Kind::Primitive(x) => x.size_of(),
            Kind::Reference(x) => x.size_of(),
            Kind::Composite(x) => x.size_of(),
        }
    }

    pub fn align_of(&self) -> u16
    {
        match self {
            Kind::Primitive(x) => x.align_of(),
            Kind::Reference(x) => x.align_of(),
            Kind::Composite(x) => x.align_of(),
        }
    }

    pub fn align_pad(&self,offset:u16) -> u16 {
        let align = self.align_of();
        let remainder = offset % align;
        if remainder !=0 {
            align - remainder
        } else {
            0
        }
    }


    fn empty_access (&'kind self, trace: &'kind mut AccessTrace<'kind>)
        -> Result<PlaceValue<'kind>,String>
    {

        use PrimValue::*;

        let refr = match self {
            Kind::Reference(refr) => refr,
            _ => return Ok(PlaceValue{ kind: self, address: trace.address })
        };

        let field_name = trace.field_name.clone();

        match Primitive::Size.parse_at(trace.ribbon, trace.address) {
            Some(Size(address)) => Ok(PlaceValue{
                    kind: refr.kind,
                    address: address as usize
                }),
            None => Err(format!("Bad deref of reference field {field_name}")),
            _ => unreachable!(),
        }

        

    }


    pub fn access (&'kind self, trace: &'kind mut AccessTrace<'kind>) -> Result<PlaceValue<'kind>,String> {


        let unit = match trace.iter.next() {
            Some(unit) => unit,
            None => return self.empty_access(trace),
        };

        let field_name = trace.field_name.clone();

        let result = match self {
            Kind::Primitive(x) => x.access(unit,trace),
            Kind::Reference(x) => x.access(unit,trace),
            Kind::Composite(x) => x.access(unit,trace),
        };

        let mut err_str = match result {
            Ok(ok) => return Ok(ok),
            Err(err_str) => err_str,
        };

        let var_str = self.variant_str();
        err_str.push_str(format!(" in {var_str} {field_name} ").as_str());

        Err(err_str)

    }


}



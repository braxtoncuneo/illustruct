use svg::node::element::Group;

use crate::block_draw::Vec2;

use super::block_draw::{ BlockDrawSpec, BlockDiagPlan };

pub(crate) struct Field<'a>
{
    pub(crate) name : Option<String>,
    pub(crate) kind : &'a Kind<'a>,
}

impl<'a> Field<'a>{
  
  

    pub(crate) fn make_plan (&self, spec: &'a BlockDrawSpec, mins: Vec2, width: Option<f32>, notch: bool) -> BlockDiagPlan<'a> {
        let width = width.unwrap_or(spec.field_width(self,notch));
        let mut plan = spec.make_plan(self.kind, mins, Some(width), notch);

        if let Some(label) = self.name.as_deref() {
            let label_pos_x : f32 = spec.name_width(self.kind, notch) + spec.label_pads.x + spec.text_pads.x * 2f32;
            let label_pos_y : f32 = spec.label_pads.y;
            let label_svg = spec.draw_label(Some(label))
                .set("transform",format!("translate({label_pos_x},{label_pos_y})"));
            plan.head = plan.head.add(label_svg);
            plan
        } else {
            plan
        }
    }


}

#[derive(Clone,Copy)]
pub(crate) enum CompositeMode
{
    Product,
    Sum,
}

#[derive(Clone,Copy)]
pub(crate) enum ReferenceMode
{
    Ref,
    Ptr,
}

pub(crate) enum Prim {
    Bool,
    Char,
    U8, U16, U32, U64,
    I8, I16, I32, I64,
             F32, F64,
    Size,
}

impl Prim {
    pub(crate) fn name_str(&self) -> &str {
        use Prim::*;
        match self {
            Bool =>    "bool",
            Char =>    "char",
            U8   => "uint8_t", U16 => "uint16_t", U32 => "uint32_t", U64 => "uint64_t",
            I8   =>  "int8_t", I16 =>  "int16_t", I32 =>  "int32_t", I64 =>  "int64_t",
            F32  =>   "float", F64 =>   "double",
            Size =>  "size_t",
        }
    }
    pub(crate) fn size_of(&self) -> u16 {
        use Prim::*;
        match self {
            Bool => 1,
            Char => 1,
            U8  =>  1, U16 => 2, U32 => 4, U64 => 8,
            I8  =>  1, I16 => 2, I32 => 4, I64 => 8,
            F32 =>  4, F64 => 8,
            Size => 4,
        }
    }
}

pub(crate) enum Kind<'a> {
    Primitive (Prim),
    Composite {
        name:   Option<String>,
        mode:   CompositeMode,
        fields: Vec<Field<'a>>,
    },
}

impl<'a> Kind<'a> {

    pub(crate) fn prim (kind: Prim) -> Self
    {
        Kind::Primitive(kind)
    }

    pub(crate) fn comp(name: Option<String>, mode: CompositeMode, fields: Vec<Field<'a>>) -> Self
    {
        Kind::Composite { name, mode, fields }
    }

    pub(crate) fn name_string(&self) -> Option<String>
    {
        match self {
            Kind::Primitive(name_kind) => Some(name_kind.name_str().to_string()),
            Kind::Composite{name,mode: _,fields: _} => name.clone(),
        }
    }

    pub(crate) fn align_of(&self) -> u16
    {
        match self {
            Kind::Primitive(name_kind) => name_kind.size_of(),
            Kind::Composite{name:_,mode:_,fields} => fields.iter()
                .map(|x| x.kind.size_of())
                .max()
                .unwrap_or(0u16),
        }
    }


    pub(crate) fn size_of(&self) -> u16
    {
        match self {
            Kind::Primitive(name_kind) => name_kind.size_of(),
            Kind::Composite{name:_,mode,fields} => {
                let base_size = match mode {
                    CompositeMode::Product => fields.iter()
                            .fold(0u16,|acc,x|
                                acc + x.kind.size_of() + x.kind.align_pad(acc)
                            ),
                    CompositeMode::Sum     => fields.iter()
                        .map(|x|x.kind.size_of())
                        .max()
                        .unwrap_or(0u16)
                };
                let align = self.align_of();
                let remainder = base_size % align;
                let padding   = if remainder > 0 {align-remainder} else {0};
                base_size + padding
            }
        }
    }

    pub(crate) fn align_pad(&self,offset:u16) -> u16 {
        let align = self.align_of();
        let remainder = offset % align;
        if remainder !=0 {
            align - remainder
        } else {
            0
        }
    }

    pub(crate) fn defn_str(&self) -> String
    {

        match self {
            Kind::Primitive (prim_kind) => prim_kind.name_str().to_string()+ ";",
            Kind::Composite { name, mode, fields } => {

                let mode_str = match mode {
                    CompositeMode::Product => "struct",
                    CompositeMode::Sum     => "union",
                };

                let members = fields.iter()
                    .map(|x| x.name
                            .as_ref()
                            .map(|x| x.to_string())
                            .unwrap_or(String::new())
                    )
                    .fold(String::new(),|x,y| x + &y + ";\n");

                mode_str.to_string()
                    + " "
                    + name.as_ref().map(|x| x as &str).unwrap_or("")
                    + "\n{\n"
                    + &members
                    + "};"

            }
        }

    }


}

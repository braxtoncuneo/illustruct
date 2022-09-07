
use svg::node::element::{
    Text, Path, Group,
    path::Data
};
use unicode_width::UnicodeWidthStr;

use crate::kind::{
    Kind,
    CompositeMode,
    Field,
};



use std::{
    iter::Iterator,
    ops::Add
};

use self::block_plan::BlockDiagPlan;






pub mod block_plan;




#[derive(Clone,Copy)]
struct RGB (u8,u8,u8);


#[derive(Clone,Copy)]
pub struct Vec2 {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl Vec2 {

    pub(crate) fn new(x: f32, y: f32) -> Self
    {
        Self{x,y}
    }

}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Self{ x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl Default for Vec2 {
    fn default() -> Self {
        Self { x: 0f32, y: 0f32 }
    }
}

pub struct BlockDrawSpec
{
    pub(crate) char_dims   : Vec2,
    pub(crate) text_pads   : Vec2,
    pub(crate) label_pads  : Vec2,
    pub(crate) union_xpad  :  f32,
    pub(crate) fill_inset  :  f32,
    pub(crate) prong_width :  f32,
    pub(crate) prong_xpad  :  f32,
    pub(crate) chamfer_size:  f32,
}





impl BlockDrawSpec
{

    fn label_height (&self) -> f32
    {
        self.char_dims.y + 2.0f32 * self.text_pads.y
    }

    pub(crate) fn line_height (&self) -> f32
    {
        self.label_height() + 2.0f32 * self.label_pads.y
    }

    fn bare_width(&self,text: &str) -> f32
    {
        (text.width_cjk() as f32) * self.char_dims.x
    }

    pub(crate) fn tpad_width(&self,text: &str) -> f32
    {
        self.bare_width(text) + 2.0f32 * self.text_pads.x
    }

    pub(crate) fn label_width(&self,text: &str) -> f32
    {
        self.tpad_width(text) + 2.0f32 * self.label_pads.x
    }


    pub(crate) fn draw_header(&self, text: &str, width: f32, notch: bool) -> Group {

        let half_height = self.line_height()/2f32;

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

        Group::new().add(path).add(Text::new()
        .add(svg::node::Text::new(text))
        .set("fill", "white")
        .set("font-family", "monospace")
        .set("font-size", self.char_dims.y)
        .set("dominant-baseline", "middle")
        .set("text-anchor", "middle")
        .set("x", self.tpad_width(text)/2f32)
        .set("y", half_height))

    }

    pub(crate) fn draw_block <'kind> (&self,kind: &'kind Kind, width: f32, notch: bool) -> Option<Group>
    {
        let size = kind.size_of();

        let head_height = self.line_height();
        let half_head = head_height / 2f32;
        let height = self.line_height() * ( size as f32 );
        let chamfer = if size == 1 { 0f32 } else { self.chamfer_size };
        let rem_height  = height - head_height - chamfer;

        let width_chopped = width - chamfer;

        let notch_x = notch.then_some(self.prong_width).unwrap_or_default();

        if size > 1 {

            let half_inset = self.fill_inset/2f32;

            let data = Data::new()
                .move_to((                 0,                       0))
                .line_by((                 0,  height+self.fill_inset))
                .line_by((     width_chopped,                       0))
                .line_by((           chamfer,     -chamfer-half_inset))
                .line_by((                 0,  -rem_height-half_inset))
                .line_by((          -notch_x,              -half_head))
                .line_by((           notch_x,              -half_head))
                .close();

            let path = Path::new()
                .set("fill", "black")
                .set("stroke", "none")
                .set("d", data);

            let fill_data = Data::new()
                .move_to((   self.fill_inset,                self.fill_inset))
                .line_by((                 0,         height-self.fill_inset))
                .line_by((width_chopped-half_inset*3f32,                   0))
                .line_by((      chamfer-half_inset,                 -chamfer))
                .line_by((                 0, -rem_height+half_inset))
                .line_by((          -notch_x, -half_head -half_inset))
                .line_by((           notch_x, -half_head+half_inset))
                .close();

            let fill_path = Path::new()
                //.set("fill", "grey")
                .set("stroke", "none")
                .set("d", fill_data);

            let g = Group::new()
                .add(path)
                .add(fill_path);

            Some(g)

        } else {
            None
        }


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

    pub(crate) fn draw_label(&self, text: &str) -> Group
    {

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


        let group = Group::new().add(path);

        group.add( Text::new()
            .add(svg::node::Text::new(text))
            .set("fill", "black")
            .set("font-family", "monospace")
            .set("font-size", self.char_dims.y)
            .set("dominant-baseline", "middle")
            .set("text-anchor", "middle")
            .set("x",  width/2f32)
            .set("y", height/2f32)
        )

    }



    pub(crate) fn name_width <'kind> (&self, kind: &'kind Kind<'kind> ) -> f32
    {
        self.tpad_width(kind.name_string().as_str())
    }

    pub(crate) fn member_width <'kind> (&self,kind: &'kind Kind<'kind>) -> f32
    {
        match kind {
            Kind::Composite(comp) => match comp.mode {
                CompositeMode::Product => {
                    comp.fields.iter().enumerate()
                        .map(|(i,f)| self.field_width(f,i==0))
                        .fold(0f32,|x,y| y.max(x) )
                        + self.prong_xpad
                },
                CompositeMode::Sum => {
                    comp.fields.iter()
                        .map(|x| self.field_width(x,false))
                        .fold(0f32,|a,x| a+x + self.union_xpad + self.prong_width + self.prong_xpad)

                }
            }
            _ => 0f32,
        }
    }

    pub(crate) fn unlabeled_width <'kind> (&self, kind: &'kind Kind<'kind>, notch: bool) -> f32
    {

        let prong_width = match kind {
            Kind::Composite(_) => self.prong_width,
            _ => 0f32,
        };
        
        self.name_width(kind)
        + self.member_width(kind)
        + prong_width
    }

    pub(crate) fn height <'kind> (&self,kind: &'kind Kind<'kind>) -> f32
    {
        (kind.size_of() as f32) * self.line_height()
    }


    pub(crate) fn field_height(&self, field: &Field) -> f32 {
        self.height(field.kind)
    }

    pub(crate) fn field_width(&self, field: &Field, notch: bool) -> f32 {
        self.unlabeled_width(field.kind,notch)
        + self.label_width(field.name.as_deref().unwrap_or_default())
        + self.label_pads.x * 2f32
    }




    pub(crate) fn plan_primitive <'kind> (&'kind self, kind:&'kind Kind<'kind>, mins:Vec2, width:Option<f32>, notch:bool) -> block_plan::BlockDiagPlan<'kind> {
        let dims = Vec2{
                x: width.unwrap_or(self.unlabeled_width(kind,notch)),
                y: self.height(kind)
            };
        block_plan::BlockDiagPlan {
            spec: &self,
            head: self.draw_header(
                kind.name_string().as_str(), 
                width.unwrap_or(self.unlabeled_width(kind,notch)),
                notch,
            ),
            body_plan: None,
            mins: mins,
            maxs: mins+dims,
            kind: kind,
            sub_blocks: Vec::new(),
            graph_index: None,
        }
    }

    pub(crate) fn plan_product_fields <'kind> ( &'kind self, fields: &Vec<Field<'kind>>, mins: Vec2, width: f32 ) -> Vec<block_plan::BlockDiagPlan<'kind>> {
       
        let mut field_plans: Vec<block_plan::BlockDiagPlan> = Vec::new();
        let mut offset       = 0u16;
        let mut deltas = Vec2::new(0f32,0f32);

        for f in fields {

            let pad = f.kind.align_pad(offset);
            let pad_height = (pad as f32) * self.line_height();
            let size = f.kind.size_of();
            let size_height = (size as f32) * self.line_height();

            deltas.y += pad_height;

            let notch = offset == 0;

            let mut f_plan = f.make_plan(
                self,
                mins+deltas,
                Some(width),
                notch
            );

            if let Some(body_plan) = &mut f_plan.body_plan{

                _ = body_plan.relative_pos.insert(deltas);
                //g.clone().set("transform",format!("translate({x},{y})"));

            };

            field_plans.push(f_plan);
            deltas.y += size_height;
            offset += size + pad;

        }
        field_plans

    }


    pub(crate) fn plan_sum_fields <'a> ( &'a self, fields: &Vec<Field<'a>>, mins: Vec2) -> Vec<block_plan::BlockDiagPlan<'a>> {
       
        let mut field_plans: Vec<block_plan::BlockDiagPlan> = Vec::new();
        let mut deltas = Vec2::new(0f32,0f32);

        for f in fields {

            let w = self.field_width(&f,false)
                    + self.union_xpad
                    + self.prong_width
                    + self.prong_xpad;


            let notch = false;

            let mut f_plan = f.make_plan(
                self,
                mins+deltas,
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



    pub(crate) fn make_plan <'kind> (&'kind self, kind: &'kind Kind<'kind>, mins: Vec2, width: Option<f32>, notch: bool) -> block_plan::BlockDiagPlan<'kind>
    {


        let block_width   = width.unwrap_or(self.unlabeled_width(kind,notch));
        let block_height  = self.height(kind);

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
                CompositeMode::Sum     => self.plan_sum_fields(
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
        let prong_padding   = if gapped {self.prong_xpad} else {0f32};
        let header_offset = member_width + prong_padding;
        let head = self.draw_header(
                kind.name_string().as_str(), 
                block_width-member_width-prong_padding,
                notch
            )
            .set("transform",format!("translate({header_offset},0)"));
        
        block_plan::BlockDiagPlan {
            spec: self,
            head,
            body_plan: Some(body_plan),
            mins,
            maxs,
            kind: kind,
            sub_blocks: fields,
            graph_index: None
        }


    }



}











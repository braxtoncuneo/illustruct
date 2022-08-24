
use svg::{node::element::{
    Text, Path, Group,
    path::Data
}, Node};
use unicode_width::UnicodeWidthStr;

use crate::kind::{
    Kind,
    Prim,
    CompositeMode,
    Field,
};


use petgraph::{
    Undirected,
    graph::NodeIndex,  stable_graph::StableGraph, data::Build
};

use std::{
    iter::{
        Peekable,
        Iterator,
    },
    collections::HashMap, ops::Add, fmt::format
};



#[derive(Clone,Copy)]
struct RGB (u8,u8,u8);


#[derive(Clone,Copy)]
pub(crate) struct Vec2 {
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




trait BlockAdjSpanIter = Iterator<Item=BlockAdjSpan>;

#[derive(Clone)]
struct BlockAdjSpan {
    min: f32,
    max: f32,
    index: NodeIndex,
}

struct BlockAdjListPairIter <A: BlockAdjSpanIter,B: BlockAdjSpanIter>{
    a: Peekable<A>,
    b: Peekable<B>,
}

impl <A,B> BlockAdjListPairIter <A,B>
where   A : BlockAdjSpanIter,
        B : BlockAdjSpanIter
{
    fn new ( a : Peekable<A>, b : Peekable<B> ) -> Self {
        Self { a, b }
    }
}

impl <A,B> Iterator for BlockAdjListPairIter <A,B>
where   A : BlockAdjSpanIter,
        B : BlockAdjSpanIter
{
    type Item = (NodeIndex,NodeIndex);

    fn next(&mut self) -> Option<Self::Item> {

        match (self.a.peek().cloned(),self.b.peek().cloned()) {
            (Some(x),Some(y)) => loop{
                if x.max < y.min {
                    println!("xmax{} < ymin{}",x.max,y.min);
                    self.a.next();
                    continue;
                } else if y.max < x.min {
                    println!("ymax{} < xmin{}",x.max,y.min);
                    self.b.next();
                    continue;
                } else if x.max < y.max {
                    println!("xmax{} < ymin{}",x.max,y.min);
                    self.a.next();
                } else {
                    println!("xmax{} >= ymin{}",x.max,y.min);
                    self.b.next();
                }
                break Some((x.index,y.index))
            },
            (x,y) => {
                println!("x is_some = {} y is_some = {}",x.is_some(),y.is_some());
                None
            },
        }
    }
}






struct BlockBodyPlan
{
    block_width  : f32,
    notch        : bool,
    relative_pos : Option<Vec2>,
}


pub(crate) struct BlockDiagPlan<'a>
{
    pub(crate) spec : &'a BlockDrawSpec,
    pub(crate) head : Group,
    body_plan : Option<BlockBodyPlan>,
    pub(crate) mins : Vec2,
    pub(crate) maxs : Vec2,
    pub(crate) kind : &'a Kind<'a>,
    pub(crate) sub_blocks : Vec<BlockDiagPlan<'a>>,
    pub(crate) graph_index : Option<NodeIndex>,
}

impl <'plan> BlockDiagPlan <'plan>
{

    fn setup_nodes (
            &mut self,
            graph: &mut StableGraph<(),(),Undirected>,
            parent: Option<NodeIndex>
        )
    {
        if self.body_plan.is_some() {
            let index = *self.graph_index.get_or_insert_with(||graph.add_node(()));
            let name_str = self.kind.name_string().unwrap_or_default();
            let uidx = index.index();
            println!("{uidx} @ {name_str}");
            if let Some(p_idx) = parent {
                graph.add_edge(p_idx, index, ());
            };
            for block in self.sub_blocks.iter_mut() {
                block.setup_nodes(graph, Some(index))
            }
        }
    }

    fn block_product_stack (
            &self,
            graph: &mut StableGraph<(),(),Undirected>,
            left_side: Option<NodeIndex>
        ) -> (Vec<BlockAdjSpan>,Vec<BlockAdjSpan>)
    {
        let index = self.graph_index.unwrap();
        let mut interface: Option<(Vec<BlockAdjSpan>,Vec<BlockAdjSpan>)> = None;
        for block in self.sub_blocks.iter()
        {
            let sub_pair = 
                block.block_graph_recurse(
                    graph,
                    Some(index),
                    left_side,
                );
            interface = if let Some((top,bot)) = interface {
                let (sub_top,sub_bot) = sub_pair;
                for (a,b) in BlockAdjListPairIter::new(
                    bot.into_iter().peekable(),
                    sub_top.into_iter().peekable(),
                ) {
                    let ai = a.index();
                    let bi = b.index();
                    println!("P {ai} <- {bi}");
                    graph.add_edge(a, b,());
                };
                Some((top,sub_bot))
            } else {
                Some(sub_pair)
            }
        };
        let (mut top, mut bot) = interface.unwrap_or_default();
        if let Some(last) = top.last() {
                top.push(BlockAdjSpan { index, min: last.max, max: self.maxs.x});
        } else {
                top.push(BlockAdjSpan { index, min: self.mins.x, max: self.maxs.x});
        }
        if let Some(last) = bot.last() {
                bot.push(BlockAdjSpan { index, min: last.max, max: self.maxs.x});
        } else {
                bot.push(BlockAdjSpan { index, min: self.mins.x, max: self.maxs.x});
        }
        (top,bot) 
    }


    fn block_sum_row (
            &self,
            graph: &mut StableGraph<(),(),Undirected>,
            left_side: Option<NodeIndex>,
        ) -> (Vec<BlockAdjSpan>,Vec<BlockAdjSpan>)
    {
        let index = self.graph_index.unwrap();
        let mut top = Vec::<BlockAdjSpan>::new();
        let mut bot = Vec::<BlockAdjSpan>::new();
        for (i,block) in self.sub_blocks.iter().enumerate()
        {
            println!("{i}");
            let lefty = if i == 0 {
                left_side
            } else {
                Some(index)
            };

            let (mut sub_top,mut sub_bot) = 
                block.block_graph_recurse(
                    graph, 
                    Some(index),
                    lefty,
                );
            match ( top.last(), sub_top.first() ) {
                (Some(last),Some(first)) => {
                    top.push(BlockAdjSpan { index, min: last.max, max: first.min});
                },
                _ => {},
            };
            top.append(&mut sub_top);
            match ( bot.last(), sub_bot.first() ) {
                (Some(last),Some(first)) =>
                if block.maxs.y + 0.1 > self.maxs.y {
                    bot.push(BlockAdjSpan { index, min: last.max, max: first.min});
                },
                _ => {},
            };
            bot.append(&mut sub_bot);
        };
        if let Some(last) = top.last() {
                top.push(BlockAdjSpan { index, min: last.max, max: self.maxs.x});
        } else {
                top.push(BlockAdjSpan { index, min: self.mins.x, max: self.maxs.x});
        }
        if let Some(last) = bot.last() {
                bot.push(BlockAdjSpan { index, min: last.max, max: self.maxs.x});
        } else {
                bot.push(BlockAdjSpan { index, min: self.mins.x, max: self.maxs.x});
        }
        (top,bot)
    }


    fn block_graph_recurse (
            &self,
            graph: &mut StableGraph<(),(),Undirected>,
            parent: Option<NodeIndex>,
            left_side: Option<NodeIndex>,
        ) -> (Vec<BlockAdjSpan>,Vec<BlockAdjSpan>)
    {
        use super::kind::CompositeMode;


        match self.kind {
            Kind::Primitive(Prim::U8) => { (vec![],vec![]) },
            Kind::Primitive(_)        => {
                let index = self.graph_index.unwrap();
                let spans = vec![BlockAdjSpan{ 
                    min: self.mins.x,
                    max: self.maxs.x,
                    index
                }];
                println!("At {}",index.index());
                if let Some(lefty) = left_side {
                    graph.add_edge(index,lefty,());
                }
                (spans.clone(),spans)
            },
            Kind::Composite { name: _, mode, fields: _ } => match mode {
                CompositeMode::Product => self.block_product_stack(graph, left_side),
                CompositeMode::Sum     => self.block_sum_row(graph,left_side),
            },
        
        }
    } 


    fn into_svg_recurse (&mut self,color_map : &HashMap<NodeIndex,usize>) -> Group {
       
        let colors : [&str;8] = ["red","green","blue","magenta","cyan","yellow","grey","white" ];
        //let colors : [&str;8] = ["#EEEEEE","#CCCCC","#AAAAAA","#888888","cyan","yellow","grey","white" ];

        let label = self.kind.name_string().unwrap_or_default();
        println!("Drawing {label}");

        let base_group = if let Some(body_plan) = self.body_plan.as_ref() {

            let tone = colors[color_map[&self.graph_index.unwrap()]];

            self.spec.draw_block(
                self.kind,
                body_plan.block_width,
                body_plan.notch
            ).map(|g|{
                let delta = body_plan.relative_pos.unwrap_or_default();
                let x = delta.x;
                let y = delta.y;
                g.set("transform",format!("translate({x},{y})"))
                 .set("fill",tone)

            }).unwrap_or_default()
        } else {
            Group::default()
        };

        self.sub_blocks.iter_mut()
            .fold(base_group, |b,sub| b.add(sub.into_svg_recurse(color_map)))
            .add(self.head.clone())
        
    }

    pub(crate) fn into_svg (&mut self) -> Group {
        let mut color_graph = StableGraph::<(),(),Undirected>::default();
        println!("Setting up nodes!");
        self.setup_nodes(&mut color_graph, None);
        println!("Linking nodes!");
        self.block_graph_recurse(&mut color_graph, None, None);
        println!("Coloring nodes!");
        let color_map = crate::graph::block_graph_color(&color_graph);
        println!("Generating SVG!");
        self.into_svg_recurse(&color_map)
    }


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

    fn bare_width(&self,text: Option<&str>) -> f32
    {
        text.map(|x|x.width_cjk() as f32)
            .unwrap_or(0f32) * self.char_dims.x
    }

    pub(crate) fn tpad_width(&self,text: Option<&str>) -> f32
    {
        self.bare_width(text) + 2.0f32 * self.text_pads.x
    }

    pub(crate) fn label_width(&self,text: Option<&str>) -> f32
    {
        self.tpad_width(text) + 2.0f32 * self.label_pads.x
    }


    pub(crate) fn draw_header(&self, text: Option<&str>, width: f32, notch: bool) -> Group {

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

        if let Some(header_text) = text {
            Group::new().add(path).add(Text::new()
            .add(svg::node::Text::new(header_text))
            .set("fill", "white")
            .set("font-family", "monospace")
            .set("font-size", self.char_dims.y)
            .set("dominant-baseline", "middle")
            .set("text-anchor", "middle")
            .set("x", self.tpad_width(text)/2f32)
            .set("y", half_height))
        } else {
            Group::new().add(path)
        }

    }

    pub(crate) fn draw_block(&self,kind: &Kind, width: f32, notch: bool) -> Option<Group>
    {
        let size = kind.size_of();
        let label = kind.name_string();

        let head_height = self.line_height();
        let half_head = head_height / 2f32;
        let height = self.line_height() * ( size as f32 );
        let chamfer = if size == 1 { 0f32 } else { self.chamfer_size };
        let height_chopped = height - chamfer;
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

    pub(crate) fn draw_label(&self, text: Option<&str>) -> Group
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


        let mut group = Group::new().add(path);

        if let Some(label_text) = text {
            group.add( Text::new()
                .add(svg::node::Text::new(label_text))
                .set("fill", "black")
                .set("font-family", "monospace")
                .set("font-size", self.char_dims.y)
                .set("dominant-baseline", "middle")
                .set("text-anchor", "middle")
                .set("x",  width/2f32)
                .set("y", height/2f32)
            )
        } else {
            group
        }

    }



    pub(crate) fn name_width  (&self, kind: &Kind, notch: bool) -> f32
    {
        match kind {
            Kind::Primitive(prim_kind) => self.tpad_width(Some(&prim_kind.name_str())),
            Kind::Composite { name, ..} =>
                self.tpad_width(name.as_deref())
                //+ self.prong_xpad
        }
    }

    pub(crate) fn member_width  (&self,kind: &Kind) -> f32
    {
        match kind {
            Kind::Primitive (_) => 0f32,
            Kind::Composite {  mode: CompositeMode::Product, name, fields } => {
                fields.iter().enumerate()
                    .map(|(i,f)| self.field_width(f,(i==0)))
                    .fold(0f32,|x,y| y.max(x) )
                    + self.prong_xpad
            }
            Kind::Composite {  mode: CompositeMode::Sum, name, fields } => {
                fields.iter()
                    .map(|x| self.field_width(x,false))
                    .fold(0f32,|a,x| a+x + self.union_xpad + self.prong_width + self.prong_xpad)

            }
        }
    }

    pub(crate) fn unlabeled_width  (&self, kind: &Kind, notch: bool) -> f32
    {

        let prong_width = match kind {
            Kind::Primitive(_) => 0f32,
            Kind::Composite { .. } => self.prong_width
        };
        
        self.name_width(kind,notch)
        + self.member_width(kind)
        + prong_width
    }

    pub(crate) fn height (&self,kind: &Kind) -> f32
    {
        (kind.size_of() as f32) * self.line_height()
    }


    pub(crate) fn field_height(&self, field: &Field) -> f32 {
        self.height(field.kind)
    }

    pub(crate) fn field_width(&self, field: &Field, notch: bool) -> f32 {
        self.unlabeled_width(field.kind,notch)
        + self.label_width(field.name.as_deref())
        + self.label_pads.x * 2f32
    }




    pub(crate) fn plan_primitive <'a> (&'a self, kind:&'a Kind, mins:Vec2, width:Option<f32>, notch:bool) -> BlockDiagPlan<'a> {
        let dims = Vec2{
                x: width.unwrap_or(self.unlabeled_width(kind,notch)),
                y: self.height(kind)
            };
        BlockDiagPlan {
            spec: &self,
            head: self.draw_header(
                kind.name_string().as_deref(), 
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

    pub(crate) fn plan_product_fields <'a> ( &'a self, fields: &Vec<Field<'a>>, mins: Vec2, width: f32 ) -> Vec<BlockDiagPlan<'a>> {
       
        let mut field_plans: Vec<BlockDiagPlan> = Vec::new();
        let mut offset       = 0u16;
        let mut deltas = Vec2::new(0f32,0f32);

        for f in fields {

            let pad = f.kind.align_pad(offset);
            let pad_height = (pad as f32) * self.line_height();
            let size = f.kind.size_of();
            let size_height = (size as f32) * self.line_height();

            deltas.y += pad_height;

            let y = deltas.y;
            let x = deltas.x;

            let notch = (offset == 0);

            let mut f_plan = f.make_plan(
                self,
                mins+deltas,
                Some(width),
                notch
            );

            if let Some(body_plan) = &mut f_plan.body_plan{

                body_plan.relative_pos.insert(deltas);
                //g.clone().set("transform",format!("translate({x},{y})"));

            };

            field_plans.push(f_plan);
            deltas.y += size_height;
            offset += size + pad;

        }
        field_plans

    }


    pub(crate) fn plan_sum_fields <'a> ( &'a self, fields: &Vec<Field<'a>>, mins: Vec2, width: f32 ) -> Vec<BlockDiagPlan<'a>> {
       
        let mut field_plans: Vec<BlockDiagPlan> = Vec::new();
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
                body_plan.relative_pos.insert(deltas);
            };

            field_plans.push(f_plan);
            deltas.x += w;

        }
        field_plans

    }



    pub(crate) fn make_plan <'a> (&'a self, kind: &'a Kind, mins: Vec2, width: Option<f32>, notch: bool) -> BlockDiagPlan<'a>
    {


        let block_width   = width.unwrap_or(self.unlabeled_width(kind,notch));
        let block_height  = self.height(kind);

        let body_plan = BlockBodyPlan {
            block_width,
            notch,
            relative_pos: None,
        };


        let fields = match kind {
            Kind::Primitive(_) => Vec::new(),
            Kind::Composite { mode: CompositeMode::Product, name, fields } =>
                self.plan_product_fields(fields, mins, self.member_width(kind) ),
            Kind::Composite { mode: CompositeMode::Sum,     name, fields } =>
                self.plan_sum_fields    (fields, mins, self.member_width(kind)),
               
        };

        let gapped = match kind {
            Kind::Primitive ( _) => false,
            Kind::Composite {..} => true,
        };


        let maxs = mins + Vec2::new(block_width,block_height);
        
        let member_width = self.member_width(kind);
        let prong_padding   = if gapped {self.prong_xpad} else {0f32};
        let header_offset = member_width + prong_padding;
        let head = self.draw_header(
                kind.name_string().as_deref(), 
                block_width-member_width-prong_padding,
                notch
            )
            .set("transform",format!("translate({header_offset},0)"));
        
        BlockDiagPlan {
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

use std::collections::HashMap;
use std::iter::Peekable;

use crate::kind::CompositeMode;

use crate::kind::Primitive;




use petgraph::Undirected;

use petgraph::stable_graph::StableGraph;

use petgraph::graph::NodeIndex;

use crate::kind::Kind;

use svg::node::element::Group;

use super::BlockDrawSpec;


use super::Vec2;


pub(crate) trait BlockAdjSpanIter = Iterator<Item=BlockAdjSpan>;

#[derive(Clone)]
pub(crate) struct BlockAdjSpan {
    min: f32,
    max: f32,
    index: NodeIndex,
}

pub(crate) struct BlockAdjListPairIter <A: BlockAdjSpanIter,B: BlockAdjSpanIter>{
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



pub(crate) struct BlockBodyPlan
{
    pub(crate) block_width  : f32,
    pub(crate) notch        : bool,
    pub(crate) relative_pos : Option<Vec2>,
}

pub(crate) struct BlockDiagPlan<'kind>
{
    pub(crate) spec : &'kind BlockDrawSpec,
    pub(crate) head : Group,
    pub(crate) body_plan : Option<BlockBodyPlan>,
    pub(crate) mins : Vec2,
    pub(crate) maxs : Vec2,
    pub(crate) kind : &'kind Kind<'kind>,
    pub(crate) sub_blocks : Vec<BlockDiagPlan<'kind>>,
    pub(crate) graph_index : Option<NodeIndex>,
}

impl <'kind> BlockDiagPlan <'kind>
{

    pub(crate) fn setup_nodes (
            &mut self,
            graph: &mut StableGraph<(),(),Undirected>,
            parent: Option<NodeIndex>
        )
    {
        if self.body_plan.is_some() {
            let index = *self.graph_index.get_or_insert_with(||graph.add_node(()));
            let name_str = self.kind.name_string();
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


    pub(crate) fn cap_span (&self, spans: (Vec<BlockAdjSpan>,Vec<BlockAdjSpan>) )
        -> (Vec<BlockAdjSpan>,Vec<BlockAdjSpan>)
    {

        let index = self.graph_index.unwrap();

        let (mut top, mut bot) = spans;
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


    pub(crate) fn block_product_stack (
            &self,
            graph: &mut StableGraph<(),(),Undirected>,
            left_side: Option<NodeIndex>
        ) -> (Vec<BlockAdjSpan>,Vec<BlockAdjSpan>)
    {
        let mut interface: Option<(Vec<BlockAdjSpan>,Vec<BlockAdjSpan>)> = None;
        for block in self.sub_blocks.iter()
        {
            let sub_pair = 
                block.block_graph_recurse(
                    graph,
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
        self.cap_span(interface.unwrap_or_default())
    }


    pub(crate) fn block_sum_row (
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
        self.cap_span((top,bot))
    }


    pub(crate) fn block_graph_recurse (
            &self,
            graph: &mut StableGraph<(),(),Undirected>,
            left_side: Option<NodeIndex>,
        ) -> (Vec<BlockAdjSpan>,Vec<BlockAdjSpan>)
    {

        if self.kind.size_of() == 1 {
            return (Vec::new(),Vec::new());
        }

        let comp = match self.kind {
            Kind::Composite(comp) => comp,
            _ => {
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
                return (spans.clone(),spans)
            },
        };

        match comp.mode {
            CompositeMode::Product => self.block_product_stack(graph, left_side),
            CompositeMode::Sum     => self.block_sum_row(graph,left_side),
        }

    } 


    pub(crate) fn into_svg_recurse (&mut self,color_map : &HashMap<NodeIndex,usize>) -> Group {
   
        //let colors : [&str;8] = ["red","green","blue","magenta","cyan","yellow","grey","white" ];
        let colors : [&str;8] = ["#DDD","#BBB","#999","#777","#555","#333","#111","#222" ];
        //let colors : [&str;8] = ["#FBB","#D99","#B77","#955","#733","yellow","grey","white" ];
        //let colors : [&str;8] = ["#FFB","#9DD","#B7B","#595","#733","#115","grey","white" ];

        let label = self.kind.name_string();
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
        self.block_graph_recurse(&mut color_graph, None);
        println!("Coloring nodes!");
        let color_map = crate::graph::block_graph_color(&color_graph);
        println!("Generating SVG!");
        self.into_svg_recurse(&color_map)
    }


}

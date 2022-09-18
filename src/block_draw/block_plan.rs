use std::collections::HashMap;

use crate::{kind::{Kind, composite::CompositeMode}, block_draw::util::Translate};
use super::{BlockDrawSpec, util::{BlockAdjSpan, BlockAdjListPairIter, Vec2}};

use petgraph::{graph::NodeIndex, Undirected, stable_graph::StableGraph};
use svg::node::element::Group;

pub struct BlockBodyPlan {
    pub block_width: f32,
    pub notch: bool,
}

pub struct BlockDiagPlan<'kind> {
    pub spec: &'kind BlockDrawSpec,
    pub head: Group,
    pub head_offset: f32,
    pub body_plan: Option<BlockBodyPlan>,
    pub mins: Vec2,
    pub maxs: Vec2,
    pub kind: &'kind Kind<'kind>,
    pub sub_blocks: Vec<BlockDiagPlan<'kind>>,
    pub graph_index: Option<NodeIndex>,
    pub relative_pos: Option<Vec2>,
}

impl<'kind> BlockDiagPlan<'kind> {
    pub fn setup_nodes (
        &mut self,
        graph: &mut StableGraph<(), (), Undirected>,
        parent: Option<NodeIndex>,
    ) {
        if self.body_plan.is_some() {
            let index = *self.graph_index.get_or_insert_with(|| graph.add_node(()));
            if let Some(p_idx) = parent {
                graph.add_edge(p_idx, index, ());
            };
            for block in self.sub_blocks.iter_mut() {
                block.setup_nodes(graph, Some(index))
            }
        }
    }

    pub fn cap_span(
        &self,
        (mut top, mut bot): (Vec<BlockAdjSpan>, Vec<BlockAdjSpan>),
    ) -> (Vec<BlockAdjSpan>, Vec<BlockAdjSpan>)
    {
        let index = self.graph_index.unwrap();

        top.push(BlockAdjSpan {
            min: top.last().map(|last| last.max).unwrap_or(self.mins.x),
            max: self.maxs.x,
            index,
        });

        bot.push(BlockAdjSpan {
            min: bot.last().map(|last| last.max).unwrap_or(self.mins.x),
            max: self.maxs.x,
            index,
        });

        (top, bot)
    }

    pub fn block_product_stack(
        &self,
        graph: &mut StableGraph<(), (), Undirected>,
        left_side: Option<NodeIndex>,
    ) -> (Vec<BlockAdjSpan>,Vec<BlockAdjSpan>)
    {
        let mut interface: Option<(Vec<BlockAdjSpan>,Vec<BlockAdjSpan>)> = None;
        for block in &self.sub_blocks {
            let sub_pair = block.block_graph_recurse(graph, left_side);

            interface = match interface {
                None => Some(sub_pair),
                Some((top,bot)) => {
                    let (sub_top, sub_bot) = sub_pair;

                    for (a, b) in BlockAdjListPairIter::new(bot, sub_top) {
                        graph.add_edge(a, b, ());
                    };

                    Some((top, sub_bot))
                }
            }
        };

        self.cap_span(interface.unwrap_or_default())
    }

    pub fn block_sum_row(
        &self,
        graph: &mut StableGraph<(),(),Undirected>,
        left_side: Option<NodeIndex>,
    ) -> (Vec<BlockAdjSpan>,Vec<BlockAdjSpan>)
    {
        let index = self.graph_index.unwrap();
        let mut top = Vec::<BlockAdjSpan>::new();
        let mut bot = Vec::<BlockAdjSpan>::new();

        for (i, block) in self.sub_blocks.iter().enumerate() {
            dbg!(i);

            let lefty = if i == 0 {
                left_side
            } else {
                Some(index)
            };

            let (mut sub_top,mut sub_bot) = block.block_graph_recurse(graph, lefty);

            if let (Some(last),Some(first)) = (top.last(), sub_top.first()) {
                top.push(BlockAdjSpan {
                    index,
                    min: last.max,
                    max: first.min
                });
            }

            top.append(&mut sub_top);
            if let (Some(last), Some(first)) = (bot.last(), sub_bot.first()) {
                if block.maxs.y + 0.1 > self.maxs.y {
                    bot.push(BlockAdjSpan {
                        index,
                        min: last.max,
                        max: first.min
                    });
                }
            }
            bot.append(&mut sub_bot);
        }

        self.cap_span((top,bot))
    }

    pub fn block_graph_recurse(
        &self,
        graph: &mut StableGraph<(), (), Undirected>,
        left_side: Option<NodeIndex>,
    ) -> (Vec<BlockAdjSpan>,Vec<BlockAdjSpan>)
    {
        if self.kind.size_of() == 1 {
            return Default::default();
        }

        match self.kind {
            Kind::Composite(comp) => match comp.mode {
                CompositeMode::Product => self.block_product_stack(graph, left_side),
                CompositeMode::Sum => self.block_sum_row(graph,left_side),
            }
            Kind::Array(_) => self.block_product_stack(graph, left_side),
            _ => {
                let index = self.graph_index.unwrap();
                let spans = vec![BlockAdjSpan {
                    min: self.mins.x,
                    max: self.maxs.x,
                    index
                }];
                if let Some(lefty) = left_side {
                    graph.add_edge(index,lefty,());
                }
                (spans.clone(), spans)
            }
        }
    }

    pub fn into_svg_recurse(&mut self, color_map: &HashMap<NodeIndex, usize>) -> Group {
        //let colors : [&str;8] = ["red","green","blue","magenta","cyan","yellow","grey","white" ];
        let colors : [&str;8] = ["#DDD","#BBB","#999","#777","#555","#333","#111","#222" ];

        let base_pos = self.relative_pos.unwrap_or_default();

        let base_group = self.body_plan.as_ref()
            .map(|body_plan| {
                let tone = colors[color_map[&self.graph_index.unwrap()]];

                self.spec.draw_block(self.kind, body_plan.block_width, body_plan.notch)
                    .map(|group| group.set("fill", tone))
                    .unwrap_or_default()
            })
            .unwrap_or_default()
            .set("transform", Translate::from(base_pos));

        self.sub_blocks.iter_mut()
            .fold(base_group, |b,sub| b.add(sub.into_svg_recurse(color_map)))
            .add(self.head.clone()
                .set("transform",Translate(self.head_offset,0f32))
        )
    }

    pub fn into_svg (&mut self) -> Group {
        let mut color_graph = StableGraph::<(),(),Undirected>::default();
        self.setup_nodes(&mut color_graph, None);
        self.block_graph_recurse(&mut color_graph, None);
        let color_map = crate::graph::block_graph_color(&color_graph);
        self.into_svg_recurse(&color_map)
    }


    pub fn member_svg_recurse(&mut self, color_map: &HashMap<NodeIndex, usize>) -> Group {
        let base_pos = self.relative_pos.unwrap_or_default();

        let base_group = Group::new().set("transform", Translate::from(base_pos));

        self.sub_blocks.iter_mut()
            .fold(base_group, |b,sub| b.add(sub.into_svg_recurse(color_map)))
            .add(self.head.clone()
                .set("transform",Translate(self.head_offset,0f32))
        )
    }

    pub fn member_svg (&mut self) -> Group {
        let mut color_graph = StableGraph::<(),(),Undirected>::default();
        self.setup_nodes(&mut color_graph, None);
        self.block_graph_recurse(&mut color_graph, None);
        let color_map = crate::graph::block_graph_color(&color_graph);
        self.member_svg_recurse(&color_map)
    }

}

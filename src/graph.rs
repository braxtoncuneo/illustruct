use std::collections::HashMap;

use petgraph::{graph::NodeIndex, Undirected, stable_graph::StableGraph};

pub fn block_graph_color<N, E>(graph: &StableGraph<N,E,Undirected>) -> HashMap<NodeIndex, usize> {
    let mut cut_graph: StableGraph<Result<usize, ()>, (), Undirected> = graph.map(
        |_,_| Err(()),
        |_,_| (),
    );

    let mut cut_order : Vec<NodeIndex> = Vec::new();
    let mut cut_count = 0;
    let     cut_limit = cut_graph.node_count();

    while cut_count < cut_limit {
        let best = cut_graph.node_indices()
            .filter(|n| cut_graph.node_weight(*n).unwrap().is_err())
            .map(|n| {
                let best = cut_graph.neighbors(n)
                    .filter(|m| cut_graph.node_weight(*m).unwrap().is_err());
                (n, best.count())
            })
            .min_by_key(|(_, n)| *n)
            .unwrap().0;

        cut_order.push(best);
        *(cut_graph.node_weight_mut(best).unwrap()) = Ok(0);
        cut_count += 1;
    }

    cut_order.reverse();
    for index in cut_order {
        let mut n_weights : Vec<usize> = cut_graph
            .neighbors(index)
            .map(|n| (*cut_graph.node_weight(n).unwrap()).unwrap())
            .collect();

        n_weights.sort_unstable();

        let mut id: usize = 0;
        for w in n_weights {
            if w == id {
                id += 1;
            }
        }
        *cut_graph.node_weight_mut(index).unwrap().as_mut().unwrap() = id;
    }

    cut_graph.node_indices()
        .map(|n| (n, cut_graph.node_weight(n).unwrap().unwrap()))
        .collect()
}

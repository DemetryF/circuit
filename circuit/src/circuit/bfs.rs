use std::collections::VecDeque;
use std::iter;

use super::matrix::{MatrixGraph, NodeIndex};

pub fn bfs_nodes_as_undirected(
    graph: &MatrixGraph,
    start: NodeIndex,
) -> impl Iterator<Item = NodeIndex> + '_ {
    let mut visited = vec![false; graph.nodes_count()];
    let mut queue = VecDeque::from_iter([start]);

    iter::from_fn(move || {
        queue.pop_front().inspect(|&idx| {
            visited[idx.0] = true;

            queue.extend(graph.neighbour_nodes(idx).filter(|&idx| !visited[idx.0]));
        })
    })
}

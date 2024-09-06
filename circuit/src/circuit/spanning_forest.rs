use std::collections::VecDeque;
use std::iter;

use super::bfs::bfs_nodes_as_undirected;
use super::matrix::{MatrixGraph, NodeIndex};

#[derive(Default)]
pub struct SpanningForest {
    forest: MatrixGraph,
    roots: Vec<NodeIndex>,
}

impl SpanningForest {
    pub fn build(graph: &MatrixGraph) -> Self {
        let mut forest = MatrixGraph::new(graph.nodes_count());

        let mut visited = vec![false; graph.nodes_count()];

        let roots: Vec<_> = iter::from_fn(|| {
            let root = visited
                .iter()
                .enumerate()
                .find(|(_, &visited)| !visited)
                .map(|(idx, _)| NodeIndex(idx));

            root.inspect(|&root| {
                let mut queue = VecDeque::from_iter([root]);

                while let Some(idx) = queue.pop_front() {
                    visited[idx.0] = true;

                    graph
                        .neighbour_nodes(idx)
                        .filter(|&other_idx| !visited[other_idx.0])
                        .filter(|&other_idx| {
                            graph
                                .neighbour_nodes(other_idx)
                                .all(|other_idx| other_idx == idx || !visited[other_idx.0])
                        })
                        .for_each(|other_idx| {
                            forest.add_edge([idx, other_idx]);

                            queue.push_back(other_idx);
                        });
                }
            })
        })
        .collect();

        Self { forest, roots }
    }

    pub fn next_node(&mut self) {
        let new_node = self.forest.next_node();
        self.roots.push(new_node);
    }

    pub fn remove_node(&mut self, node: NodeIndex) {
        self.forest.remove_node(node);
    }

    pub fn add_edge(&mut self, endpoints: [NodeIndex; 2]) -> bool {
        let roots_and_indexes = endpoints.map(|endpoint| {
            bfs_nodes_as_undirected(&self.forest, endpoint)
                .find_map(|node| {
                    self.roots
                        .iter()
                        .position(|&root| node == root)
                        .map(|index| (index, node))
                })
                .unwrap()
        });

        let roots = roots_and_indexes.map(|(_, root)| root);
        let indexes = roots_and_indexes.map(|(index, _)| index);

        if roots[0] != roots[1] {
            self.roots.remove(indexes[1]);
            self.forest.add_edge(endpoints);

            true
        } else {
            false
        }
    }

    pub fn has_edge(&self, endpoints: [NodeIndex; 2]) -> bool {
        self.forest.has_edge(endpoints)
    }

    pub fn find_path(&self, endpoints: [NodeIndex; 2]) -> Vec<[NodeIndex; 2]> {
        let mut visited = vec![false; self.forest.nodes_count()];

        find_path(&self.forest, endpoints, &mut visited).unwrap()
    }
}

fn find_path(
    graph: &MatrixGraph,
    [start, end]: [NodeIndex; 2],
    visited: &mut Vec<bool>,
) -> Option<Vec<[NodeIndex; 2]>> {
    if graph.neighbour_nodes(start).any(|node| node == end) {
        return Some(vec![[end, start]]);
    } else {
        for new_start in graph.neighbour_nodes(start) {
            if visited[new_start.0] {
                continue;
            }

            visited[new_start.0] = true;

            if let Some(mut path) = find_path(graph, [new_start, end], visited) {
                path.push([new_start, start]);

                return Some(path);
            }
        }

        None
    }
}

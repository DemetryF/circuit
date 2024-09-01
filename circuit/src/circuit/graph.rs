use std::mem;

use nalgebra::DMatrix;

use super::matrix::{MatrixGraph, NodeIndex};
use super::spanning_forest::SpanningForest;

#[derive(Default)]
pub struct CircuitGraph {
    graph: MatrixGraph,
    spanning: SpanningForest,

    edges: Vec<[NodeIndex; 2]>,

    loops: DMatrix<f32>,
    loops_transposed: DMatrix<f32>,
}

impl CircuitGraph {
    pub fn next_node(&mut self) -> NodeIndex {
        self.spanning.next_node();
        self.graph.next_node()
    }

    pub fn remove_node(&mut self, node: NodeIndex) {
        self.spanning.remove_node(node);
        self.graph.remove_node(node);
    }

    pub fn add_edge(&mut self, endpoints: [NodeIndex; 2]) {
        self.graph.add_edge(endpoints);
        self.spanning.add_edge(endpoints);

        self.edges.push(endpoints);

        self.update_loops();
    }

    pub fn remove_edge(&mut self, endpoints: [NodeIndex; 2]) {
        self.graph.remove_edge(endpoints);

        self.edges
            .retain(|&other_endpoints| other_endpoints != endpoints);

        self.spanning = SpanningForest::build(&self.graph);

        self.update_loops()
    }

    pub fn loops(&self) -> (&DMatrix<f32>, &DMatrix<f32>) {
        (&self.loops, &self.loops_transposed)
    }

    pub fn edges(&mut self) -> impl Iterator<Item = [NodeIndex; 2]> + '_ {
        self.edges.iter().copied()
    }

    fn update_loops(&mut self) {
        self.loops = DMatrix::from_vec(0, self.edges.len(), vec![]);

        let cycles = fundamental_cycles(&self.graph, &self.spanning).enumerate();

        for (i, cycle) in cycles {
            self.loops = mem::take(&mut self.loops).insert_row(i, 0.0);

            for edge in cycle {
                let mut edges = self.edges.iter().enumerate();

                let reversed_edge = [edge[1], edge[0]];

                let (j, direction) = edges
                    .find_map(|(j, &other_edge)| {
                        Option::or(
                            (other_edge == edge).then_some((j, 1.0)),
                            (other_edge == reversed_edge).then_some((j, -1.0)),
                        )
                    })
                    .unwrap();

                self.loops[(i, j)] = direction;
            }
        }

        self.loops_transposed = self.loops.transpose();
    }
}

fn fundamental_cycles<'circuit>(
    graph: &'circuit MatrixGraph,
    spanning: &'circuit SpanningForest,
) -> impl Iterator<Item = Vec<[NodeIndex; 2]>> + 'circuit {
    let graph_xor_spanning = graph.edges().filter(|&edge| !spanning.has_edge(edge));

    graph_xor_spanning.map(|endpoints| {
        let mut path = spanning.find_path(endpoints);
        path.push(endpoints);
        path
    })
}

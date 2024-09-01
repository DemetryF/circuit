use std::mem;

use nalgebra::DMatrix;

#[derive(Default)]
pub struct MatrixGraph {
    matrix: DMatrix<f32>,
}

impl MatrixGraph {
    pub fn new(nodes_count: usize) -> Self {
        Self {
            matrix: DMatrix::from_vec(
                nodes_count,
                nodes_count,
                vec![0.0; nodes_count * nodes_count],
            ),
        }
    }

    pub fn add_edge(&mut self, endpoints: [NodeIndex; 2]) {
        let endpoints = endpoints.map(|idx| idx.0);

        self.matrix[(endpoints[0], endpoints[1])] = 1.0;
        self.matrix[(endpoints[1], endpoints[0])] = -1.0;
    }

    pub fn remove_edge(&mut self, endpoints: [NodeIndex; 2]) {
        let endpoints = endpoints.map(|idx| idx.0);

        self.matrix[(endpoints[0], endpoints[1])] = 0.0;
        self.matrix[(endpoints[1], endpoints[0])] = 0.0;
    }

    pub fn next_node(&mut self) -> NodeIndex {
        let new_size = self.nodes_count() + 1;

        self.matrix = mem::take(&mut self.matrix).resize(new_size, new_size, 0.0);

        NodeIndex(new_size - 1)
    }

    pub fn remove_node(&mut self, node: NodeIndex) {
        self.matrix = mem::take(&mut self.matrix)
            .remove_column(node.0)
            .remove_row(node.0);
    }

    pub fn has_edge(&self, endpoints: [NodeIndex; 2]) -> bool {
        let endpoints = endpoints.map(|idx| idx.0);

        self.matrix[(endpoints[0], endpoints[1])] == 1.0
            || self.matrix[(endpoints[1], endpoints[0])] == 1.0
    }

    pub fn nodes_count(&self) -> usize {
        self.matrix.ncols()
    }

    pub fn neighbour_nodes(&self, node: NodeIndex) -> impl Iterator<Item = NodeIndex> + '_ {
        let row = self.matrix.row(node.0).into_iter().copied().enumerate();

        row.filter(|&(_, v)| v.abs() == 1.0)
            .map(|(idx, _)| NodeIndex(idx))
    }

    pub fn edges(&self) -> impl Iterator<Item = [NodeIndex; 2]> + '_ {
        let matrix_values = self
            .matrix
            .iter()
            .enumerate()
            .map(|(i, &v)| (i / self.matrix.ncols(), i % self.matrix.ncols(), v));

        matrix_values
            .filter(|&(_, _, value)| value == 1.0)
            .map(|(y, x, _)| [NodeIndex(y), NodeIndex(x)])
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeIndex(pub(super) usize);

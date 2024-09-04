mod bfs;
mod graph;
mod matrix;
mod spanning_forest;

use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

use bimap::BiHashMap;
use nalgebra::DMatrix;

use crate::conductor::Conductor;
use graph::CircuitGraph;
use matrix::NodeIndex;

pub struct Circuit<'data, C, N>
where
    C: BorrowMut<dyn Conductor + 'data>,
    N: Copy + Hash + Eq,
{
    graph: CircuitGraph,
    elements: HashMap<ElementId, CircuitElement<'data, C>>,
    nodes: BiHashMap<NodeIndex, N>,

    ids: Vec<ElementId>,
    ids_count: usize,

    resistances: DMatrix<f32>,
    emf: DMatrix<f32>,

    lt: PhantomData<&'data ()>,
}

impl<'data, C, N> Circuit<'data, C, N>
where
    C: BorrowMut<dyn Conductor + 'data>,
    N: Copy + Hash + Eq,
{
    pub fn update(&mut self, delta_time: f32) {
        let (loops, loops_transposed) = self.graph.loops();

        for (i, id) in self.ids.iter().enumerate() {
            let conductor = self.elements[&id].conductor.borrow();

            self.resistances[(i, i)] = conductor.resistance();
            self.emf[(i, 0)] = conductor.emf();
        }

        let lhs = loops * &self.resistances * loops_transposed;
        let rhs = loops * &self.emf;

        let loop_currents = lhs.qr().solve(&rhs).unwrap();
        let edge_currents = loops_transposed * loop_currents;

        for (i, &id) in self.ids.iter().enumerate() {
            let current = edge_currents[i];
            let conductor = self.elements.get_mut(&id).unwrap().conductor.borrow_mut();

            conductor.zap(current, delta_time)
        }
    }

    pub fn add(&mut self, endpoints: [N; 2], conductor: C) -> ElementId {
        let endpoints = endpoints.map(|weight| {
            self.nodes
                .get_by_right(&weight)
                .copied()
                .unwrap_or_else(|| self.add_node(weight))
        });

        self.graph.add_edge(endpoints);

        let id = ElementId(self.ids_count);

        self.ids_count += 1;
        self.ids.push(id);

        let element = CircuitElement::new(endpoints, conductor);

        self.elements.insert(id, element);

        self.resize_matrices();

        id
    }

    pub fn change(&mut self, id: ElementId, new_endpoints: [N; 2]) {
        let new_endpoints = new_endpoints.map(|weight| {
            self.nodes
                .get_by_right(&weight)
                .copied()
                .unwrap_or_else(|| self.add_node(weight))
        });

        let old_endpoints = self.elements[&id].endpoints;

        for endpoint in old_endpoints {
            if !self
                .graph
                .edges()
                .any(|endpoints| endpoints.contains(&endpoint))
            {
                self.graph.remove_node(endpoint);
            }
        }

        self.elements.get_mut(&id).unwrap().endpoints = new_endpoints;

        self.ids.retain(|&other_id| other_id != id);
        self.ids.push(id);

        self.graph.remove_edge(old_endpoints);
        self.graph.add_edge(new_endpoints);
    }

    pub fn remove(&mut self, id: ElementId) {
        let element = self.elements.remove(&id).unwrap();
        let edge = element.endpoints;

        self.ids.retain(|&other_id| other_id != id);
        self.resize_matrices();

        self.graph.remove_edge(edge);
    }

    pub fn iter<'a: 'data>(&'a self) -> impl Iterator<Item = (ElementId, &C)> + 'data {
        self.elements
            .iter()
            .map(move |(&idx, element)| (idx, &element.conductor))
    }

    pub fn endpoints(&self, id: ElementId) -> [N; 2] {
        self.elements
            .get(&id)
            .unwrap()
            .endpoints
            .map(|idx| *self.nodes.get_by_left(&idx).unwrap())
    }

    pub fn get_mut(&mut self, id: ElementId) -> &mut C {
        &mut self.elements.get_mut(&id).unwrap().conductor
    }

    fn add_node(&mut self, weight: N) -> NodeIndex {
        let new_idx = self.graph.next_node();

        self.nodes.insert(new_idx, weight);

        new_idx
    }

    fn resize_matrices(&mut self) {
        let size = self.ids.len();

        self.resistances = DMatrix::from_element(size, size, 0.0);
        self.emf = DMatrix::from_element(size, 1, 0.0);
    }
}

impl<'data, C, N> Default for Circuit<'data, C, N>
where
    C: BorrowMut<dyn Conductor + 'data>,
    N: Copy + Hash + Eq + Default,
{
    fn default() -> Self {
        Self {
            graph: Default::default(),
            elements: Default::default(),
            nodes: Default::default(),
            resistances: Default::default(),
            emf: Default::default(),
            lt: Default::default(),
            ids: Default::default(),
            ids_count: Default::default(),
        }
    }
}

struct CircuitElement<'data, C: BorrowMut<dyn Conductor + 'data>> {
    pub endpoints: [NodeIndex; 2],
    pub conductor: C,
    lt: PhantomData<&'data ()>,
}

impl<'data, C: BorrowMut<dyn Conductor + 'data>> CircuitElement<'data, C> {
    pub fn new(endpoints: [NodeIndex; 2], conductor: C) -> Self {
        Self {
            endpoints,
            conductor,
            lt: PhantomData,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ElementId(usize);

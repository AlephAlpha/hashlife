use crate::rule::Rule;
use rustc_hash::FxHashMap;
use slab::Slab;
use std::{
    cell::Cell,
    ops::{Index, IndexMut},
};

#[derive(Hash, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub(crate) struct NodeId(u32);
pub(crate) type Leaf = u16;
const GC_THRESHOLD: usize = 1 << 24;

#[derive(Hash, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub(crate) enum Node {
    Leaf(Leaf),
    NodeId(NodeId),
}

#[derive(Hash, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub(crate) enum QuadChildren {
    Leaf {
        nw: Leaf,
        ne: Leaf,
        sw: Leaf,
        se: Leaf,
    },
    NodeId {
        nw: NodeId,
        ne: NodeId,
        sw: NodeId,
        se: NodeId,
    },
}

impl QuadChildren {
    fn new(nw: Node, ne: Node, sw: Node, se: Node) -> Self {
        match (nw, ne, sw, se) {
            (Node::Leaf(nw), Node::Leaf(ne), Node::Leaf(sw), Node::Leaf(se)) => {
                QuadChildren::Leaf { nw, ne, sw, se }
            }
            (Node::NodeId(nw), Node::NodeId(ne), Node::NodeId(sw), Node::NodeId(se)) => {
                QuadChildren::NodeId { nw, ne, sw, se }
            }
            _ => unreachable!("All children must have the same level."),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NodeData {
    pub(crate) level: u8,
    population: u64,
    pub(crate) children: QuadChildren,
    pub(crate) cache_step: Option<Node>,
    pub(crate) cache_step_max: Option<Node>,
    gc_mark: Cell<bool>,
}

impl NodeData {
    pub(crate) fn nw(&self) -> Node {
        match self.children {
            QuadChildren::NodeId { nw, .. } => Node::NodeId(nw),
            QuadChildren::Leaf { nw, .. } => Node::Leaf(nw),
        }
    }

    pub(crate) fn ne(&self) -> Node {
        match self.children {
            QuadChildren::NodeId { ne, .. } => Node::NodeId(ne),
            QuadChildren::Leaf { ne, .. } => Node::Leaf(ne),
        }
    }

    pub(crate) fn sw(&self) -> Node {
        match self.children {
            QuadChildren::NodeId { sw, .. } => Node::NodeId(sw),
            QuadChildren::Leaf { sw, .. } => Node::Leaf(sw),
        }
    }

    pub(crate) fn se(&self) -> Node {
        match self.children {
            QuadChildren::NodeId { se, .. } => Node::NodeId(se),
            QuadChildren::Leaf { se, .. } => Node::Leaf(se),
        }
    }
}

#[derive(Clone, Debug)]
pub struct World {
    pub(crate) rule: Rule,
    pub(crate) generation: u64,
    pub(crate) step: u8,
    hash_table: FxHashMap<QuadChildren, NodeId>,
    node_data: Slab<NodeData>,
    empty_nodes: Vec<Node>,
    pub(crate) root: Node,
    gc_threshold: usize,
}

impl Index<NodeId> for World {
    type Output = NodeData;

    fn index(&self, id: NodeId) -> &Self::Output {
        &self.node_data[id.0 as usize]
    }
}

impl IndexMut<NodeId> for World {
    fn index_mut(&mut self, id: NodeId) -> &mut Self::Output {
        &mut self.node_data[id.0 as usize]
    }
}

impl Default for World {
    fn default() -> Self {
        World::new("B3/S23".parse().unwrap())
    }
}

impl World {
    pub fn new(rule: Rule) -> Self {
        Self::new_with_step(rule, 0)
    }

    pub fn new_with_step(rule: Rule, step: u8) -> Self {
        let hash_table = FxHashMap::default();
        let node_data = Slab::new();
        let empty_nodes = Vec::new();
        let root = Node::Leaf(0);
        World {
            rule,
            generation: 0,
            step,
            hash_table,
            node_data,
            empty_nodes,
            root,
            gc_threshold: GC_THRESHOLD,
        }
    }

    pub fn population(&self) -> u64 {
        self.node_population(self.root)
    }

    pub fn get_cell(&mut self, x: i64, y: i64) -> bool {
        self.get_cell_rec(self.root, x, y)
    }

    pub fn set_cell(&mut self, x: i64, y: i64, state: bool) -> &mut Self {
        while {
            let offset = 1 << (self.node_level(self.root) - 2);
            x >= 2 * offset || x < -2 * offset || y >= 2 * offset || y < -2 * offset
        } {
            self.expand();
        }
        self.root = self.set_cell_rec(self.root, x, y, state);
        self
    }

    pub fn get_step(&self) -> u8 {
        self.step
    }

    pub fn set_step(&mut self, step: u8) -> &mut Self {
        self.clear_cache();
        self.step = step;
        self
    }

    pub fn get_generation(&self) -> u64 {
        self.generation
    }

    pub fn set_generation(&mut self, generation: u64) -> &mut Self {
        self.generation = generation;
        self
    }

    pub fn clear(&mut self, clear_nodes: bool) {
        if clear_nodes {
            self.hash_table.clear();
            self.empty_nodes.clear();
            self.node_data.clear();
        } else {
            self.clear_cache();
        }
        self.generation = 0;
        self.root = Node::Leaf(0);
    }

    pub fn garbage_collect(&mut self) {
        self.empty_nodes.last().map(|&node| self.mark_gc(node));
        self.mark_gc(self.root);
        let hash_table = &mut self.hash_table;
        hash_table.clear();
        self.node_data.retain(|i, data| {
            let mark = data.gc_mark.get();
            if mark {
                hash_table.insert(data.children, NodeId(i as u32));
                data.gc_mark.set(false);
            }
            mark
        });
    }

    pub fn bound(&self) -> Option<(i64, i64, i64, i64)> {
        match (
            self.left_bound(self.root),
            self.right_bound(self.root),
            self.top_bound(self.root),
            self.bottom_bound(self.root),
        ) {
            (Some(left), Some(right), Some(top), Some(bottom)) => Some((left, right, top, bottom)),
            (None, None, None, None) => None,
            _ => unreachable!(),
        }
    }

    pub(crate) fn check_gc(&mut self) {
        if self.node_data.len() >= self.gc_threshold {
            self.garbage_collect();
        }
    }

    fn mark_gc(&self, node: Node) {
        if let Node::NodeId(id) = node {
            let data = &self[id];
            if !data.gc_mark.get() {
                data.gc_mark.set(true);
                self.mark_gc(data.nw());
                self.mark_gc(data.ne());
                self.mark_gc(data.sw());
                self.mark_gc(data.se());
                data.cache_step.map(|node| self.mark_gc(node));
                data.cache_step_max.map(|node| self.mark_gc(node));
            }
        }
    }

    pub(crate) fn empty_node(&mut self, level: u8) -> Node {
        debug_assert!(level >= 2, "Level of a node must be >= 2");
        if self.empty_nodes.is_empty() {
            self.empty_nodes.push(Node::Leaf(0));
        }
        while self.empty_nodes.len() <= level as usize - 2 {
            let last = self.empty_nodes.last().unwrap().clone();
            let new = self.find_node(last, last, last, last);
            self.empty_nodes.push(Node::NodeId(new));
        }
        self.empty_nodes[level as usize - 2]
    }

    pub(crate) fn expand(&mut self) {
        match self.root {
            Node::Leaf(leaf) => {
                let nw = Node::Leaf((leaf & 0xcc00) >> 10);
                let ne = Node::Leaf((leaf & 0x3300) >> 6);
                let sw = Node::Leaf((leaf & 0x00cc) << 6);
                let se = Node::Leaf((leaf & 0x0033) << 10);
                self.root = Node::NodeId(self.find_node(nw, ne, sw, se));
            }
            Node::NodeId(id) => {
                let level = self[id].level;
                let empty = self.empty_node(level - 1);
                let nw = Node::NodeId(self.find_node(empty, empty, empty, self[id].nw()));
                let ne = Node::NodeId(self.find_node(empty, empty, self[id].ne(), empty));
                let sw = Node::NodeId(self.find_node(empty, self[id].sw(), empty, empty));
                let se = Node::NodeId(self.find_node(self[id].se(), empty, empty, empty));
                self.root = Node::NodeId(self.find_node(nw, ne, sw, se));
            }
        }
    }

    pub(crate) fn should_expand(&self) -> bool {
        match self.root {
            Node::Leaf(_) => true,
            Node::NodeId(id) => match self[id].children {
                QuadChildren::Leaf { nw, ne, sw, se } => {
                    nw & 0xfffe != 0 || ne & 0xfff7 != 0 || sw & 0xefff != 0 || se & 0x7fff != 0
                }
                QuadChildren::NodeId { nw, ne, sw, se } => {
                    let nw_se_se = match self[nw].se() {
                        Node::Leaf(leaf) => (leaf & 0x0033).count_ones() as u64,
                        Node::NodeId(id) => self.node_population(self[id].se()),
                    };
                    let ne_sw_sw = match self[ne].sw() {
                        Node::Leaf(leaf) => (leaf & 0x00cc).count_ones() as u64,
                        Node::NodeId(id) => self.node_population(self[id].sw()),
                    };
                    let sw_ne_ne = match self[sw].ne() {
                        Node::Leaf(leaf) => (leaf & 0x3300).count_ones() as u64,
                        Node::NodeId(id) => self.node_population(self[id].ne()),
                    };
                    let se_nw_nw = match self[se].nw() {
                        Node::Leaf(leaf) => (leaf & 0xcc00).count_ones() as u64,
                        Node::NodeId(id) => self.node_population(self[id].nw()),
                    };
                    self[nw].population != nw_se_se
                        || self[ne].population != ne_sw_sw
                        || self[sw].population != sw_ne_ne
                        || self[se].population != se_nw_nw
                }
            },
        }
    }

    fn clear_cache(&mut self) {
        self.node_data.iter_mut().for_each(|node| {
            node.1.cache_step.take();
        })
    }

    pub(crate) fn find_node(&mut self, nw: Node, ne: Node, sw: Node, se: Node) -> NodeId {
        let children = QuadChildren::new(nw, ne, sw, se);
        self.hash_table.get(&children).copied().unwrap_or_else(|| {
            let level = self.children_level(children) + 1;
            let population = self.children_population(children);
            let id = NodeId(self.node_data.insert(NodeData {
                level,
                population,
                children,
                cache_step: None,
                cache_step_max: None,
                gc_mark: Cell::new(false),
            }) as u32);
            self.hash_table.insert(children, id);
            id
        })
    }

    pub(crate) fn node_level(&self, node: Node) -> u8 {
        match node {
            Node::Leaf(_) => 2,
            Node::NodeId(id) => self[id].level,
        }
    }

    fn children_level(&self, children: QuadChildren) -> u8 {
        match children {
            QuadChildren::Leaf { .. } => 2,
            QuadChildren::NodeId { nw, .. } => self[nw].level,
        }
    }

    pub(crate) fn node_population(&self, node: Node) -> u64 {
        match node {
            Node::Leaf(leaf) => leaf.count_ones() as u64,
            Node::NodeId(id) => self[id].population,
        }
    }

    fn children_population(&self, children: QuadChildren) -> u64 {
        match children {
            QuadChildren::Leaf { nw, ne, sw, se } => {
                nw.count_ones() as u64
                    + ne.count_ones() as u64
                    + sw.count_ones() as u64
                    + se.count_ones() as u64
            }
            QuadChildren::NodeId { nw, ne, sw, se } => {
                self[nw].population
                    + self[ne].population
                    + self[sw].population
                    + self[se].population
            }
        }
    }

    fn get_cell_rec(&self, node: Node, x: i64, y: i64) -> bool {
        if self.node_population(node) == 0 {
            return false;
        }
        let node_size = 1 << (self.node_level(node) - 2);
        if x >= 2 * node_size || x < -2 * node_size || y >= 2 * node_size || y < -2 * node_size {
            return false;
        }
        match node {
            Node::Leaf(leaf) => leaf & 1 << ((1 - y) * 4 + (1 - x)) != 0,
            Node::NodeId(id) => match (x.is_negative(), y.is_negative()) {
                (true, true) => self.get_cell_rec(self[id].nw(), x + node_size, y + node_size),
                (false, true) => self.get_cell_rec(self[id].ne(), x - node_size, y + node_size),
                (true, false) => self.get_cell_rec(self[id].sw(), x + node_size, y - node_size),
                (false, false) => self.get_cell_rec(self[id].se(), x - node_size, y - node_size),
            },
        }
    }

    fn set_cell_rec(&mut self, node: Node, x: i64, y: i64, state: bool) -> Node {
        let node_size = 1 << (self.node_level(node) - 2);
        debug_assert!(
            x < 2 * node_size && x >= -2 * node_size && y < 2 * node_size && y >= -2 * node_size,
            "Cannot set cell outside of the node."
        );
        match node {
            Node::Leaf(leaf) => {
                if state {
                    Node::Leaf(leaf | 1 << ((1 - y) * 4 + (1 - x)))
                } else {
                    Node::Leaf(leaf & !(1 << ((1 - y) * 4 + (1 - x))))
                }
            }
            Node::NodeId(id) => {
                let data = &self[id];
                let mut nw = data.nw();
                let mut ne = data.ne();
                let mut sw = data.sw();
                let mut se = data.se();
                match (x.is_negative(), y.is_negative()) {
                    (true, true) => nw = self.set_cell_rec(nw, x + node_size, y + node_size, state),
                    (false, true) => {
                        ne = self.set_cell_rec(ne, x - node_size, y + node_size, state)
                    }
                    (true, false) => {
                        sw = self.set_cell_rec(sw, x + node_size, y - node_size, state)
                    }
                    (false, false) => {
                        se = self.set_cell_rec(se, x - node_size, y - node_size, state)
                    }
                }
                Node::NodeId(self.find_node(nw, ne, sw, se))
            }
        }
    }

    fn left_bound(&self, node: Node) -> Option<i64> {
        if self.node_population(node) == 0 {
            return None;
        }
        match node {
            Node::Leaf(leaf) => {
                if leaf & 0x8888 != 0 {
                    Some(-2)
                } else if leaf & 0x4444 != 0 {
                    Some(-1)
                } else if leaf & 0x2222 != 0 {
                    Some(0)
                } else if leaf & 0x1111 != 0 {
                    Some(1)
                } else {
                    None
                }
            }
            Node::NodeId(id) => {
                let node_size = 1 << (self.node_level(node) - 2);
                let data = &self[id];
                self.left_bound(data.nw())
                    .into_iter()
                    .chain(self.left_bound(data.sw()).into_iter())
                    .min()
                    .map(|min| min - node_size)
                    .or_else(|| {
                        self.left_bound(data.ne())
                            .into_iter()
                            .chain(self.left_bound(data.se()).into_iter())
                            .min()
                            .map(|min| min + node_size)
                    })
            }
        }
    }

    fn right_bound(&self, node: Node) -> Option<i64> {
        if self.node_population(node) == 0 {
            return None;
        }
        match node {
            Node::Leaf(leaf) => {
                if leaf & 0x1111 != 0 {
                    Some(2)
                } else if leaf & 0x2222 != 0 {
                    Some(1)
                } else if leaf & 0x4444 != 0 {
                    Some(0)
                } else if leaf & 0x8888 != 0 {
                    Some(-1)
                } else {
                    None
                }
            }
            Node::NodeId(id) => {
                let node_size = 1 << (self.node_level(node) - 2);
                let data = &self[id];
                self.right_bound(data.ne())
                    .into_iter()
                    .chain(self.right_bound(data.se()).into_iter())
                    .max()
                    .map(|max| max + node_size)
                    .or_else(|| {
                        self.right_bound(data.nw())
                            .into_iter()
                            .chain(self.right_bound(data.sw()).into_iter())
                            .max()
                            .map(|max| max - node_size)
                    })
            }
        }
    }

    fn top_bound(&self, node: Node) -> Option<i64> {
        if self.node_population(node) == 0 {
            return None;
        }
        match node {
            Node::Leaf(leaf) => {
                if leaf & 0xf000 != 0 {
                    Some(-2)
                } else if leaf & 0x0f00 != 0 {
                    Some(-1)
                } else if leaf & 0x00f0 != 0 {
                    Some(0)
                } else if leaf & 0x000f != 0 {
                    Some(1)
                } else {
                    None
                }
            }
            Node::NodeId(id) => {
                let node_size = 1 << (self.node_level(node) - 2);
                let data = &self[id];
                self.top_bound(data.nw())
                    .into_iter()
                    .chain(self.top_bound(data.ne()).into_iter())
                    .min()
                    .map(|min| min - node_size)
                    .or_else(|| {
                        self.top_bound(data.sw())
                            .into_iter()
                            .chain(self.top_bound(data.se()).into_iter())
                            .min()
                            .map(|min| min + node_size)
                    })
            }
        }
    }

    fn bottom_bound(&self, node: Node) -> Option<i64> {
        if self.node_population(node) == 0 {
            return None;
        }
        match node {
            Node::Leaf(leaf) => {
                if leaf & 0x000f != 0 {
                    Some(2)
                } else if leaf & 0x00f0 != 0 {
                    Some(1)
                } else if leaf & 0x0f00 != 0 {
                    Some(0)
                } else if leaf & 0xf000 != 0 {
                    Some(-1)
                } else {
                    None
                }
            }
            Node::NodeId(id) => {
                let node_size = 1 << (self.node_level(node) - 2);
                let data = &self[id];
                self.bottom_bound(data.sw())
                    .into_iter()
                    .chain(self.bottom_bound(data.se()).into_iter())
                    .max()
                    .map(|max| max + node_size)
                    .or_else(|| {
                        self.bottom_bound(data.nw())
                            .into_iter()
                            .chain(self.bottom_bound(data.ne()).into_iter())
                            .max()
                            .map(|max| max - node_size)
                    })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cell() {
        let mut world = World::default();
        world.set_step(8);
        world.root = Node::Leaf(0b_0000_0011_0110_0010);
        assert_eq!(world.get_cell(-10, -10), false);
        assert_eq!(world.get_cell(-2, -2), false);
        assert_eq!(world.get_cell(-1, 0), true);
        assert_eq!(world.get_cell(0, 1), true);
        world.step();
        assert_eq!(world.get_cell(0, 0), false);
        assert_eq!(world.get_cell(-29, -17), true);
        assert_eq!(world.get_cell(21, -6), true);
    }

    #[test]
    fn test_set_cell() {
        let mut world = World::default();
        world.set_step(8);
        let cells = vec![(0, -1), (1, -1), (-1, 0), (0, 0), (0, 1)];
        for (x, y) in cells {
            world.set_cell(x, y, true);
        }
        assert_eq!(world.root, Node::Leaf(0b_0000_0011_0110_0010));
        world.step();
        assert_eq!(world.population(), 141);
        world.set_cell(18, 8, false);
        assert_eq!(world.population(), 140);
        world.step();
        assert_eq!(world.population(), 97);
    }

    #[test]
    fn test_set_step() {
        let mut world = World::default();
        world.root = Node::Leaf(0b_0000_0011_0110_0010);
        assert_eq!(world.population(), 5);
        assert_eq!(world.get_generation(), 0);
        let populations = [6, 7, 9, 8, 9, 12, 11, 18];
        for &n in populations.iter() {
            world.step();
            assert_eq!(world.population(), n);
        }
        assert_eq!(world.get_generation(), 8);
        world.set_step(3);
        let populations = [23, 46, 32, 33, 45, 66, 69, 65];
        for &n in populations.iter() {
            world.step();
            assert_eq!(world.population(), n);
        }
        assert_eq!(world.get_generation(), 72);
        world.set_step(0);
        let populations = [70, 71, 73, 65, 69, 77, 75, 67];
        for &n in populations.iter() {
            world.step();
            assert_eq!(world.population(), n);
        }
        assert_eq!(world.get_generation(), 80);
    }

    #[test]
    fn test_gc() {
        let mut world = World::default();
        world.set_step(8);
        world.root = Node::Leaf(0b_0000_0011_0110_0010);
        assert_eq!(world.population(), 5);
        let populations = [141, 188, 204, 162, 116, 116, 116, 116];
        for &n in populations.iter() {
            world.step();
            world.garbage_collect();
            assert_eq!(world.population(), n);
        }
    }

    #[test]
    fn test_bound() {
        let mut world = World::default();
        world.set_step(8);
        world.root = Node::Leaf(0b_0000_0011_0110_0010);
        assert_eq!(world.bound(), Some((-1, 2, -1, 2)));
        world.step();
        assert_eq!(world.bound(), Some((-41, 48, -47, 54)));
    }
}

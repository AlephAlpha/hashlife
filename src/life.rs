use crate::rule::Rule;
use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};

/// The id of a node, i.e., its index in the world's node list.
#[derive(Hash, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
struct NodeId(usize);
/// A leaf, i.e., a 4x4 grid, represented by a 16-bit integer.
type Leaf = u16;

/// A node or a leaf.
#[derive(Hash, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
enum Node {
    Leaf(u16),
    NodeId(NodeId),
}

/// Four children of a node.
#[derive(Hash, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
enum QuadChildren {
    Leaf {
        nw: u16,
        ne: u16,
        sw: u16,
        se: u16,
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

    fn nw(self) -> Node {
        match self {
            QuadChildren::NodeId { nw, .. } => Node::NodeId(nw),
            QuadChildren::Leaf { nw, .. } => Node::Leaf(nw),
        }
    }

    fn ne(self) -> Node {
        match self {
            QuadChildren::NodeId { ne, .. } => Node::NodeId(ne),
            QuadChildren::Leaf { ne, .. } => Node::Leaf(ne),
        }
    }

    fn sw(self) -> Node {
        match self {
            QuadChildren::NodeId { sw, .. } => Node::NodeId(sw),
            QuadChildren::Leaf { sw, .. } => Node::Leaf(sw),
        }
    }

    fn se(self) -> Node {
        match self {
            QuadChildren::NodeId { se, .. } => Node::NodeId(se),
            QuadChildren::Leaf { se, .. } => Node::Leaf(se),
        }
    }
}

/// Children, cached results, and other data.
#[derive(Debug)]
struct NodeData {
    /// A node with level `n` represents a square grid with size `2.pow(n)`.
    level: u8,
    population: u64,
    children: QuadChildren,
    /// The result of evolving the specified number of steps.
    cache_step: Option<Node>,
    /// The result of evolving `2.pow(level - 2)` steps.
    cache_step_max: Option<Node>,
}

pub struct World {
    rule: Rule,
    generation: u64,
    /// The actual step is `2.pow(step)`.
    step: u8,
    hash_table: HashMap<QuadChildren, NodeId>,
    node_data: Vec<NodeData>,
    /// Cached empty nodes.
    empty_nodes: Vec<Node>,
    root: Node,
}

impl Index<NodeId> for World {
    type Output = NodeData;

    fn index(&self, id: NodeId) -> &Self::Output {
        &self.node_data[id.0]
    }
}

impl IndexMut<NodeId> for World {
    fn index_mut(&mut self, id: NodeId) -> &mut Self::Output {
        &mut self.node_data[id.0]
    }
}

impl World {
    pub fn new(rule: Rule, step: u8) -> Self {
        let hash_table = HashMap::new();
        let node_data = Vec::new();
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
        }
    }

    pub fn step(&mut self) {
        while self.node_level(self.root) <= self.step + 1 || self.should_expand() {
            self.expand()
        }
        self.root = self.step_node(self.root, self.step);
        self.generation += 1 << self.step;
    }

    pub fn population(&self) -> u64 {
        self.node_population(self.root)
    }

    pub fn get_cell(&mut self, x: i64, y: i64) -> bool {
        self.get_cell_node(self.root, x, y)
    }

    pub fn set_cell(&mut self, x: i64, y: i64, state: bool) {
        while {
            let offset = 1 << (self.node_level(self.root) - 2);
            x >= 2 * offset || x < -2 * offset || y >= 2 * offset || y < -2 * offset
        } {
            self.expand();
        }
        self.root = self.set_cell_node(self.root, x, y, state);
    }

    fn empty_node(&mut self, level: u8) -> Node {
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

    fn expand(&mut self) {
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
                let nw = Node::NodeId(self.find_node(empty, empty, empty, self[id].children.nw()));
                let ne = Node::NodeId(self.find_node(empty, empty, self[id].children.ne(), empty));
                let sw = Node::NodeId(self.find_node(empty, self[id].children.sw(), empty, empty));
                let se = Node::NodeId(self.find_node(self[id].children.se(), empty, empty, empty));
                self.root = Node::NodeId(self.find_node(nw, ne, sw, se));
            }
        }
    }

    fn should_expand(&self) -> bool {
        match self.root {
            Node::Leaf(_) => true,
            Node::NodeId(id) => match self[id].children {
                QuadChildren::Leaf { nw, ne, sw, se } => {
                    nw & 0xfffe != 0 || ne & 0xfff7 != 0 || sw & 0xefff != 0 || se & 0x7fff != 0
                }
                QuadChildren::NodeId { nw, ne, sw, se } => {
                    let nw_se_se = match self[nw].children.se() {
                        Node::Leaf(leaf) => (leaf & 0x0033).count_ones() as u64,
                        Node::NodeId(id) => self.node_population(self[id].children.se()),
                    };
                    let ne_sw_sw = match self[ne].children.sw() {
                        Node::Leaf(leaf) => (leaf & 0x00cc).count_ones() as u64,
                        Node::NodeId(id) => self.node_population(self[id].children.sw()),
                    };
                    let sw_ne_ne = match self[sw].children.ne() {
                        Node::Leaf(leaf) => (leaf & 0x3300).count_ones() as u64,
                        Node::NodeId(id) => self.node_population(self[id].children.ne()),
                    };
                    let se_nw_nw = match self[se].children.nw() {
                        Node::Leaf(leaf) => (leaf & 0xcc00).count_ones() as u64,
                        Node::NodeId(id) => self.node_population(self[id].children.nw()),
                    };
                    self[nw].population != nw_se_se
                        || self[ne].population != ne_sw_sw
                        || self[sw].population != sw_ne_ne
                        || self[se].population != se_nw_nw
                }
            },
        }
    }

    fn find_node(&mut self, nw: Node, ne: Node, sw: Node, se: Node) -> NodeId {
        let children = QuadChildren::new(nw, ne, sw, se);
        self.hash_table.get(&children).copied().unwrap_or_else(|| {
            let new_id = NodeId(self.node_data.len());
            let level = self.children_level(children) + 1;
            let population = self.children_population(children);
            self.hash_table.insert(children, new_id);
            self.node_data.push(NodeData {
                level,
                population,
                children,
                cache_step: None,
                cache_step_max: None,
            });
            new_id
        })
    }

    fn node_level(&self, node: Node) -> u8 {
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

    fn node_population(&self, node: Node) -> u64 {
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

    fn get_cell_node(&self, node: Node, x: i64, y: i64) -> bool {
        let offset = 1 << (self.node_level(node) - 2);
        if x >= 2 * offset || x < -2 * offset || y >= 2 * offset || y < -2 * offset {
            return false;
        }
        match node {
            Node::Leaf(leaf) => leaf & 1 << ((1 - y) * 4 + (1 - x)) != 0,
            Node::NodeId(id) => match (x.is_negative(), y.is_negative()) {
                (true, true) => self.get_cell_node(self[id].children.nw(), x + offset, y + offset),
                (false, true) => self.get_cell_node(self[id].children.ne(), x - offset, y + offset),
                (true, false) => self.get_cell_node(self[id].children.sw(), x + offset, y - offset),
                (false, false) => {
                    self.get_cell_node(self[id].children.se(), x - offset, y - offset)
                }
            },
        }
    }

    fn set_cell_node(&mut self, node: Node, x: i64, y: i64, state: bool) -> Node {
        let offset = 1 << (self.node_level(node) - 2);
        debug_assert!(
            x < 2 * offset && x >= -2 * offset && y < 2 * offset && y >= -2 * offset,
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
                let children = self[id].children;
                let mut nw = children.nw();
                let mut ne = children.ne();
                let mut sw = children.sw();
                let mut se = children.se();
                match (x.is_negative(), y.is_negative()) {
                    (true, true) => nw = self.set_cell_node(nw, x + offset, y + offset, state),
                    (false, true) => ne = self.set_cell_node(ne, x - offset, y + offset, state),
                    (true, false) => sw = self.set_cell_node(sw, x + offset, y - offset, state),
                    (false, false) => se = self.set_cell_node(se, x - offset, y - offset, state),
                }
                Node::NodeId(self.find_node(nw, ne, sw, se))
            }
        }
    }

    fn step_node(&mut self, node: Node, step: u8) -> Node {
        match node {
            Node::Leaf(leaf) => {
                debug_assert!(step == 0);
                Node::Leaf(self.step_leaf(leaf))
            }
            Node::NodeId(id) => self.step_id(id, step),
        }
    }

    fn step_leaf(&self, leaf: Leaf) -> Leaf {
        self.rule.rule_table[leaf as usize] as u16
    }

    fn step_id(&mut self, id: NodeId, step: u8) -> Node {
        let data = &self[id];
        debug_assert!(
            1 + step < data.level,
            "A level {} node cannot evolve for 2.pow({}) steps.",
            data.level,
            step
        );
        if step == self.step {
            if let Some(node) = data.cache_step {
                return node;
            }
        }
        let node = if step + 2 == data.level {
            self.step_max_id(id)
        } else {
            match data.children {
                QuadChildren::Leaf { nw, ne, sw, se } => self.step_quad_leaf(nw, ne, sw, se, 0),
                QuadChildren::NodeId { nw, ne, sw, se } => self.step_quad(nw, ne, sw, se, step),
            }
        };
        if step == self.step {
            self[id].cache_step = Some(node);
        }
        node
    }

    fn step_max_id(&mut self, id: NodeId) -> Node {
        let data = &self[id];
        match data.cache_step_max {
            Some(node) => node,
            None => {
                let node = match data.children {
                    QuadChildren::Leaf { nw, ne, sw, se } => self.step_quad_leaf(nw, ne, sw, se, 1),
                    QuadChildren::NodeId { nw, ne, sw, se } => self.step_max_quad(nw, ne, sw, se),
                };
                self[id].cache_step_max = Some(node);
                node
            }
        }
    }

    fn step_quad_leaf(&self, nw: Leaf, ne: Leaf, sw: Leaf, se: Leaf, step: u8) -> Node {
        debug_assert!(
            step < 2,
            "A level 3 node cannot evolve for 2.pow({}) steps.",
            step
        );
        let t00 = self.step_leaf(nw);
        let t01 = self.step_leaf((nw & 0x3333) << 2 | (ne & 0xcccc) >> 2);
        let t02 = self.step_leaf(ne);
        let t10 = self.step_leaf((nw & 0x00ff) << 8 | (sw & 0xff00) >> 8);
        let t11 = self.step_leaf(
            (nw & 0x0033) << 10 | (ne & 0x00cc) << 6 | (sw & 0x3300) >> 6 | (se & 0xcc00) >> 10,
        );
        let t12 = self.step_leaf((ne & 0x00ff) << 8 | (se & 0xff00) >> 8);
        let t20 = self.step_leaf(sw);
        let t21 = self.step_leaf((sw & 0x3333) << 2 | (se & 0xcccc) >> 2);
        let t22 = self.step_leaf(se);
        if step == 0 {
            Node::Leaf(
                (t00 & 0x01) << 15
                    | (t01 & 0x03) << 13
                    | (t02 & 0x02) << 11
                    | (t10 & 0x11) << 7
                    | (t11 & 0x33) << 5
                    | (t12 & 0x22) << 3
                    | (t20 & 0x10) >> 1
                    | (t21 & 0x30) >> 3
                    | (t22 & 0x20) >> 5,
            )
        } else {
            let new_nw = self.step_leaf(t00 << 10 | t01 << 8 | t10 << 2 | t11 << 0);
            let new_ne = self.step_leaf(t01 << 10 | t02 << 8 | t11 << 2 | t12 << 0);
            let new_sw = self.step_leaf(t10 << 10 | t11 << 8 | t20 << 2 | t21 << 0);
            let new_se = self.step_leaf(t11 << 10 | t12 << 8 | t21 << 2 | t22 << 0);
            Node::Leaf(new_nw << 10 | new_ne << 8 | new_sw << 2 | new_se << 0)
        }
    }

    fn step_quad(&mut self, nw: NodeId, ne: NodeId, sw: NodeId, se: NodeId, step: u8) -> Node {
        let n01 = self.find_node(
            self[nw].children.ne(),
            self[ne].children.nw(),
            self[nw].children.se(),
            self[ne].children.sw(),
        );
        let n10 = self.find_node(
            self[nw].children.sw(),
            self[nw].children.se(),
            self[sw].children.nw(),
            self[sw].children.ne(),
        );
        let n11 = self.find_node(
            self[nw].children.se(),
            self[ne].children.sw(),
            self[sw].children.ne(),
            self[se].children.nw(),
        );
        let n12 = self.find_node(
            self[ne].children.sw(),
            self[ne].children.se(),
            self[se].children.nw(),
            self[se].children.ne(),
        );
        let n21 = self.find_node(
            self[sw].children.ne(),
            self[se].children.nw(),
            self[sw].children.se(),
            self[se].children.sw(),
        );
        let t00 = self.step_id(nw, step);
        let t01 = self.step_id(n01, step);
        let t02 = self.step_id(ne, step);
        let t10 = self.step_id(n10, step);
        let t11 = self.step_id(n11, step);
        let t12 = self.step_id(n12, step);
        let t20 = self.step_id(sw, step);
        let t21 = self.step_id(n21, step);
        let t22 = self.step_id(se, step);
        match (t00, t01, t02, t10, t11, t12, t20, t21, t22) {
            (
                Node::Leaf(t00),
                Node::Leaf(t01),
                Node::Leaf(t02),
                Node::Leaf(t10),
                Node::Leaf(t11),
                Node::Leaf(t12),
                Node::Leaf(t20),
                Node::Leaf(t21),
                Node::Leaf(t22),
            ) => {
                let new_nw = Node::Leaf(
                    (t00 & 0x0033) << 10
                        | (t01 & 0x00cc) << 6
                        | (t10 & 0x3300) >> 6
                        | (t11 & 0xcc00) >> 10,
                );
                let new_ne = Node::Leaf(
                    (t01 & 0x0033) << 10
                        | (t02 & 0x00cc) << 6
                        | (t11 & 0x3300) >> 6
                        | (t12 & 0xcc00) >> 10,
                );
                let new_sw = Node::Leaf(
                    (t10 & 0x0033) << 10
                        | (t11 & 0x00cc) << 6
                        | (t20 & 0x3300) >> 6
                        | (t21 & 0xcc00) >> 10,
                );
                let new_se = Node::Leaf(
                    (t11 & 0x0033) << 10
                        | (t12 & 0x00cc) << 6
                        | (t21 & 0x3300) >> 6
                        | (t22 & 0xcc00) >> 10,
                );
                Node::NodeId(self.find_node(new_nw, new_ne, new_sw, new_se))
            }
            (
                Node::NodeId(t00),
                Node::NodeId(t01),
                Node::NodeId(t02),
                Node::NodeId(t10),
                Node::NodeId(t11),
                Node::NodeId(t12),
                Node::NodeId(t20),
                Node::NodeId(t21),
                Node::NodeId(t22),
            ) => {
                let new_nw = Node::NodeId(self.find_node(
                    self[t00].children.se(),
                    self[t01].children.sw(),
                    self[t10].children.ne(),
                    self[t11].children.nw(),
                ));
                let new_ne = Node::NodeId(self.find_node(
                    self[t01].children.se(),
                    self[t02].children.sw(),
                    self[t11].children.ne(),
                    self[t12].children.nw(),
                ));
                let new_sw = Node::NodeId(self.find_node(
                    self[t10].children.se(),
                    self[t11].children.sw(),
                    self[t20].children.ne(),
                    self[t21].children.nw(),
                ));
                let new_se = Node::NodeId(self.find_node(
                    self[t11].children.se(),
                    self[t12].children.sw(),
                    self[t21].children.ne(),
                    self[t22].children.nw(),
                ));
                Node::NodeId(self.find_node(new_nw, new_ne, new_sw, new_se))
            }
            _ => unreachable!("All children must have the same level."),
        }
    }

    fn step_max_quad(&mut self, nw: NodeId, ne: NodeId, sw: NodeId, se: NodeId) -> Node {
        let n01 = self.find_node(
            self[nw].children.ne(),
            self[ne].children.nw(),
            self[nw].children.se(),
            self[ne].children.sw(),
        );
        let n10 = self.find_node(
            self[nw].children.sw(),
            self[nw].children.se(),
            self[sw].children.nw(),
            self[sw].children.ne(),
        );
        let n11 = self.find_node(
            self[nw].children.se(),
            self[ne].children.sw(),
            self[sw].children.ne(),
            self[se].children.nw(),
        );
        let n12 = self.find_node(
            self[ne].children.sw(),
            self[ne].children.se(),
            self[se].children.nw(),
            self[se].children.ne(),
        );
        let n21 = self.find_node(
            self[sw].children.ne(),
            self[se].children.nw(),
            self[sw].children.se(),
            self[se].children.sw(),
        );
        let t00 = self.step_max_id(nw);
        let t01 = self.step_max_id(n01);
        let t02 = self.step_max_id(ne);
        let t10 = self.step_max_id(n10);
        let t11 = self.step_max_id(n11);
        let t12 = self.step_max_id(n12);
        let t20 = self.step_max_id(sw);
        let t21 = self.step_max_id(n21);
        let t22 = self.step_max_id(se);
        let pre_new_nw = self.find_node(t00, t01, t10, t11);
        let pre_new_ne = self.find_node(t01, t02, t11, t12);
        let pre_new_sw = self.find_node(t10, t11, t20, t21);
        let pre_new_se = self.find_node(t11, t12, t21, t22);
        let new_nw = self.step_max_id(pre_new_nw);
        let new_ne = self.step_max_id(pre_new_ne);
        let new_sw = self.step_max_id(pre_new_sw);
        let new_se = self.step_max_id(pre_new_se);
        Node::NodeId(self.find_node(new_nw, new_ne, new_sw, new_se))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leaf() {
        let world = World::new("B3/S23".parse().unwrap(), 0);
        macro_rules! test_leaf {
            ($leaf:expr, $result:expr $(,)?) => {{
                let result = world.step_leaf($leaf);
                assert_eq!(result, $result);
            }};
        }
        test_leaf!(0b_0000_0000_0000_0000, 0b_00_00_00);
        test_leaf!(0b_0101_1000_1010_0111, 0b_01_00_01);
        test_leaf!(0b_1011_1110_0000_1101, 0b_01_00_00);
        test_leaf!(0b_0111_0011_1010_0000, 0b_00_00_11);
        test_leaf!(0b_0000_0110_1000_1100, 0b_10_00_01);
    }

    #[test]
    fn test_level_3() {
        let world = World::new("B3/S23".parse().unwrap(), 0);
        macro_rules! test_level_3 {
            ($nw:expr, $ne:expr, $sw:expr, $se:expr, $result1:expr, $result2:expr $(,)?) => {{
                let result1 = world.step_quad_leaf($nw, $ne, $sw, $se, 0);
                assert_eq!(result1, Node::Leaf($result1));
                let result2 = world.step_quad_leaf($nw, $ne, $sw, $se, 1);
                assert_eq!(result2, Node::Leaf($result2));
            }};
        }
        test_level_3!(
            0b_0000_0000_0000_0000,
            0b_0000_0000_0000_0000,
            0b_0000_0000_0000_0000,
            0b_0000_0000_0000_0000,
            0b_0000_0000_0000_0000,
            0b_0000_0000_0000_0000,
        );
        test_level_3!(
            0b_0101_1000_1010_0111,
            0b_1011_1110_0000_1101,
            0b_0111_0011_1010_0000,
            0b_0000_0110_1000_1100,
            0b_1000_0010_0000_0001,
            0b_0001_0000_0000_0000,
        );
        test_level_3!(
            0b_0010_0001_0101_0100,
            0b_0001_0010_0101_1000,
            0b_1001_1000_1011_1011,
            0b_0101_0110_0111_1101,
            0b_0101_0101_0001_1100,
            0b_0100_0001_0100_1100,
        );
    }

    #[test]
    fn test_step_1() {
        let mut world = World::new("B3/S23".parse().unwrap(), 0);
        world.root = Node::Leaf(0b_0000_0011_0110_0010);
        let populations = [5, 6, 7, 9, 8, 9, 12, 11, 18];
        for &n in populations.iter() {
            assert_eq!(world.population(), n);
            world.step();
        }
    }

    #[test]
    fn test_step_256() {
        let mut world = World::new("B3/S23".parse().unwrap(), 8);
        world.root = Node::Leaf(0b_0000_0011_0110_0010);
        let populations = [5, 141, 188, 204, 162, 116, 116, 116, 116];
        for &n in populations.iter() {
            assert_eq!(world.population(), n);
            world.step();
        }
    }

    #[test]
    fn test_get_cell() {
        let mut world = World::new("B3/S23".parse().unwrap(), 8);
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
        let mut world = World::new("B3/S23".parse().unwrap(), 8);
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
}

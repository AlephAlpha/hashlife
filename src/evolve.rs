use crate::world::{Leaf, Node, NodeId, QuadChildren, World};

impl World {
    pub fn step(&mut self) {
        while self.node_level(self.root) <= self.step + 1 || self.should_expand() {
            self.expand()
        }
        debug_assert!(
            match self.root {
                Node::Leaf(_) => false,
                Node::NodeId(_) => true,
            },
            "The root cannot be a leaf after expansion."
        );
        self.root = self.step_node(self.root);
        self.generation += 1 << self.step;
        self.check_gc();
    }

    fn step_node(&mut self, node: Node) -> Node {
        match node {
            Node::Leaf(_) => unreachable!(),
            Node::NodeId(id) => self.step_id(id),
        }
    }

    const fn step_leaf(&self, leaf: Leaf) -> Leaf {
        self.rule.rule_table[leaf as usize] as Leaf
    }

    fn step_id(&mut self, id: NodeId) -> Node {
        let data = &self[id];
        if let Some(node) = data.cache_step {
            return node;
        }
        let max = self.step + 2 >= data.level;
        let node = match data.children {
            QuadChildren::Leaf { nw, ne, sw, se } => self.step_quad_leaf(nw, ne, sw, se, max),
            QuadChildren::NodeId { nw, ne, sw, se } => self.step_quad(nw, ne, sw, se, max),
        };
        self[id].cache_step = Some(node);
        node
    }

    const fn step_quad_leaf(&self, nw: Leaf, ne: Leaf, sw: Leaf, se: Leaf, max: bool) -> Node {
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
        if max {
            let new_nw = self.step_leaf(t00 << 10 | t01 << 8 | t10 << 2 | t11);
            let new_ne = self.step_leaf(t01 << 10 | t02 << 8 | t11 << 2 | t12);
            let new_sw = self.step_leaf(t10 << 10 | t11 << 8 | t20 << 2 | t21);
            let new_se = self.step_leaf(t11 << 10 | t12 << 8 | t21 << 2 | t22);
            Node::Leaf(new_nw << 10 | new_ne << 8 | new_sw << 2 | new_se)
        } else {
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
        }
    }

    fn central_node(&mut self, id: NodeId) -> Node {
        match self[id].children {
            QuadChildren::Leaf { nw, ne, sw, se } => Node::Leaf(
                (nw & 0x0033) << 10 | (ne & 0x00cc) << 6 | (sw & 0x3300) >> 6 | (se & 0xcc00) >> 10,
            ),
            QuadChildren::NodeId { nw, ne, sw, se } => {
                let new_nw = self[nw].se();
                let new_ne = self[ne].sw();
                let new_sw = self[sw].ne();
                let new_se = self[se].nw();
                Node::NodeId(self.find_node(new_nw, new_ne, new_sw, new_se))
            }
        }
    }

    fn step_quad(&mut self, nw: NodeId, ne: NodeId, sw: NodeId, se: NodeId, max: bool) -> Node {
        let n01 = self.find_node(self[nw].ne(), self[ne].nw(), self[nw].se(), self[ne].sw());
        let n10 = self.find_node(self[nw].sw(), self[nw].se(), self[sw].nw(), self[sw].ne());
        let n11 = self.find_node(self[nw].se(), self[ne].sw(), self[sw].ne(), self[se].nw());
        let n12 = self.find_node(self[ne].sw(), self[ne].se(), self[se].nw(), self[se].ne());
        let n21 = self.find_node(self[sw].ne(), self[se].nw(), self[sw].se(), self[se].sw());
        let (t00, t01, t02, t10, t11, t12, t20, t21, t22);
        if max {
            t00 = self.step_id(nw);
            t01 = self.step_id(n01);
            t02 = self.step_id(ne);
            t10 = self.step_id(n10);
            t11 = self.step_id(n11);
            t12 = self.step_id(n12);
            t20 = self.step_id(sw);
            t21 = self.step_id(n21);
            t22 = self.step_id(se);
        } else {
            t00 = self.central_node(nw);
            t01 = self.central_node(n01);
            t02 = self.central_node(ne);
            t10 = self.central_node(n10);
            t11 = self.central_node(n11);
            t12 = self.central_node(n12);
            t20 = self.central_node(sw);
            t21 = self.central_node(n21);
            t22 = self.central_node(se);
        }
        let pre_new_nw = self.find_node(t00, t01, t10, t11);
        let pre_new_ne = self.find_node(t01, t02, t11, t12);
        let pre_new_sw = self.find_node(t10, t11, t20, t21);
        let pre_new_se = self.find_node(t11, t12, t21, t22);
        let new_nw = self.step_id(pre_new_nw);
        let new_ne = self.step_id(pre_new_ne);
        let new_sw = self.step_id(pre_new_sw);
        let new_se = self.step_id(pre_new_se);
        Node::NodeId(self.find_node(new_nw, new_ne, new_sw, new_se))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leaf() {
        let world = World::default();

        fn test_leaf(world: &World, leaf: u16, expected: u16) {
            let result = world.step_leaf(leaf);
            assert_eq!(result, expected);
        }

        test_leaf(&world, 0b_0000_0000_0000_0000, 0b_00_00_00);
        test_leaf(&world, 0b_0101_1000_1010_0111, 0b_01_00_01);
        test_leaf(&world, 0b_1011_1110_0000_1101, 0b_01_00_00);
        test_leaf(&world, 0b_0111_0011_1010_0000, 0b_00_00_11);
        test_leaf(&world, 0b_0000_0110_1000_1100, 0b_10_00_01);
    }

    #[test]
    fn test_level_3() {
        let world = World::default();

        fn test_level_3(
            world: &World,
            nw: u16,
            ne: u16,
            sw: u16,
            se: u16,
            expected1: u16,
            expected2: u16,
        ) {
            let result1 = world.step_quad_leaf(nw, ne, sw, se, false);
            assert_eq!(result1, Node::Leaf(expected1));
            let result2 = world.step_quad_leaf(nw, ne, sw, se, true);
            assert_eq!(result2, Node::Leaf(expected2));
        }
        test_level_3(
            &world,
            0b_0000_0000_0000_0000,
            0b_0000_0000_0000_0000,
            0b_0000_0000_0000_0000,
            0b_0000_0000_0000_0000,
            0b_0000_0000_0000_0000,
            0b_0000_0000_0000_0000,
        );
        test_level_3(
            &world,
            0b_0101_1000_1010_0111,
            0b_1011_1110_0000_1101,
            0b_0111_0011_1010_0000,
            0b_0000_0110_1000_1100,
            0b_1000_0010_0000_0001,
            0b_0001_0000_0000_0000,
        );
        test_level_3(
            &world,
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
        let mut world = World::default();
        world.root = Node::Leaf(0b_0000_0011_0110_0010);
        assert_eq!(world.population(), 5);
        let populations = [6, 7, 9, 8, 9, 12, 11, 18];
        for &n in populations.iter() {
            world.step();
            assert_eq!(world.population(), n);
        }
    }

    #[test]
    fn test_step_256() {
        let mut world = World::default();
        world.set_step(8);
        world.root = Node::Leaf(0b_0000_0011_0110_0010);
        assert_eq!(world.population(), 5);
        let populations = [141, 188, 204, 162, 116, 116, 116, 116];
        for &n in populations.iter() {
            world.step();
            assert_eq!(world.population(), n);
        }
    }
}

use crate::world::{Node, World};

impl World {
    // Bound: (left, right, top, bottom).
    pub fn for_nodes<F>(&self, level: u8, bound: (i64, i64, i64, i64), f: F)
    where
        F: FnMut(i64, i64),
    {
        let mut f = f;
        self.for_nodes_rec(self.root, level, bound, (0, 0), &mut f)
    }

    // Bound: (left, right, top, bottom).
    pub fn for_living_cells<F>(&self, bound: (i64, i64, i64, i64), f: F)
    where
        F: FnMut(i64, i64),
    {
        self.for_nodes(0, bound, f)
    }

    fn for_nodes_rec<F>(
        &self,
        node: Node,
        level: u8,
        bound: (i64, i64, i64, i64),
        offset: (i64, i64),
        f: &mut F,
    ) where
        F: FnMut(i64, i64),
    {
        if self.node_population(node) != 0 {
            let node_level = self.node_level(node);
            let (left, right, top, bottom) = bound;

            if node_level <= level {
                if left <= 0 && right > 0 && top <= 0 && bottom > 0 {
                    f(offset.0, offset.1);
                }
            } else {
                match node {
                    Node::Leaf(leaf) => match level {
                        0 => {
                            let left = left.max(-2);
                            let right = right.min(2);
                            let top = top.max(-2);
                            let bottom = bottom.min(2);
                            for y in top..bottom {
                                for x in left..right {
                                    if leaf & 1 << ((1 - y) * 4 + (1 - x)) != 0 {
                                        f(x + offset.0, y + offset.1);
                                    }
                                }
                            }
                        }
                        1 => {
                            let left = left.max(-1);
                            let right = right.min(1);
                            let top = top.max(-1);
                            let bottom = bottom.min(1);
                            for y in top..bottom {
                                for x in left..right {
                                    if leaf & 0x0033 << (-8 * y - 2 * x) != 0 {
                                        f(x + offset.0, y + offset.1);
                                    }
                                }
                            }
                        }
                        _ => unreachable!(),
                    },
                    Node::NodeId(id) => {
                        let data = &self[id];
                        if node_level >= level + 2 {
                            let node_size = 1 << (node_level - level - 2);
                            if left < 0 && top < 0 {
                                self.for_nodes_rec(
                                    data.nw(),
                                    level,
                                    (
                                        left + node_size,
                                        right.min(0) + node_size,
                                        top + node_size,
                                        bottom.min(0) + node_size,
                                    ),
                                    (offset.0 - node_size, offset.1 - node_size),
                                    f,
                                );
                            }
                            if right > 0 && top < 0 {
                                self.for_nodes_rec(
                                    data.ne(),
                                    level,
                                    (
                                        left.max(0) - node_size,
                                        right - node_size,
                                        top + node_size,
                                        bottom.min(0) + node_size,
                                    ),
                                    (offset.0 + node_size, offset.1 - node_size),
                                    f,
                                );
                            }
                            if left < 0 && bottom > 0 {
                                self.for_nodes_rec(
                                    data.sw(),
                                    level,
                                    (
                                        left + node_size,
                                        right.min(0) + node_size,
                                        top.max(0) - node_size,
                                        bottom - node_size,
                                    ),
                                    (offset.0 - node_size, offset.1 + node_size),
                                    f,
                                );
                            }
                            if right > 0 && bottom > 0 {
                                self.for_nodes_rec(
                                    data.se(),
                                    level,
                                    (
                                        left.max(0) - node_size,
                                        right - node_size,
                                        top.max(0) - node_size,
                                        bottom - node_size,
                                    ),
                                    (offset.0 + node_size, offset.1 + node_size),
                                    f,
                                );
                            }
                        } else {
                            if left < 0 && top < 0 {
                                self.for_nodes_rec(
                                    data.nw(),
                                    level,
                                    (left + 1, right.min(0) + 1, top + 1, bottom.min(0) + 1),
                                    (offset.0 - 1, offset.1 - 1),
                                    f,
                                );
                            }
                            if right > 0 && top < 0 {
                                self.for_nodes_rec(
                                    data.ne(),
                                    level,
                                    (left.max(0), right, top + 1, bottom.min(0) + 1),
                                    (offset.0, offset.1 - 1),
                                    f,
                                );
                            }
                            if left < 0 && bottom > 0 {
                                self.for_nodes_rec(
                                    data.sw(),
                                    level,
                                    (left + 1, right.min(0) + 1, top.max(0), bottom),
                                    (offset.0 - 1, offset.1),
                                    f,
                                );
                            }
                            if right > 0 && bottom > 0 {
                                self.for_nodes_rec(
                                    data.se(),
                                    level,
                                    (left.max(0), right, top.max(0), bottom),
                                    (offset.0, offset.1),
                                    f,
                                );
                            }
                        };
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn test_for_living_cells() {
        let mut world = World::default();
        world.root = Node::Leaf(0b_0000_0011_0110_0010);
        world.step();
        let mut cells = Vec::new();
        world.for_living_cells((-2, 2, -2, 2), |x, y| cells.push((x, y)));
        assert_eq!(
            cells,
            vec![(-1, -1), (0, -1), (1, -1), (-1, 0), (-1, 1), (0, 1)]
        );
        cells.clear();
        world.for_nodes(1, (-2, 2, -2, 2), |x, y| cells.push((x, y)));
        assert_eq!(cells, vec![(-1, -1), (0, -1), (-1, 0), (0, 0)]);
        world.set_step(3);
        world.step();
        assert_eq!(world.get_generation(), 9);
        cells.clear();
        world.for_nodes(2, (-2, 2, -2, 2), |x, y| cells.push((x, y)));
        assert_eq!(cells, vec![(-1, -1), (0, -1), (-1, 0), (0, 0)]);
    }
}

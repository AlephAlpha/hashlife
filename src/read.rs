use crate::world::{Leaf, Node, World};
use ca_formats::{
    macrocell::{Error as MacrocellError, Macrocell, NodeData},
    rle::{Error as RleError, Rle},
    Input,
};

impl World {
    pub fn from_rle<I: Input>(rle: Rle<I>) -> Result<Self, RleError> {
        let rule = rle
            .header_data()
            .and_then(|header| header.rule.as_deref())
            .and_then(|rulestring| rulestring.parse().ok())
            .unwrap_or_else(|| "B3/S23".parse().unwrap());
        let mut world = Self::new(rule);
        for cell in rle {
            let (x, y) = cell?.position;
            world.set_cell(x, y, true);
        }
        Ok(world)
    }

    pub fn from_macrocell<I: Input>(macrocell: Macrocell<I>) -> Result<Self, MacrocellError> {
        let rule = macrocell
            .rule()
            .and_then(|rulestring| rulestring.parse().ok())
            .unwrap_or_else(|| "B3/S23".parse().unwrap());
        let mut world = Self::new(rule);
        let mut nodes = Vec::new();
        for node in macrocell {
            let node = node?;
            debug_assert_eq!(nodes.len() + 1, node.id);
            let node = match node.data {
                NodeData::Level1 { .. } => {
                    return Err(MacrocellError::InvalidNodeLine(
                        "Rules with more than 2 states are not supported".to_string(),
                    ))
                }
                NodeData::Level3(n) => {
                    let nw = Node::Leaf(
                        ((n & 0x_f000_0000_0000_0000) >> 48
                            | (n & 0x_00f0_0000_0000_0000) >> 44
                            | (n & 0x_0000_f000_0000_0000) >> 40
                            | (n & 0x_0000_00f0_0000_0000) >> 36) as Leaf,
                    );
                    let ne = Node::Leaf(
                        ((n & 0x_0f00_0000_0000_0000) >> 44
                            | (n & 0x_000f_0000_0000_0000) >> 40
                            | (n & 0x_0000_0f00_0000_0000) >> 36
                            | (n & 0x_0000_000f_0000_0000) >> 32) as Leaf,
                    );
                    let sw = Node::Leaf(
                        ((n & 0x_0000_0000_f000_0000) >> 16
                            | (n & 0x_0000_0000_00f0_0000) >> 12
                            | (n & 0x_0000_0000_0000_f000) >> 8
                            | (n & 0x_0000_0000_0000_00f0) >> 4) as Leaf,
                    );
                    let se = Node::Leaf(
                        ((n & 0x_0000_0000_0f00_0000) >> 12
                            | (n & 0x_0000_0000_000f_0000) >> 8
                            | (n & 0x_0000_0000_0000_0f00) >> 4
                            | (n & 0x_0000_0000_0000_000f)) as Leaf,
                    );
                    world.find_node(nw, ne, sw, se)
                }
                NodeData::Node {
                    level,
                    nw,
                    ne,
                    sw,
                    se,
                } => {
                    let nw = if nw == 0 {
                        world.empty_node(level - 1)
                    } else {
                        Node::NodeId(nodes[nw - 1])
                    };
                    let ne = if ne == 0 {
                        world.empty_node(level - 1)
                    } else {
                        Node::NodeId(nodes[ne - 1])
                    };
                    let sw = if sw == 0 {
                        world.empty_node(level - 1)
                    } else {
                        Node::NodeId(nodes[sw - 1])
                    };
                    let se = if se == 0 {
                        world.empty_node(level - 1)
                    } else {
                        Node::NodeId(nodes[se - 1])
                    };
                    world.find_node(nw, ne, sw, se)
                }
            };
            nodes.push(node);
        }
        world.root = Node::NodeId(nodes.pop().unwrap());
        Ok(world)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_macrocell() {
        let macrocell = Macrocell::new(include_str!("../patterns/totalperiodic.mc")).unwrap();
        let world = World::from_macrocell(macrocell).unwrap();
        assert_eq!(world.population(), 196);
    }
}

#[cfg(feature = "graphviz")]
mod graphviz;

mod algorithms;

use std::sync::Arc;

use indexmap::{map::Entry, IndexMap};
use once_cell::sync::OnceCell;
use ordered_float::NotNan;

pub type Cost = NotNan<f64>;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(Arc<str>);

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ClassId(Arc<str>);

mod id_impls {
    use super::*;

    impl AsRef<str> for NodeId {
        fn as_ref(&self) -> &str {
            &self.0
        }
    }

    impl<S: Into<String>> From<S> for NodeId {
        fn from(s: S) -> Self {
            Self(s.into().into())
        }
    }

    impl std::fmt::Display for NodeId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl AsRef<str> for ClassId {
        fn as_ref(&self) -> &str {
            &self.0
        }
    }

    impl<S: Into<String>> From<S> for ClassId {
        fn from(s: S) -> Self {
            Self(s.into().into())
        }
    }

    impl std::fmt::Display for ClassId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EGraph {
    pub nodes: IndexMap<NodeId, Node>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub root_eclasses: Vec<ClassId>,
    // Optional mapping of e-class ids to some additional data about the e-class
    #[cfg_attr(feature = "serde", serde(default))]
    pub class_data: IndexMap<ClassId, ClassData>,
    #[cfg_attr(feature = "serde", serde(skip))]
    once_cell_classes: OnceCell<IndexMap<ClassId, Class>>,
}

impl EGraph {
    /// Adds a new node to the egraph
    ///
    /// Panics if a node with the same id already exists
    pub fn add_node(&mut self, node_id: impl Into<NodeId>, node: Node) {
        match self.nodes.entry(node_id.into()) {
            Entry::Occupied(e) => {
                panic!(
                    "Duplicate node with id {key:?}\nold: {old:?}\nnew: {new:?}",
                    key = e.key(),
                    old = e.get(),
                    new = node
                )
            }
            Entry::Vacant(e) => e.insert(node),
        };
    }

    pub fn nid_to_cid(&self, node_id: &NodeId) -> &ClassId {
        &self[node_id].eclass
    }

    pub fn nid_to_class(&self, node_id: &NodeId) -> &Class {
        &self[&self[node_id].eclass]
    }

    /// Groups the nodes in the e-graph by their e-class
    ///
    /// This is *only done once* and then the result is cached.
    /// Modifications to the e-graph will not be reflected
    /// in later calls to this function.
    pub fn classes(&self) -> &IndexMap<ClassId, Class> {
        self.once_cell_classes.get_or_init(|| {
            let mut classes = IndexMap::new();
            for (node_id, node) in &self.nodes {
                classes
                    .entry(node.eclass.clone())
                    .or_insert_with(|| Class {
                        id: node.eclass.clone(),
                        nodes: vec![],
                    })
                    .nodes
                    .push(node_id.clone())
            }
            classes
        })
    }

    #[cfg(feature = "serde")]
    pub fn from_json_file(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let egraph: Self = serde_json::from_reader(std::io::BufReader::new(file))?;
        Ok(egraph)
    }

    #[cfg(feature = "serde")]
    pub fn to_json_file(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        let file = std::fs::File::create(path)?;
        serde_json::to_writer_pretty(std::io::BufWriter::new(file), self)?;
        Ok(())
    }

    #[cfg(feature = "serde")]
    pub fn test_round_trip(&self) {
        let json = serde_json::to_string_pretty(&self).unwrap();
        let egraph2: EGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(self, &egraph2);
    }
}

impl std::ops::Index<&NodeId> for EGraph {
    type Output = Node;

    fn index(&self, index: &NodeId) -> &Self::Output {
        self.nodes
            .get(index)
            .unwrap_or_else(|| panic!("No node with id {:?}", index))
    }
}

impl std::ops::Index<&ClassId> for EGraph {
    type Output = Class;

    fn index(&self, index: &ClassId) -> &Self::Output {
        self.classes()
            .get(index)
            .unwrap_or_else(|| panic!("No class with id {:?}", index))
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Node {
    pub op: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub children: Vec<NodeId>,
    pub eclass: ClassId,
    #[cfg_attr(feature = "serde", serde(default = "one"))]
    pub cost: Cost,
}

impl Node {
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

fn one() -> Cost {
    Cost::new(1.0).unwrap()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Class {
    pub id: ClassId,
    pub nodes: Vec<NodeId>,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassData {
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub typ: Option<String>,
}

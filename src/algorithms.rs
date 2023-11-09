use crate::EGraph;

pub const MISSING_ARG_VALUE: &str = "·";

impl EGraph {
    /// Inline all leaves (e-classes with a single node that has no children) into their parents, so that they
    /// are added to the function name like f(10, ·).
    /// Returns the number of leaves inlined.
    pub fn inline_leaves(&mut self) -> usize {
        // 1. Create mapping of eclass to nodes as well as nodes to their parents
        let mut eclass_to_nodes = std::collections::HashMap::new();
        let mut node_to_parents = std::collections::HashMap::new();
        for (node_id, node) in &self.nodes {
            eclass_to_nodes
                .entry(node.eclass.clone())
                .or_insert_with(Vec::new)
                .push((node_id.clone(), node));
            for child in &node.children {
                node_to_parents
                    .entry(child.clone())
                    .or_insert_with(Vec::new)
                    .push(node_id.clone());
            }
        }
        // 2. Find all leaves (e-classes with a single node that has no children and also not in root-eclasses)
        let mut leaves = Vec::new();
        let mut leave_to_op = std::collections::HashMap::new();
        for (eclass, nodes) in eclass_to_nodes {
            if nodes.len() == 1 && nodes[0].1.children.is_empty() {
                leaves.push((eclass, nodes[0].0.clone()));
                leave_to_op.insert(nodes[0].0.clone(), nodes[0].1.op.clone());
            }
        }
        // 3. Create mapping from all parents which are updated to the children which are inlined
        let mut parents_to_children = std::collections::HashMap::new();
        for (_, node_id) in &leaves {
            let parents = node_to_parents.get(node_id);
            // There will be no parents for isolated nodes with no parents or children
            if let Some(parents) = parents {
                for parent in parents {
                    parents_to_children
                        .entry(parent.clone())
                        .or_insert_with(Vec::new)
                        .push(node_id.clone());
                }
            }
        }
        // 4. Inline leaf nodes into their parents
        for (parent, leaf_children) in &parents_to_children {
            let additional_cost = leaf_children
                .iter()
                .map(|child| self.nodes.get(child).unwrap().cost)
                .sum::<ordered_float::NotNan<f64>>();
            let parent_node = self.nodes.get_mut(parent).unwrap();
            let args = parent_node
                .children
                .iter()
                .map(|child| {
                    if leaf_children.contains(child) {
                        leave_to_op.get(child).unwrap()
                    } else {
                        MISSING_ARG_VALUE
                    }
                })
                .collect::<Vec<_>>();
            // Remove leaf children from children
            parent_node
                .children
                .retain(|child| !leaf_children.contains(child));
            // If the parent node already had some children replaced, then just replace the remaining children
            // otherwise, replace the entire op
            let new_op = if parent_node.op.matches(MISSING_ARG_VALUE).count() == args.len() {
                // Replace all instances of MISSING_ARG_VALUE with the corresponding arg by interleaving
                // the op split by MISSING_ARG_VALUE with the args
                parent_node
                    .op
                    .split(MISSING_ARG_VALUE)
                    .enumerate()
                    .flat_map(|(i, s)| {
                        if i == args.len() {
                            vec![s.to_string()]
                        } else {
                            vec![s.to_string(), args[i].to_string()]
                        }
                    })
                    .collect::<String>()
            } else {
                format!("{}({})", parent_node.op, args.join(", "))
            };
            parent_node.op = new_op;
            parent_node.cost += additional_cost;
        }
        let mut n_inlined = 0;
        // 5. Remove leaf nodes from egraph, class data, and root eclasses
        for (eclass, node_id) in &leaves {
            // If this node has no parents, don't remove it, since it wasn't inlined
            if node_to_parents.get(node_id).is_none() {
                continue;
            }
            n_inlined += 1;
            self.nodes.remove(node_id);
            self.class_data.remove(eclass);
            self.root_eclasses.retain(|root| root != eclass);
        }
        n_inlined
    }

    /// Inline all leaves (e-classes with a single node that has no children) into their parents, recursively.
    pub fn saturate_inline_leaves(&mut self) {
        while self.inline_leaves() > 0 {}
    }
}

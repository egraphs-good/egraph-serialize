use std::{fmt, io::Write};

use crate::EGraph;
use graphviz_rust::{
    attributes::*,
    dot_generator::*,
    dot_structures::{
        Attribute, Edge, EdgeTy, Graph, GraphAttributes as GA, Id, Node, NodeId, Port, Stmt,
        Subgraph, Vertex,
    },
    printer::{DotPrinter, PrinterContext},
};

impl EGraph {
    pub fn to_dot(&self) -> String {
        self.to_graphviz().print(&mut PrinterContext::default())
    }

    pub fn to_svg_file(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        graphviz_rust::exec_dot(
            self.to_dot(),
            vec![
                graphviz_rust::cmd::Format::Svg.into(),
                graphviz_rust::cmd::CommandArg::Output(path.as_ref().to_str().unwrap().to_string()),
            ],
        )?;
        Ok(())
    }

    pub fn to_dot_file(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        let mut file = std::fs::File::create(path)?;
        file.write_all(self.to_dot().as_bytes())?;
        Ok(())
    }

    fn to_graphviz(&self) -> Graph {
        // 1. Group nodes by type and class (use BTreeMap to keep sorted so colors are consistent)
        let mut class_nodes = std::collections::BTreeMap::new();
        //  and create mapping from each node ID to its class
        let mut node_to_class = std::collections::HashMap::new();
        for (node_id, node) in &self.nodes {
            let typ = self
                .class_data
                .get(&node.eclass)
                .and_then(|data| data.typ.clone());
            node_to_class.insert(node_id.clone(), node.eclass.clone());
            class_nodes
                .entry(typ)
                .or_insert_with(std::collections::HashMap::new)
                .entry(node.eclass.clone())
                .or_insert_with(Vec::new)
                .push((node_id.clone(), node));
        }
        // 2. Start with configuration
        let mut stmts = vec![
            // Set to compound so we can have edge to clusters
            stmt!(GraphAttributes::compound(true)),
            // Set default sub-graph rank to be same so that all nodes in e-class are on same level
            stmt!(SubgraphAttributes::rank(rank::same)),
            stmt!(GraphAttributes::fontname("helvetica".to_string())),
            stmt!(GraphAttributes::fontsize(9.0)),
            stmt!(GraphAttributes::margin(3.0)),
            stmt!(GraphAttributes::nodesep(0.05)),
            stmt!(GraphAttributes::ranksep(0.6)),
            stmt!(GraphAttributes::colorscheme("set312".to_string())),
            stmt!(GA::Edge(vec![EdgeAttributes::arrowsize(0.5)])),
            stmt!(GA::Node(vec![
                NodeAttributes::shape(shape::none),
                NodeAttributes::margin(0.0),
                NodeAttributes::fontname("helvetica".to_string())
            ])),
            // Draw edges first, so that they are behind nodes
            stmt!(GraphAttributes::outputorder(outputorder::edgesfirst)),
            stmt!(GA::Graph(vec![GraphAttributes::style(quote(
                "dashed,rounded,filled"
            ))])),
        ];
        // 3. Add each e-class

        // Mapping of sort names to color index
        let mut typ_colors = std::collections::HashMap::new();

        for (typ, class_to_node) in class_nodes {
            let next_color = (typ_colors.len() + INITIAL_COLOR) % N_COLORS;
            let color = typ_colors.entry(typ).or_insert(next_color);
            stmts.push(stmt!(attr!("fillcolor", color)));
            for (class_id, nodes) in class_to_node {
                let mut inner_stmts = vec![];

                // Add nodes
                for (node_id, node) in nodes {
                    let label = node.op.as_ref();
                    let tooltip = format!("{}: {}", class_id, node_id);
                    let html_label = html_label(label, node.children.len());
                    let quoted_tooltip = quote(&tooltip);
                    let quoted_node_id = quote(node_id.as_ref());
                    // Add edges
                    for (i, child) in node.children.iter().enumerate() {
                        let source = node_id!(quoted_node_id, port!(id!(i), "s"));
                        let target = node_id!(quote(child.as_ref()));
                        let child_eclass = node_to_class.get(child).unwrap();
                        let child_subgraph_id = format!("cluster_{}", child_eclass);
                        let edge = edge!(source => target; EdgeAttributes::lhead(quote(&child_subgraph_id)));
                        // Make sure edge is part of outer statements so it doesn't add nodes to the subgraph which
                        // don't belong there
                        stmts.push(stmt!(edge));
                    }
                    let node = node!(quoted_node_id;NodeAttributes::label(html_label), NodeAttributes::tooltip(quoted_tooltip));
                    inner_stmts.push(stmt!(node));
                }

                let subgraph_id = format!("cluster_{}", class_id);
                let outer_subgraph_id = quote(&format!("outer_{}", subgraph_id));
                let quoted_subgraph_id = quote(&subgraph_id);

                let subgraph = subgraph!(outer_subgraph_id;
                    // Disable label for now, to reduce size
                    // NodeAttributes::label(subgraph_html_label(&typ)),

                    // Nest in empty sub-graph so that we can use rank=same
                    // https://stackoverflow.com/a/55562026/907060
                    subgraph!(quoted_subgraph_id; subgraph!("", inner_stmts)),

                    // Make outer subgraph a cluster but make it invisible, so just used for padding
                    // https://forum.graphviz.org/t/how-to-add-space-between-clusters/1209/3
                    SubgraphAttributes::style(quote("invis")),
                    attr!("cluster", "true")
                );
                // If this is a root e-class, make the border bold
                if self.root_eclasses.contains(&class_id) {
                    stmts.push(stmt!(attr!("penwidth", 2)));
                }
                stmts.push(stmt!(subgraph));
            }
        }
        // Set margin to 0 at the end again, so that total graph margin is 0, but all the clusters
        // defined above have some margins
        stmts.push(stmt!(GraphAttributes::margin(0.0)));
        graph!(di id!(), stmts)
    }
}

// Number of colors in the graphviz color scheme
// https://graphviz.org/doc/info/colors.html
const N_COLORS: usize = 12;
// Initial color to use for the first type
const INITIAL_COLOR: usize = 2;

/// Returns an html label for the node with the function name and ports for each argumetn
fn html_label(label: &str, n_args: usize) -> String {
    format!(
        "<<TABLE BGCOLOR=\"white\" CELLBORDER=\"0\" CELLSPACING=\"0\" CELLPADDING=\"0\" style=\"rounded\"><tr><td BALIGN=\"left\" CELLPADDING=\"4\" WIDTH=\"30\" HEIGHT=\"30\"{}>{}</td></tr>{}</TABLE>>",
        (if n_args  == 0 {"".to_string()} else {format!(" colspan=\"{}\"", n_args)}),
        Escape(label),
        (if n_args == 0 {
            "".to_string()
        } else {
            format!(
                "<TR>{}</TR>",
                (0..n_args)
                    .map(|i| format!("<TD PORT=\"{}\"></TD>", i))
                    .collect::<Vec<String>>()
                    .join("")
            )
        })
    )
}

/// Adds double quotes and escapes the quotes in the string
fn quote(s: &str) -> String {
    format!("{:?}", s)
}

// Copied from https://doc.rust-lang.org/stable/nightly-rustc/src/rustdoc/html/escape.rs.html#10
// but added conversion of \n to <br/>

/// Wrapper struct which will emit the HTML-escaped version of the contained
/// string when passed to a format string.
pub(crate) struct Escape<'a>(pub &'a str);

impl<'a> fmt::Display for Escape<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Because the internet is always right, turns out there's not that many
        // characters to escape: http://stackoverflow.com/questions/7381974
        let Escape(s) = *self;
        let pile_o_bits = s;
        let mut last = 0;
        for (i, ch) in s.char_indices() {
            let s = match ch {
                '>' => "&gt;",
                '<' => "&lt;",
                '&' => "&amp;",
                '\'' => "&#39;",
                '"' => "&quot;",
                '\n' => "<br/>",
                _ => continue,
            };
            fmt.write_str(&pile_o_bits[last..i])?;
            fmt.write_str(s)?;
            // NOTE: we only expect single byte characters here - which is fine as long as we
            // only match single byte characters
            last = i + 1;
        }

        if last < s.len() {
            fmt.write_str(&pile_o_bits[last..])?;
        }
        Ok(())
    }
}

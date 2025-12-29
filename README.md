# serialize ur egraphs

mostly for use in [extraction gym](https://github.com/egraphs-good/extraction-gym) rn

## snippet for egg

One day egg will natively export to this format, but for now you can use this:

```rust
pub fn egg_to_serialized_egraph<L, A>(egraph: &EGraph<L, A>) -> egraph_serialize::EGraph
where
    L: Language + Display,
    A: Analysis<L>,
{
    use egraph_serialize::*;
    let mut out = EGraph::default();
    for class in egraph.classes() {
        for (i, node) in class.nodes.iter().enumerate() {
            out.add_node(
                format!("{}.{}", class.id, i),
                Node {
                    op: node.to_string(),
                    children: node
                        .children()
                        .iter()
                        .map(|id| NodeId::from(format!("{}.0", id)))
                        .collect(),
                    eclass: ClassId::from(format!("{}", class.id)),
                    cost: Cost::new(1.0).unwrap(),
                },
            )
        }
    }
    out
}
```

Don't forget to add something to `root_eclasses` on the resulting serialized egraph!


## Visualization

Check out the [`./tests-viz`](./tests-viz/README.md) directory to view visualizations of all the test cases with Graphviz.

To remake them, run `make tests` from the root of this repo. 

### SVG Generation 
Generating SVG files requires having [Graphviz](https://graphviz.org/) installed. This cannot be distributed through rust's crates toolchain, so the function `to_svg_file` is gated behind the `graphviz-exec` feature flag. 

### DOT Generation
The `graphviz-rust` crate can produce `.dot` files without requiring the existence of the [Graphviz](https://graphviz.org/) CLI tool. Therefore creation of dot files and other functionality which only relies on `graphviz-rust` is gated behind the `graphviz` feature flag. 
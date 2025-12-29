use std::path::{Path, PathBuf};

use egraph_serialize::*;

#[cfg(feature = "serde")]
#[test]
fn test_round_trip() {
    let mut n_tested = 0;
    for entry in test_files() {
        println!("Testing {entry:?}");
        let egraph = EGraph::from_json_file(entry.as_path()).unwrap();
        egraph.test_round_trip();
        n_tested += 1;
    }
    assert!(n_tested > 0);
}

#[cfg(all(feature = "serde", feature = "graphviz"))]
#[test]
fn test_graphviz() {
    let mut names = Vec::new();
    for entry in test_files() {
        println!("Testing graphviz {entry:?}");
        let mut egraph = EGraph::from_json_file(entry.as_path()).unwrap();
        names.push(entry.file_stem().unwrap().to_str().unwrap().to_string());

        egraph.to_dot(); // Always test `to_dot`

        // Test `to_svg` only when the graphviz-exec feature is available
        #[cfg(feature = "graphviz-exec")]
        {
            let path = Path::new("./tests-viz")
                .join(entry.file_name().unwrap())
                .with_extension("svg");
            if path.exists() {
                println!("Skipping {path:?}");
            } else {
                println!("Writing to {path:?}");
                egraph.to_svg_file(path).unwrap();
            }
        }
        // Generate graphs with inlined leaves as well
        egraph.inline_leaves();

        egraph.to_dot();

        #[cfg(feature = "graphviz-exec")]
        {
            let path = Path::new("./tests-viz").join(format!(
                "{}-inlined.svg",
                entry.file_stem().unwrap().to_str().unwrap()
            ));
            if path.exists() {
                println!("Skipping {path:?}");
            } else {
                println!("Writing to {path:?}");
                egraph.to_svg_file(path).unwrap();
            }
        }

        // Saturate inlining
        egraph.saturate_inline_leaves();

        egraph.to_dot();

        #[cfg(feature = "graphviz-exec")]
        {
            let path = Path::new("./tests-viz").join(format!(
                "{}-inlined-saturated.svg",
                entry.file_stem().unwrap().to_str().unwrap()
            ));
            if path.exists() {
                println!("Skipping {path:?}");
            } else {
                println!("Writing to {path:?}");
                egraph.to_svg_file(path).unwrap();
            }
        }
    }

    let markdown = format!(
        r#"<!-- Auto generate from tests -->
# EGraph Visualization Tests

This is a list of all the tests in the `tests` directory. Each test is a JSON file that is loaded into an EGraph and then rendered as an SVG.

| Test | Image | Inlined Leaves Image | Inlined Leaves Saturated Image |
| ---- | ----- | -------------------- | -------------------------- |
{}"#,
        names
            .iter()
            .map(|name| {
                format!(
                    "| [`{name}`](../tests/{name}.json) | ![svg file](./{name}.svg) | ![inlined leaves svg file](./{name}-inlined.svg) | ![inlined leaves saturated svg file](./{name}-inlined-saturated.svg) |"
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    );
    std::fs::write("./tests-viz/README.md", markdown).unwrap();
}

fn test_files() -> Vec<PathBuf> {
    let mut test_files = Vec::new();
    for entry in glob::glob("tests/*.json").expect("Failed to read glob pattern") {
        test_files.push(entry.unwrap());
    }
    test_files
}

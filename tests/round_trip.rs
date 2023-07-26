use std::path::PathBuf;

use egraph_serialize::*;

#[test]
fn test_round_trip() {
    let mut n_tested = 0;
    for entry in test_files() {
        println!("Testing {:?}", entry);
        let egraph = EGraph::from_json_file(entry.as_path()).unwrap();
        egraph.test_round_trip();
        n_tested += 1;
    }
    assert!(n_tested > 0);
}

#[cfg(feature = "graphviz")]
#[test]
fn test_graphviz() {
    use std::path::Path;
    let mut names = Vec::new();
    for entry in test_files() {
        println!("Testing graphviz {:?}", entry);
        let egraph = EGraph::from_json_file(entry.as_path()).unwrap();
        let name = entry.file_stem().unwrap();
        names.push(name.to_str().unwrap().to_string());
        let path = Path::new("./tests-viz").join(name).with_extension("svg");
        egraph.to_svg_file(path).unwrap();
    }

    let markdown = format!(
        r#"<!-- Auto generate from tests --> # EGraph Visualization Tests

This is a list of all the tests in the `tests` directory. Each test is a JSON file that is loaded into an EGraph and then rendered as an SVG.

| Test | Image |
| ---- | ----- |
{}"#,
        names
            .iter()
            .map(|name| {
                format!(
                    "| [`{}`](../tests/{}.json) | ![svg file](./{}.svg) |",
                    name, name, name
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    );
    std::fs::write("./tests-viz/INDEX.md", markdown).unwrap();
}

fn test_files() -> Vec<PathBuf> {
    let mut test_files = Vec::new();
    for entry in glob::glob("tests/*.json").expect("Failed to read glob pattern") {
        test_files.push(entry.unwrap());
    }
    test_files
}

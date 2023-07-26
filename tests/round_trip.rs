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

    for entry in test_files() {
        println!("Testing graphviz {:?}", entry);
        let egraph = EGraph::from_json_file(entry.as_path()).unwrap();
        let path = Path::new("./tests-viz").join(entry.file_stem().unwrap()).with_extension("svg");
        egraph.to_svg_file(path).unwrap();
    }
}


fn test_files() -> Vec<PathBuf> {
    let mut test_files = Vec::new();
    for entry in glob::glob("tests/*.json").expect("Failed to read glob pattern") {
        test_files.push(entry.unwrap());
    }
    test_files
}

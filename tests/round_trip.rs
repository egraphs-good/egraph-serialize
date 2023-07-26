use egraph_serialize::*;

#[test]
fn test_round_trip() {
    let mut n_tested = 0;
    let pattern = "tests/*.json";
    for entry in glob::glob(pattern).expect("Failed to read glob pattern") {
        let entry = entry.unwrap();
        println!("Testing {:?}", entry);
        let egraph = EGraph::from_json_file(entry.as_path()).unwrap();
        egraph.test_round_trip();
        egraph.to_dot();
        n_tested += 1;
    }
    assert!(n_tested > 0);
}

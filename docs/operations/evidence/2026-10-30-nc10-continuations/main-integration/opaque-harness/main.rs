fn main() {
    let output = std::env::args().nth(1).expect("output JSON path");
    let result = nc10_opaque_planner_evidence::run_opaque_planner_matrix();
    std::fs::write(output, &result).unwrap();
    println!("{result}");
}

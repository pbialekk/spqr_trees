use petgraph::visit::EdgeRef;
use spqr_trees::from_file;

fn main() {
    let graph = from_file("assets/in1.graph");

    println!("Number of nodes: {}", graph.node_count());
    println!("Number of edges: {}", graph.edge_count());

    for edge in graph.edge_references() {
        println!(
            "Edge: {:?} <-> {:?} [{:?}]",
            graph.node_weight(edge.source()).unwrap(),
            graph.node_weight(edge.target()).unwrap(),
            edge.weight()
        );
    }
}

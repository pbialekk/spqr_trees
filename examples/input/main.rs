use petgraph::visit::EdgeRef;
use spqr_trees::input::from_file;

fn main() {
    let graph = from_file("assets/graph.in");

    println!("Number of nodes: {}", graph.node_count());
    println!("Number of edges: {}", graph.edge_count());

    for node in graph.node_indices() {
        println!("Node internal index: {}, node label: {}", node.index(), graph[node]);
    }

    for edge in graph.edge_references() {
        println!(
            "Edge: {:?} <-> {:?} [{:?}]",
            graph.node_weight(edge.source()).unwrap(),
            graph.node_weight(edge.target()).unwrap(),
            edge.weight()
        );
    }
}

/// Example of parsing a graph from string.
/// Number of nodes and edges is printed.
/// Then there is shown a relation between petgraph's internal indices and our labels.
/// Finally, all edges are printed with their labels.
/// [Real] label is just indicator that edge is real, not virtual.
use petgraph::visit::EdgeRef;
use spqr_trees::input::from_str;

fn main() {
    // here you can also use `from_file` with `assets/posch.in`, file input is more readable
    let graph = from_str("1,2\n2,3\n3,4\n4,5\n5,1\n1,3\n2,4\n");

    println!("Number of nodes: {}", graph.node_count());
    println!("Number of edges: {}", graph.edge_count());

    for node in graph.node_indices() {
        println!(
            "Node internal index: {}, node label: {}",
            node.index(),
            graph[node]
        );
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

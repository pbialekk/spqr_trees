use petgraph::graph::UnGraph;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};

/// Reads a graph from a file.
///
/// Connected undirected graph input:
/// - one line, one edge in format "u,v",
/// - by convetion start numbering from 0 and go up to |V|-1.
///
/// <div class="warning">
///
/// > Graph must be at least 1-connected.  
/// > Multi-edges and self-loops are not yet supported.  
/// > Petgraph will decide about IDs of nodes and edges. They may be not the same as you provided.
///
/// </div>
///
/// Example input:
/// ```text
/// 0,1
/// 1,2
/// 2,3
/// 2,4
/// 4,5
/// 5,6
/// 4,7
/// 7,8
/// 2,0
/// 3,0
/// 4,1
/// 6,2
/// 6,4
/// 8,1
/// ```
pub fn from_file(path: &str) -> UnGraph<u32, String> {
    let file = File::open(path).expect("File should exist and be readable");
    let reader = BufReader::new(file);
    parse_graph_from_custom_format(reader)
}

/// This is equivalent to [`from_file`], but takes string as an input.
pub fn from_str(input: &str) -> UnGraph<u32, String> {
    let cursor = Cursor::new(input);
    let reader = BufReader::new(cursor);
    parse_graph_from_custom_format(reader)
}

fn parse_graph_from_custom_format<R: BufRead>(reader: R) -> UnGraph<u32, String> {
    let mut edges = Vec::new();
    let mut max_node = 0;

    for line in reader.lines() {
        let line = line.expect("Line should be readable");
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<_> = line.split(',').collect();
        if parts.len() != 2 {
            panic!("Wrong format, expected 'u,v' for an edge");
        }
        let u: usize = parts[0].parse().expect("Node index should be a non-negative number");
        let v: usize = parts[1].parse().expect("Node index should be a non-negative number");
        max_node = max_node.max(u).max(v);
        edges.push((u, v));
    }

    let mut graph = UnGraph::<u32, String>::new_undirected();
    let nodes: Vec<_> = (0..=max_node).map(|i| graph.add_node(i as u32)).collect();

    graph.extend_with_edges(edges.iter().map(|&(u, v)| (nodes[u], nodes[v], String::from("REAL"))));

    graph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let input = "0,1\n1,2\n";
        let graph = from_str(input);
        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 2);
    }
}
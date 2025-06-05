use crate::EdgeLabel;
use crate::UnGraph;
use hashbrown::{HashMap, HashSet};
use petgraph::graph::NodeIndex;
use std::fs::File;
use std::io::{BufRead, BufReader, Cursor};


/// This is equivalent to [`from_str`], but takes file path as an input.
pub fn from_file(path: &str) -> UnGraph {
    let file = File::open(path).expect("File should exist and be readable");
    let reader = BufReader::new(file);
    parse_graph_from_custom_format(reader)
}

/// /// Reads a graph from a string.
///
/// Connected undirected graph input:
/// - one line, one edge in format "u,v",
/// - you can number vertices with non-negative integers,
/// numbers will be used only as labels in dot format,
/// for nodes indentification you should see petgraph's `NodeIndex`,
/// 
/// <div class="warning">
///
/// > Graph must be at least 1-connected.  
/// > Multi-edges and self-loops are not yet supported.  
///
/// </div>
///
/// Example input:
/// ```text
/// 1,2
/// 3,4
/// 3,4
/// 3,5
/// 5,6
/// 6,7
/// 5,8
/// 8,9
/// 3,1
/// 4,1
/// 5,2
/// 7,3
/// 7,5
/// 9,2
/// ```
pub fn from_str(input: &str) -> UnGraph {
    let cursor = Cursor::new(input);
    let reader = BufReader::new(cursor);
    parse_graph_from_custom_format(reader)
}

fn parse_graph_from_custom_format<R: BufRead>(reader: R) -> UnGraph {
    let mut edges = Vec::new();
    let mut node_ids = HashSet::<u32>::new();
    let mut ids_to_internal = HashMap::<u32, NodeIndex>::new();

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
        let u: u32 = parts[0]
            .parse()
            .expect("Node index should be a non-negative number");
        let v: u32 = parts[1]
            .parse()
            .expect("Node index should be a non-negative number");

        node_ids.insert(u);
        node_ids.insert(v);

        edges.push((u, v));
    }

    let mut graph = UnGraph::new_undirected();

    for &id in &node_ids {
        let internal_id = graph.add_node(id);
        ids_to_internal.insert(id, internal_id);
    }


    graph.extend_with_edges(
        edges
            .iter()
            .map(|&(u, v)| (ids_to_internal[&u], ids_to_internal[&v], EdgeLabel::Real)),
    );

    graph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let input = "1,2\n2,3\n";
        let graph = from_str(input);
        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 2);
    }
}

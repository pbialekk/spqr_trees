use dot::{Edges, GraphWalk, Labeller, Nodes};

use crate::triconnected::EdgeType;

type Node = usize;
type EdgeId = usize;

#[derive(Debug, Clone)]
struct Edge {
    id: EdgeId,
    id_internal: usize,
    source: Node,
    target: Node,
    edge_type: Option<EdgeType>,
    start_path: bool,
}

struct Graph<'a> {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    lowpt1: &'a [usize],
    lowpt2: &'a [usize],
    parent: &'a [Option<usize>],
    subsz: &'a [usize],
    num: &'a [usize],
    high: &'a [Vec<usize>],
}

impl<'a> Labeller<'a, Node, Edge> for Graph<'a> {
    fn graph_id(&self) -> dot::Id<'_> {
        dot::Id::new("G").unwrap()
    }

    fn node_id(&self, n: &Node) -> dot::Id<'_> {
        dot::Id::new(format!("N{}", n)).unwrap()
    }

    fn node_label(&self, n: &Node) -> dot::LabelText<'a> {
        dot::LabelText::label(format!(
            "{}\nnum:{}\nhigh:{:?}\nl1:{} l2:{}\np:{} sz:{}",
            n,
            self.num[*n],
            self.high[*n],
            self.lowpt1[*n],
            self.lowpt2[*n],
            if self.parent[*n].is_some() {
                self.parent[*n].unwrap().to_string()
            } else {
                "Root".to_string()
            },
            self.subsz[*n]
        ))
    }

    fn edge_label(&self, e: &Edge) -> dot::LabelText<'a> {
        let etype = match &e.edge_type {
            Some(t) => format!("{:?}", t),
            None => "None".to_string(),
        };
        dot::LabelText::label(format!(
            "{}({}) {} {}",
            e.id,
            e.id_internal,
            etype,
            if e.start_path { "start" } else { "" }
        ))
    }
}

impl<'a> GraphWalk<'a, Node, Edge> for Graph<'a> {
    fn nodes(&self) -> Nodes<'_, Node> {
        self.nodes.iter().cloned().collect()
    }

    fn edges(&self) -> Edges<'_, Edge> {
        self.edges.as_slice().into()
    }

    fn source(&self, e: &Edge) -> Node {
        e.source
    }

    fn target(&self, e: &Edge) -> Node {
        e.target
    }
}

pub fn draw(
    adj: &[Vec<usize>],
    edges: &[(usize, usize)],
    num: &[usize],
    high: &[Vec<usize>],
    edge_type: &[Option<EdgeType>],
    starts_path: &[bool],
    lowpt1: &[usize],
    lowpt2: &[usize],
    parent: &[Option<usize>],
    subsz: &[usize],
) -> String {
    let mut graph = Graph {
        nodes: (0..adj.len()).collect(),
        edges: Vec::new(),
        lowpt1,
        lowpt2,
        parent,
        subsz,
        num,
        high,
    };

    for (v, eids) in adj.iter().enumerate() {
        for (i, eid) in eids.iter().enumerate() {
            let (u, v) = edges[*eid];
            graph.edges.push(Edge {
                id: *eid,
                id_internal: i,
                source: u,
                target: v,
                edge_type: edge_type.get(*eid).cloned().unwrap_or(None),
                start_path: starts_path.get(*eid).cloned().unwrap_or(false),
            });
        }
    }

    let mut buffer = std::io::Cursor::new(Vec::new());
    dot::render(&graph, &mut buffer).unwrap();
    String::from_utf8(buffer.into_inner()).unwrap()
}

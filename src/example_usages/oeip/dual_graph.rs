use crate::testing::grids::Point;
use hashbrown::HashSet;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use crate::{EdgeLabel, UnGraph};

/// Represents a face in the dual graph of a planar graph.
#[derive(Debug, Clone)]
pub struct Face {
    /// Order of vertices (clockwise or counterclockwise for outer face)
    pub order: Vec<usize>,
    /// Indices of edges
    pub edges: HashSet<usize>,
    /// Indices of vertices
    pub vertices: HashSet<usize>,
}

impl Face {
    pub fn new() -> Self {
        Face {
            order: vec![],
            edges: HashSet::new(),
            vertices: HashSet::new(),
        }
    }
}

/// Represents the dual graph of a planar graph.
///
/// Each face is a vertex.
///
/// Vertices are connected if their faces share an edge in the original graph.
#[derive(Debug, Clone)]
pub struct DualGraph {
    /// Faces of the dual graph
    pub faces: Vec<Face>,
    /// Graph of faces
    pub graph: UnGraph,
    /// Index of outer face
    pub outer_face: usize,
}

/// Returns dual graph of given connected planar graph given locations of vertices.
///
/// Parameters:
/// - `points` - must allow for mapping from index to vertex point in the space (should be unique),
/// - `graph` - graph.
///
/// Based on (https://cp-algorithms.com/geometry/planar.html).
pub fn get_dual_graph(points: &[Point], graph: &UnGraph) -> DualGraph {
    let n = points.len();
    assert!(graph.edge_count() > 0); // no edges => algorithm fails

    let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
    for (i, e) in graph.edge_references().enumerate() {
        let (s, t) = (e.source().index(), e.target().index());
        adj[s].push(i);
        adj[t].push(i);
    }
    let mut used: Vec<Vec<bool>> = adj.iter().map(|v| vec![false; v.len()]).collect();
    let mut edge_to_face: Vec<Option<usize>> = vec![None; graph.edge_count()];

    fn get_other(i: usize, j: usize, graph: &UnGraph) -> usize{
        let e = graph.edge_references().nth(j).unwrap();
        let (s, t) = (e.source().index(), e.target().index());
        if s == i {
            t
        } else {
            s
        }
    }

    // sorting adjacency list for each vertex by polar angle
    for i in 0..n {
        let compare = |&el: &usize, &er: &usize| {
            let vl = get_other(i, el, graph);
            let vr = get_other(i, er, graph);
            let pl = points[vl].sub(&points[i]);
            let pr = points[vr].sub(&points[i]);
            match (pl.half(), pr.half()) {
                (hl, hr) if hl != hr => hl.cmp(&hr),
                _ => pr.cross(&pl).cmp(&0).reverse(),
            }
        };
        adj[i].sort_by(compare);
    }

    let mut faces = Vec::new();
    let mut edges_in_dual = HashSet::new();
    let mut outer_face = None;

    for i in 0..n {
        for j in 0..adj[i].len() {
            if used[i][j] {
                continue;
            }
            let mut face = Face::new();
            let (mut v, mut e) = (i, j);
            while !used[v][e] {
                used[v][e] = true;

                // each edge is traversed twice, once from each side
                // this fact is  used to build dual graph
                if let Some(face_id) = edge_to_face[adj[v][e]] {
                    edges_in_dual.insert((face_id, faces.len()));
                } else {
                    edge_to_face[adj[v][e]] = Some(faces.len());
                }
                face.order.push(v);
                face.edges.insert(adj[v][e]);
                face.vertices.insert(v);

                let ue = adj[v][e];
                let u = get_other(v, ue, graph);
                let compare = |&el: &usize, &er: &usize| {
                    let vl = get_other(u, el, graph);
                    let vr = get_other(u, er, graph);
                    let pl = points[vl].sub(&points[u]);
                    let pr = points[vr].sub(&points[u]);
                    match (pl.half(), pr.half()) {
                        (hl, hr) if hl != hr => hl.cmp(&hr),
                        _ => pr.cross(&pl).cmp(&0).reverse(),
                    }
                };
                let pos = adj[u] // next edge to traverse
                    .binary_search_by(|&x| compare(&x, &ue)).unwrap();
                let mut e1 = pos + 1;
                if e1 == adj[u].len() {
                    e1 = 0;
                }
                v = u;
                e = e1;
            }
            face.order.reverse();
            let p1 = points[face.order[0]];
            let mut sum: i128 = 0;
            for j in 0..face.order.len() {
                let p2 = points[face.order[j]];
                let p3 = points[face.order[(j + 1) % face.order.len()]];
                sum += p2.cross2(&p1, &p3) as i128;
            }
            if sum <= 0 {
                outer_face = Some(faces.len());
            }
            faces.push(face);
        }
    }

    let mut graph = UnGraph::new_undirected();
    for (i, _) in faces.iter().enumerate() {
        graph.add_node(i as u32);
    }

    for (i, j) in edges_in_dual {
        if i == j {
            continue; // degenerate case with outer face or not bijective mapping to points
        }
        graph.add_edge(NodeIndex::new(i), NodeIndex::new(j), EdgeLabel::Structure);
    }


    let dual_graph = DualGraph {
        faces,
        graph,
        outer_face: outer_face.unwrap(),
    };

    dual_graph
}

mod tests {
    use super::*;
    use crate::testing::grids::{get_arbitrary_embedding_of_grid, generate_grid_graph};
    use petgraph::algo::isomorphism::{is_isomorphic};

    fn get_iso_dual_graph_of_grid(rows: usize, cols: usize) -> UnGraph {
        assert!(rows > 2 && cols > 2);
        let mut dual_graph = generate_grid_graph(rows - 1, cols - 1);
        let outer = dual_graph.add_node(dual_graph.node_count() as u32);
        for r in 0..rows-1 {
            for c in 0..cols-1 {
                if r == 0 || r == rows - 2 || c == 0 || c == cols - 2 {
                    let node = NodeIndex::new(r * (cols - 1) + c);
                    dual_graph.add_edge(node, outer, EdgeLabel::Real);
                }
            }
        }

        dual_graph
    }

    #[test]
    fn test_dual_graph_edge() {
        let mut graph = UnGraph::new_undirected();
        let a = graph.add_node(0);
        let b = graph.add_node(1);
        graph.add_edge(a, b, EdgeLabel::Real);
        let points = vec![Point::new(0, 0), Point::new(1, 0)];
        let dual_graph = get_dual_graph(&points, &graph);
        assert_eq!(dual_graph.graph.node_count(), 1);
        assert_eq!(dual_graph.graph.edge_count(), 0);

    }

    #[test]
    fn test_dual_graph_triangle() {
        let mut graph = UnGraph::new_undirected();
        let a = graph.add_node(0);
        let b = graph.add_node(1);
        let c = graph.add_node(2);
        graph.add_edge(a, b, EdgeLabel::Real);
        graph.add_edge(b, c, EdgeLabel::Real);
        graph.add_edge(c, a, EdgeLabel::Real);
        let points = vec![Point::new(0, 0), Point::new(1, 0), Point::new(0, 1)];
        let dual_graph = get_dual_graph(&points, &graph);

        assert_eq!(dual_graph.graph.node_count(), 2);
        assert_eq!(dual_graph.graph.edge_count(), 1);
    }

    #[test]
    fn test_dual_graph_square() {
        let graph = generate_grid_graph(2, 2);
        let points = get_arbitrary_embedding_of_grid(2, 2);
        let dual_graph = get_dual_graph(&points, &graph);

        assert_eq!(dual_graph.graph.node_count(), 2);
        assert_eq!(dual_graph.graph.edge_count(), 1);
    }

    #[test]
    fn test_dual_graph_grids() {
        for rows in 3..20 {
            for cols in 3..20 {
                let graph = generate_grid_graph(rows, cols);
                let points = get_arbitrary_embedding_of_grid(rows, cols);
                let dual_graph = get_dual_graph(&points, &graph);
                let iso_dual_graph = get_iso_dual_graph_of_grid(rows, cols);
                assert!(is_isomorphic(&iso_dual_graph, &dual_graph.graph));
            }
        }
    }

    #[test]
    fn test_concave() {
        let mut graph = UnGraph::new_undirected();
        let a = graph.add_node(0);
        let b = graph.add_node(1);
        let c = graph.add_node(2);
        let d = graph.add_node(3);
        let e = graph.add_node(4);
        let f = graph.add_node(5);
        let g = graph.add_node(6);
        let h = graph.add_node(7);

        graph.add_edge(a, b, EdgeLabel::Real);
        graph.add_edge(b, c, EdgeLabel::Real);
        graph.add_edge(c, d, EdgeLabel::Real);
        graph.add_edge(d, e, EdgeLabel::Real);
        graph.add_edge(e, f, EdgeLabel::Real);
        graph.add_edge(f, g, EdgeLabel::Real);
        graph.add_edge(g, h, EdgeLabel::Real);
        graph.add_edge(h, a, EdgeLabel::Real);

        let points = vec![
            Point::new(0, 0), Point::new(1, 0), Point::new(2, 0),
            Point::new(2, -1), Point::new(2, -2), Point::new(1, -2),
            Point::new(1, -1), Point::new(0, -1)
        ];

        let dual_graph = get_dual_graph(&points, &graph);
        println!("{:?}", dual_graph.faces);
    }
}
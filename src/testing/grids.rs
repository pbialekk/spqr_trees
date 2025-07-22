use crate::{EdgeLabel, UnGraph};
use petgraph::graph::NodeIndex;

/// Generates a grid graph with the specified number of rows and columns.
#[allow(dead_code)]
pub fn generate_grid_graph(rows: usize, cols: usize) -> UnGraph {
    assert!(rows > 1 && cols > 1); // we want biconnected graph
    let mut graph = UnGraph::new_undirected();

    for r in 0..rows {
        for c in 0..cols {
            graph.add_node((r * cols + c) as u32);
        }
    }

    for r in 0..rows {
        for c in 0..cols {
            if r + 1 < rows {
                graph.add_edge(NodeIndex::new(r * cols + c), NodeIndex::new((r + 1) * cols + c), EdgeLabel::Real);
            }
            if c + 1 < cols {
                graph.add_edge(NodeIndex::new(r * cols + c), NodeIndex::new(r * cols + c + 1), EdgeLabel::Real);
            }
        }
    }

    graph
}

#[derive(Clone, Copy, Debug)]
#[derive(PartialEq)]
pub struct Point {
    x: i64,
    y: i64,
}

impl Point {
    pub fn new(x: i64, y: i64) -> Self {
        Point { x, y }
    }

    pub fn sub(&self, p: &Point) -> Point {
        Point::new(self.x - p.x, self.y - p.y)
    }

    pub fn cross(&self, p: &Point) -> i64 {
        self.x * p.y - self.y * p.x
    }

    pub fn cross2(&self, p: &Point, q: &Point) -> i64 {
        p.sub(self).cross(&q.sub(self))
    }

    pub fn half(&self) -> bool {
        self.y < 0 || (self.y == 0 && self.x < 0)
    }
}

#[allow(dead_code)]
pub fn get_arbitrary_embedding_of_grid(rows: usize, cols: usize) -> Vec<Point> {
    let mut points = vec![Point { x: 0, y: 0 }; rows * cols];
    for r in 0..rows {
        for c in 0..cols {
            points[r * cols + c] = Point { x: c as i64, y: -(r as i64) };
        }
    }

    points
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_square_embedding() {
        let emb = get_arbitrary_embedding_of_grid(2,2);
        assert_eq!(emb, vec![Point { x: 0, y: 0 }, Point { x: 1, y: 0 }, Point { x: 0, y: -1 }, Point { x: 1, y: -1 }]);

    }
}

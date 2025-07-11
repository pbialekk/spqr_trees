use crate::UnGraph;
use petgraph::visit::{EdgeRef, IntoNodeReferences, NodeIndexable};

/// Based on https://www.sciencedirect.com/science/article/pii/S1570866708000415
fn dfs(
    graph: &Vec<Vec<usize>>,
    edge_list: &Vec<(usize, usize)>,
    u: usize,
    time: &mut usize,
    parent: Option<usize>,
    preorder: &mut [usize],
    split_pairs: &mut Vec<(usize, usize)>,
    subsz: &mut Vec<usize>,
) -> (usize, usize, Vec<(usize, usize, bool, usize, usize)>) {
    *time += 1;
    preorder[u] = *time;
    let mut low = *time;
    let mut low2 = *time;

    let mut lowv = u;
    let mut low2v = u;

    let mut stack = Vec::new();

    let mut to_low = usize::MAX;
    let mut to_lowv = usize::MAX;

    let mut tree_edges = Vec::new();

    for &eid in &graph[u] {
        let v = edge_list[eid].0 ^ edge_list[eid].1 ^ u;
        if Some(eid) == parent {
            continue;
        }
        if preorder[v] == usize::MAX {
            let (v_low, v_lowv, mut v_stack) = dfs(
                graph,
                edge_list,
                v,
                time,
                Some(eid),
                preorder,
                split_pairs,
                subsz,
            );
            tree_edges.push(eid);

            subsz[u] += subsz[v];
            if preorder[u] < v_low {
                // a bridge
                split_pairs.push((eid, eid));
                continue;
            }

            if let Some(&(i, y, b, p, _)) = v_stack.last() {
                if v_stack.last().unwrap().4 == v {
                    v_stack.pop();
                    split_pairs.push((eid, i));
                    if p != u {
                        v_stack.push((i, y, b, p, u));
                    }
                }
            }

            if v_low < low {
                low2 = low;
                low2v = lowv;
                low = v_low;
                lowv = v_lowv;
                stack = v_stack;
                to_low = eid;
                to_lowv = v;
            } else if v_low < low2 {
                low2 = v_low;
                low2v = v_lowv;
            }
        } else if preorder[v] < preorder[u] {
            if preorder[v] <= low {
                low2 = low;
                low2v = lowv;
                low = preorder[v];
                lowv = v;
                stack.clear();
                to_low = eid;
                to_lowv = v;
            } else if preorder[v] < low2 {
                low2 = preorder[v];
                low2v = v;
            }
        }
    }

    if stack.is_empty() {
        if low < low2 {
            stack.push((
                to_low,
                to_lowv,
                preorder[to_lowv] < preorder[u],
                lowv,
                low2v,
            ));
        }
    } else {
        let (i, y, b, p, q) = *stack.last().unwrap();
        if preorder[q] < low2 {
            stack.push((to_low, to_lowv, preorder[to_lowv] < preorder[u], q, low2v));
        } else {
            while stack.last().map_or(false, |x| low2 <= preorder[x.3]) {
                stack.pop();
            }
            if let Some(&(i, y, b, p, q)) = stack.last() {
                if low2 < preorder[q] {
                    stack.pop();
                    stack.push((i, y, b, p, low2v));
                }
            }
        }
    }

    let mut k = 0;
    for &eid in &graph[u] {
        let v = edge_list[eid].0 ^ edge_list[eid].1 ^ u;
        if preorder[v] <= preorder[u] {
            continue;
        }
        if k < tree_edges.len() && tree_edges[k] == eid {
            k += 1;
            continue;
        }
        while stack.last().map_or(false, |x| {
            !x.2 && preorder[x.1] <= preorder[v] && preorder[v] < preorder[x.1] + subsz[x.1]
        }) {
            stack.pop();
        }
    }

    (low, lowv, stack)
}

/// Given a graph, returns a list of indices of pairs of edges that are it's split pairs. Not all pairs are listed, only those that are enough to properly construct the tri-edge-connected components.
pub fn get_edge_split_pairs(
    graph: &Vec<Vec<usize>>,
    edge_list: &Vec<(usize, usize)>,
) -> Vec<(usize, usize)> {
    let graph_size = graph.len();
    let mut split_pairs = Vec::new();
    let mut time = 0;
    let mut preorder = vec![usize::MAX; graph_size];
    let mut subsz = vec![1; graph_size];

    for u in 0..graph_size {
        if preorder[u] == usize::MAX {
            dfs(
                graph,
                edge_list,
                u,
                &mut time,
                None,
                &mut preorder,
                &mut split_pairs,
                &mut subsz,
            );
        }
    }

    split_pairs
}

#[cfg(test)]
mod tests {
    // https://judge.yosupo.jp/submission/296156
}

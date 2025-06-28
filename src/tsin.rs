use crate::{EdgeLabel, UnGraph};
use petgraph::visit::{EdgeRef, IntoNodeReferences, NodeIndexable};

fn dfs(
    graph: &UnGraph,
    u: usize,
    time: &mut usize,
    parent: Option<usize>,
    preorder: &mut [usize],
    split_pairs: &mut Vec<(usize, usize)>,
    subsz: &mut Vec<usize>,
    jump: &mut [usize],
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

    for (v, eid) in graph
        .edges_directed(graph.from_index(u), petgraph::Direction::Outgoing)
        .map(|e| (e.target().index(), e.id().index()))
    {
        if Some(eid) == parent {
            continue;
        }
        if preorder[v] == usize::MAX {
            let (v_low, v_lowv, mut v_stack) = dfs(
                graph,
                v,
                time,
                Some(eid),
                preorder,
                split_pairs,
                subsz,
                jump,
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
                    split_pairs.push((i, eid));
                    jump[y] = u; // for testing purposes
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
    for (v, eid) in graph
        .edges_directed(graph.from_index(u), petgraph::Direction::Outgoing)
        .map(|e| (e.target().index(), e.id().index()))
    {
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

/// Based on https://www.sciencedirect.com/science/article/pii/S1570866708000415
pub fn get_split_pairs(graph: &UnGraph) -> (Vec<(usize, usize)>, Vec<usize>) {
    let graph_size = graph.node_references().size_hint().0;
    let mut split_pairs = Vec::new();
    let mut time = 0;
    let mut preorder = vec![usize::MAX; graph_size];
    let mut subsz = vec![1; graph_size];
    let mut jump = vec![usize::MAX; graph_size];

    for u in graph.node_references().map(|(n, _)| n.index()) {
        if preorder[u] == usize::MAX {
            dfs(
                graph,
                u,
                &mut time,
                None,
                &mut preorder,
                &mut split_pairs,
                &mut subsz,
                &mut jump,
            );
        }
    }

    (split_pairs, jump)
}

#[cfg(test)]
mod tests {
    // https://judge.yosupo.jp/submission/296156
}

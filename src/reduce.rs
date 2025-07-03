/// Reference: https://dl.acm.org/doi/pdf/10.5555/1862776.1862783
use crate::{UnGraph, tsin::get_edge_split_pairs};
use petgraph::visit::{EdgeRef, IntoNodeReferences, NodeIndexable};
use radsort::sort_by_key;

fn new_vertex(
    graph: &mut Vec<Vec<usize>>,
    preorder: &mut Vec<usize>,
    low1: &mut Vec<usize>,
) -> usize {
    let new_index = graph.len();
    graph.push(Vec::new());
    preorder.push(usize::MAX);
    low1.push(usize::MAX);
    new_index
}

fn add_edge(
    edge_list: &mut Vec<(usize, usize)>,
    is_tree_edge: &mut Vec<bool>,
    graph: &mut Vec<Vec<usize>>,
    u: usize,
    v: usize,
    is_tree: bool,
) -> usize {
    edge_list.push((u, v));
    is_tree_edge.push(is_tree);
    let eid = edge_list.len() - 1;
    graph[u].push(eid);
    graph[v].push(eid);
    eid
}

fn move_edges(
    u: usize,
    u_fake: usize,
    edge_list: &mut Vec<(usize, usize)>,
    is_tree_edge: &mut Vec<bool>,
    graph: &mut Vec<Vec<usize>>,
    preorder: &mut Vec<usize>,
) {
    let mut i = 0;
    while i < graph[u].len() {
        let eid = graph[u][i];
        let v = edge_list[eid].0 ^ edge_list[eid].1 ^ u;
        if preorder[v] < preorder[u] && !is_tree_edge[eid] {
            // a back edge, move to u_fake
            if u == edge_list[eid].0 {
                edge_list[eid].0 = u_fake;
            } else {
                edge_list[eid].1 = u_fake;
            }
            graph[u_fake].push(eid);
            graph[u].swap_remove(i);
        } else {
            i += 1;
        }
    }
}

fn reduce_vertex(
    edge_list: &mut Vec<(usize, usize)>,
    is_tree_edge: &mut Vec<bool>,
    graph: &mut Vec<Vec<usize>>,
    u: usize,
    parent: Option<usize>,
    preorder: &mut Vec<usize>,
    low1: &mut Vec<usize>,
) {
    sort_by_key(&mut graph[u], |&eid| {
        let v = edge_list[eid].0 ^ edge_list[eid].1 ^ u;
        -(preorder[u] as isize)
    });
    // Let a_1, ..., a_k be the sequence of nodes such that \exists (u, u_i) \in non-tree-edges. (sorted by preorder)
    // Let b_1, ..., b_k be the sequence of nodes such that \exists (u_i, u) \in non-tree-edges. (...)
    // We replace u with a tree-path fake(a_k) -- ... -- fake(a_1) -- fake(b_k) -- ... -- fake(b_1).
    let mut a = Vec::new();
    let mut b = Vec::new();
    let mut children = Vec::new();
    for &eid in &graph[u] {
        if is_tree_edge[eid] {
            children.push(eid);
            continue;
        }
        let v = edge_list[eid].0 ^ edge_list[eid].1 ^ u;
        if preorder[v] < preorder[u] {
            a.push(eid);
        } else {
            b.push(eid);
        }
    }

    if a.len() + b.len() <= 1 {
        // no need to reduce
        return;
    }

    let mut first_created = usize::MAX;
    let mut last_created = usize::MAX;

    for &eid in a.iter().chain(b.iter()) {
        let fake = new_vertex(graph, preorder, low1);

        if last_created != usize::MAX {
            add_edge(edge_list, is_tree_edge, graph, last_created, fake, true);
            graph[last_created].push(edge_list.len() - 1);
            graph[fake].push(edge_list.len() - 1);
        }

        if first_created == usize::MAX {
            first_created = fake;
        }
        last_created = fake;

        if u == edge_list[eid].0 {
            edge_list[eid].0 = fake;
        } else {
            edge_list[eid].1 = fake;
        }
        graph[fake].push(eid);

        low1[fake] = low1[u];
        preorder[fake] = preorder[u];
    }

    if let Some(p) = parent {
        let parent_v = edge_list[p].0 ^ edge_list[p].1 ^ first_created;
        add_edge(
            edge_list,
            is_tree_edge,
            graph,
            parent_v,
            first_created,
            true,
        );
        graph[parent_v].push(edge_list.len() - 1);
        graph[first_created].push(edge_list.len() - 1);
    }
    for eid in children {
        if edge_list[eid].0 == u {
            edge_list[eid].0 = last_created;
        } else {
            edge_list[eid].1 = last_created;
        }
        graph[last_created].push(eid);
    }

    graph[u].clear();
}

fn dfs(
    edge_list: &mut Vec<(usize, usize)>,
    is_tree_edge: &mut Vec<bool>,
    graph: &mut Vec<Vec<usize>>,
    root: usize,
    u: usize,
    time: &mut usize,
    parent: Option<usize>,
    parent_v: Option<usize>,
    preorder: &mut Vec<usize>,
    preorder_to_vertex: &mut Vec<usize>,
    low1: &mut Vec<usize>,
    low1_realizer: &mut Vec<usize>,
    low2: &mut Vec<usize>,
    split_pairs: &mut Vec<(usize, usize)>,
    subsz: &mut Vec<usize>,
    par: &mut Vec<usize>,
) {
    preorder[u] = *time;
    low1[u] = *time;
    low2[u] = *time;
    preorder_to_vertex[*time] = u;
    *time += 1;
    low1_realizer[u] = u;
    subsz[u] = 1;

    let mut min_child_low = (usize::MAX, usize::MAX); // (low1, low1_realizer)

    let edge_ids: Vec<usize> = graph[u].clone(); // borrow checker workaround
    for &eid in &edge_ids {
        let v = edge_list[eid].0 ^ edge_list[eid].1 ^ u;

        if Some(eid) == parent {
            continue;
        }
        if preorder[v] == usize::MAX {
            is_tree_edge[eid] = true;
            par[v] = u;
            dfs(
                edge_list,
                is_tree_edge,
                graph,
                root,
                v,
                time,
                Some(eid),
                Some(u),
                preorder,
                preorder_to_vertex,
                low1,
                low1_realizer,
                low2,
                split_pairs,
                subsz,
                par,
            );
            subsz[u] += subsz[v];
            min_child_low = min_child_low.min((low1[v], low1_realizer[v]));

            if low1[v] < low1[u] {
                low2[u] = low1[u];
                low1[u] = low1[v];
                low1_realizer[u] = low1_realizer[v];
            } else if low1[v] < low2[u] {
                low2[u] = low1[v];
            }
        } else if preorder[v] < preorder[u] {
            // a back edge
            if preorder[v] < low1[u] {
                low2[u] = low1[u];
                low1[u] = preorder[v];
                low1_realizer[u] = v;
            } else if preorder[v] < low2[u] {
                low2[u] = preorder[v];
            }
        }
    }

    let x = low1_realizer[u];

    if let Some(parent_v_idx) = parent_v {
        if x != parent_v_idx
            && low2[u] >= preorder[parent_v_idx]
            && (low1[u] != preorder[root] || par[parent_v_idx] != root)
        {
            split_pairs.push((x.min(parent_v_idx), x.max(parent_v_idx)));
        }
    }

    if min_child_low.0 != usize::MAX && min_child_low.0 != low1[u] {
        // move the back-edges of u to a newly created child
        let u_fake = new_vertex(graph, preorder, low1);
        preorder[u_fake] = preorder[u];
        low1[u_fake] = low1[u];
        move_edges(u, u_fake, edge_list, is_tree_edge, graph, preorder);

        add_edge(edge_list, is_tree_edge, graph, u, u_fake, true);

        reduce_vertex(
            edge_list,
            is_tree_edge,
            graph,
            u_fake,
            parent,
            preorder,
            low1,
        );
    }

    reduce_vertex(edge_list, is_tree_edge, graph, u, parent, preorder, low1);
}

/// Input: a biconnected graph
/// Output: Type-A separation pairs and a reduced graph on which we'll run tsin's tri-edge-connectivity algorithm
fn reduce(
    in_graph: &UnGraph,
) -> (
    Vec<Vec<usize>>,
    Vec<(usize, usize)>,
    Vec<(usize, usize)>,
    Vec<bool>,
    Vec<usize>,
    Vec<usize>,
    Vec<usize>,
    Vec<usize>,
    Vec<usize>,
) {
    let graph_size = in_graph.node_references().size_hint().0;
    let edge_count = in_graph.edge_references().size_hint().0;

    let mut edge_list: Vec<(usize, usize)> = Vec::new();
    let mut graph: Vec<Vec<usize>> = vec![Vec::new(); graph_size];
    for (u, v) in in_graph
        .edge_references()
        .map(|e| (e.source().index(), e.target().index()))
    {
        edge_list.push((u, v));
        graph[u].push(edge_list.len() - 1);
        graph[v].push(edge_list.len() - 1);
    }

    let mut is_tree_edge = vec![false; edge_count];
    let mut split_pairs = Vec::new(); // only type-A
    let mut time = 1;
    let mut preorder = vec![usize::MAX; graph_size];
    let mut preorder_to_vertex = vec![usize::MAX; graph_size + 1];
    let mut low1 = vec![usize::MAX; graph_size];
    let mut low1_realizer = vec![usize::MAX; graph_size];
    let mut low2 = vec![usize::MAX; graph_size];
    let mut subsz = vec![0; graph_size]; // size of the subtree rooted at each vertex
    let mut par = vec![usize::MAX; graph_size];

    for u in 0..graph_size {
        if preorder[u] == usize::MAX {
            dfs(
                &mut edge_list,
                &mut is_tree_edge,
                &mut graph,
                u,
                u,
                &mut time,
                None,
                None,
                &mut preorder,
                &mut preorder_to_vertex,
                &mut low1,
                &mut low1_realizer,
                &mut low2,
                &mut split_pairs,
                &mut subsz,
                &mut par,
            );
        }
    }

    (
        graph,
        edge_list,
        split_pairs,
        is_tree_edge,
        low1,
        preorder,
        preorder_to_vertex,
        subsz,
        par,
    )
}

pub fn get_vertex_split_pairs(in_graph: UnGraph) -> Vec<(usize, usize)> {
    let (
        graph,
        edge_list,
        split_pairs,
        is_tree_edge,
        low1,
        preorder,
        preorder_to_vertex,
        subsz,
        par,
    ) = reduce(&in_graph);

    let mut result = split_pairs;
    for (u, v) in get_edge_split_pairs(&graph, &edge_list) {
        if !is_tree_edge[u] || !is_tree_edge[v] {
            continue;
        }
        dbg!((u, v));

        let u_owner = vec![
            preorder_to_vertex[preorder[edge_list[u].0]],
            preorder_to_vertex[preorder[edge_list[u].1]],
        ];
        let v_owner = vec![
            preorder_to_vertex[preorder[edge_list[v].0]],
            preorder_to_vertex[preorder[edge_list[v].1]],
        ];

        for &x in &u_owner {
            for &y in &v_owner {
                if x != y
                    && preorder[x] <= preorder[y]
                    && preorder[y] < preorder[x] + subsz[x]
                    && !(low1[x] == preorder[x] && subsz[y] == 1)
                {
                    // only if x is an ancestor of y and (x, y) is not a root-leaf pair

                    // TODO: make it faster
                    let mut x_son = y;
                    while par[x_son] != x {
                        x_son = par[x_son];
                    }

                    let mut v = (usize::MAX, usize::MAX);
                    for y_son in in_graph
                        .neighbors_directed(in_graph.from_index(y), petgraph::Direction::Outgoing)
                        .map(|n| n.index())
                    {
                        if par[y_son] == y {
                            v = v.min((low1[y_son], y_son));
                        }
                    }

                    let chosen = v.1;

                    let v3 = if chosen == usize::MAX {
                        0
                    } else {
                        subsz[chosen]
                    };
                    let mut v2 = subsz[x_son] - v3;
                    for y_son in in_graph
                        .neighbors_directed(in_graph.from_index(y), petgraph::Direction::Outgoing)
                        .map(|n| n.index())
                    {
                        if par[y_son] == y && low1[y_son] < preorder[x] && y_son != chosen {
                            v2 -= subsz[y_son];
                        }
                    }
                    v2 -= 1;

                    if v2 > 0 {
                        result.push((x.min(y), x.max(y)));
                    }
                }
            }
        }
    }

    result.sort();
    result.dedup();

    dbg!(&result);

    result
}

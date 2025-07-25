use embed_doc_image::embed_doc_image;
use petgraph::visit::EdgeRef;

use crate::{
    UnGraph,
    block_cut::get_block_cut_tree,
    triconnected_blocks::{
        acceptable_adj::make_adjacency_lists_acceptable,
        graph_internal::GraphInternal,
        handle_duplicate_edges::handle_duplicate_edges,
        merge_components::merge_components,
        outside_structures::{Component, ComponentType, EdgeType, TriconnectedComponents},
        palm_dfs::run_palm_dfs,
        pathfinder::run_pathfinder,
    },
};
use std::collections::HashMap;

fn find_components(
    root: usize,
    u: usize,
    vedges_cutoff: usize,
    graph: &mut GraphInternal,
    estack: &mut Vec<usize>,
    tstack: &mut Vec<(usize, usize, usize)>,
    split_components: &mut Vec<Component>,
) {
    fn update_tstack(
        u: usize,
        to: usize,
        eid: usize,
        tstack: &mut Vec<(usize, usize, usize)>,
        graph: &GraphInternal,
    ) {
        fn pop_tstack(
            cutoff: usize,
            mut max_h: usize,
            mut last_b: usize,
            tstack: &mut Vec<(usize, usize, usize)>,
        ) -> (usize, usize, usize) {
            while let Some(&(h, a, b)) = tstack.last() {
                if a > cutoff {
                    tstack.pop();
                    max_h = h.max(max_h);
                    last_b = b;
                } else {
                    break;
                }
            }

            (max_h, cutoff, last_b)
        }

        let (max_h, a, last_b) = if graph.edge_type[eid] == Some(EdgeType::Tree) {
            pop_tstack(
                graph.low1[to],
                graph.num[to] + graph.sub[to] - 1,
                graph.num[u],
                tstack,
            )
        } else {
            pop_tstack(graph.num[to], graph.num[u], graph.num[u], tstack)
        };

        tstack.push((max_h, a, last_b));
    }

    fn check_highpoint(
        u: usize,
        tstack: &mut Vec<(usize, usize, usize)>,
        graph: &mut GraphInternal,
    ) {
        let u_high = graph.get_high(u);

        while let Some(&(h, a, b)) = tstack.last() {
            if a != graph.num[u] && b != graph.num[u] && u_high > h {
                tstack.pop();
            } else {
                break;
            }
        }
    }

    fn check_type_2(
        root: usize,
        u: usize,
        mut to: usize,
        tstack: &mut Vec<(usize, usize, usize)>,
        estack: &mut Vec<usize>,
        graph: &mut GraphInternal,
        split_components: &mut Vec<Component>,
    ) {
        if graph.num[u] == root {
            return;
        }

        loop {
            let (h, a, b) = if let Some(&last) = tstack.last() {
                last
            } else {
                (0, usize::MAX, 0)
            };

            let cond_1 = a == graph.num[u];
            let cond_2 = graph.deg[to] == 2
                && graph.num[graph.first_alive(root, to).unwrap()] > graph.num[to];

            if !(cond_1 || cond_2) {
                break;
            }
            if a == graph.num[u] && graph.par[graph.numrev[b]] == Some(u) {
                tstack.pop();
                continue;
            }

            let mut eab = None;
            let mut evirt;
            if cond_2 {
                to = graph.first_alive(root, to).unwrap();

                let mut component = Component::new(ComponentType::S);

                for _ in 0..2 {
                    let eid = estack.pop().unwrap();
                    component.push_edge(eid, graph, false);
                }

                evirt = graph.new_edge(u, to, None);
                component.push_edge(evirt, graph, true);

                component.commit(split_components);

                if let Some(&eid) = estack.last() {
                    if graph.edges[eid] == (to, u) {
                        estack.pop();
                        eab = Some(eid);
                    }
                }
            } else {
                to = graph.numrev[b];

                tstack.pop();
                let mut component = Component::new(ComponentType::UNSURE);
                loop {
                    if let Some(&eid) = estack.last() {
                        let (x, y) = graph.edges[eid];

                        let x_in_subtree = graph.num[u] <= graph.num[x] && graph.num[x] <= h;
                        let y_in_subtree = graph.num[u] <= graph.num[y] && graph.num[y] <= h;
                        if !(x_in_subtree && y_in_subtree) {
                            break;
                        }

                        estack.pop();

                        if x == u && y == to || y == u && x == to {
                            eab = Some(eid);
                        } else {
                            component.push_edge(eid, graph, false);
                        }
                    } else {
                        break;
                    }
                }

                evirt = graph.new_edge(u, to, None);
                component.push_edge(evirt, graph, true);
                component.commit(split_components);
            }

            if let Some(eab) = eab {
                let mut component = Component::new(ComponentType::P);
                component.push_edge(eab, graph, false);

                component.push_edge(evirt, graph, false); // is an old vedge

                evirt = graph.new_edge(u, to, None);
                component.push_edge(evirt, graph, true);

                component.commit(split_components);
            }

            estack.push(evirt);
            graph.make_tedge(evirt);
        }
    }

    fn check_type_1(
        root: usize,
        u: usize,
        to: usize,
        estack: &mut Vec<usize>,
        graph: &mut GraphInternal,
        split_components: &mut Vec<Component>,
        t_edges_left: usize,
    ) {
        if graph.low2[to] >= graph.num[u]
            && graph.low1[to] < graph.num[u]
            && (Some(root) != graph.par[u] || t_edges_left != 0)
        {
            let mut component = Component::new(ComponentType::UNSURE);
            while let Some(&eid) = estack.last() {
                let (x, y) = graph.edges[eid];
                let x_in_subtree =
                    graph.num[to] <= graph.num[x] && graph.num[x] < graph.num[to] + graph.sub[to];
                let y_in_subtree =
                    graph.num[to] <= graph.num[y] && graph.num[y] < graph.num[to] + graph.sub[to];

                if !(x_in_subtree || y_in_subtree) {
                    break;
                }

                estack.pop();

                component.push_edge(eid, graph, true);
                graph.remove_edge(eid);
            }

            let mut evirt = graph.new_edge(u, graph.numrev[graph.low1[to]], None);
            component.push_edge(evirt, graph, true);

            component.commit(split_components);

            if let Some(&eid) = estack.last() {
                let (x, y) = graph.edges[eid];
                if (x == u && y == graph.numrev[graph.low1[to]])
                    || (y == u && x == graph.numrev[graph.low1[to]])
                {
                    estack.pop();
                    let mut component = Component::new(ComponentType::P);

                    component.push_edge(eid, graph, false);

                    component.push_edge(evirt, graph, false); // is an old vedge

                    evirt = graph.new_edge(u, graph.numrev[graph.low1[to]], None);
                    component.push_edge(evirt, graph, true);

                    component.commit(split_components);
                }
            }

            if Some(graph.numrev[graph.low1[to]]) != graph.par[u] {
                estack.push(evirt);

                graph.make_bedge(evirt);
            } else {
                let parent_edge = graph.par_edge[u].unwrap();

                let mut component = Component::new(ComponentType::P);

                component.push_edge(parent_edge, graph, false);

                component.push_edge(evirt, graph, false); // is an old vedge

                evirt = graph.new_edge(graph.par[u].unwrap(), u, None);
                component.push_edge(evirt, graph, true);

                component.commit(split_components);

                graph.make_tedge(evirt);
                graph.par_edge[u] = Some(evirt);
            }
        }
    }

    let mut adjacent_tedges = graph.adj[u]
        .iter()
        .filter(|&eid| graph.edge_type[*eid] == Some(EdgeType::Tree))
        .count();

    let mut i = 0;
    while i < graph.adj[u].len() {
        let eid = graph.adj[u][i];
        if eid >= vedges_cutoff {
            // we don't care about virtual edges here
            break;
        }

        let to = graph.get_other_vertex(eid, u);
        if graph.starts_path[eid] {
            update_tstack(u, to, eid, tstack, graph);
        }

        if graph.edge_type[eid] == Some(EdgeType::Tree) {
            let mut new_tstack = vec![];
            find_components(
                root,
                to,
                vedges_cutoff,
                graph,
                estack,
                if graph.starts_path[eid] {
                    &mut new_tstack
                } else {
                    tstack
                },
                split_components,
            );
            adjacent_tedges -= 1;

            let push_eid = graph.par_edge[to].unwrap(); // eid could be killed by the multiple edge case in check_type_x
            estack.push(push_eid);

            check_type_2(
                root,
                u,
                to,
                if graph.starts_path[eid] {
                    &mut new_tstack
                } else {
                    tstack
                },
                estack,
                graph,
                split_components,
            );
            check_type_1(
                root,
                u,
                to,
                estack,
                graph,
                split_components,
                adjacent_tedges,
            );

            check_highpoint(u, tstack, graph);
        } else {
            estack.push(eid);
        }

        i += 1;
    }
}

/// Computes the split components (triconnected components) of a biconnected, loopless undirected graph.
///
/// # Overview
///
/// Given a biconnected graph `G`, this function finds its split components, also known as triconnected components.
/// The algorithm assumes that the input graph is biconnected and contains no self-loops.
///
/// ## Split-Pair Definition
/// A pair of vertices `(s, t)` is called a *split-pair* if:
/// - Removing both `s` and `t` disconnects the graph, **or**
/// - There are multiple edges directly connecting `s` and `t`.
///
/// When a split-pair `(s, t)` is found, the graph is split into components by removing `s` and `t`.
/// For each resulting component, a new *virtual* edge `(s, t)` is added to maintain biconnectivity.
/// This allows the components to be merged later by "gluing" them together via the virtual edge (dotted one in the visualization).
///
/// ## Component Types
/// After recursively splitting on all split-pairs, the resulting components are of three types:
/// - **P node**: Exactly two vertices with at exactly three edges between them.
/// - **S node**: Exactly three vertices with exactly three edges (a triangle).
/// - **R node**: A triconnected component (cannot be split further).
///
/// After merging all P nodes with P nodes and S nodes with S nodes, the final set of triconnected components is obtained.
///
/// ## Example (visualized using .dot file generated with visualize.rs from triconnected_blocks)
///
/// ![TRICON_Full][tricon_full]
///
/// ## Reference
/// - [Hopcroft, J., & Tarjan, R. (1973). Dividing a Graph into Triconnected Components. SIAM Journal on Computing, 2(3), 135–158.](https://epubs.siam.org/doi/10.1137/0202012)
/// - [Explaining Hopcroft, Tarjan, Gutwenger, and Mutzel’s SPQR Decomposition Algorithm] (https://shoyamanishi.github.io/wailea/docs/spqr_explained/HTGMExplained.pdf)
#[embed_doc_image("tricon_full", "assets/split_components.svg")]
pub fn get_triconnected_components(in_graph: &UnGraph) -> TriconnectedComponents {
    let n = in_graph.node_count();
    let m = in_graph.edge_count();
    let root = 0;

    let mut split_components = Vec::new();

    assert!(get_block_cut_tree(&in_graph).block_count == 1);
    assert!(n >= 2);

    if n == 2 {
        let mut c = Component::new(ComponentType::P);
        let mut edges = Vec::new();
        for i in in_graph.edge_references() {
            let (s, t) = (i.source().index(), i.target().index());
            edges.push((s, t));
            c.push_edge(i.id().index(), &mut GraphInternal::new(0, 0), true);
        }

        if m >= 3 {
            return TriconnectedComponents {
                comp: vec![c],
                edges,
                is_real: vec![true; m],
                to_split: vec![Some(0); m],
            };
        } else {
            return TriconnectedComponents {
                comp: vec![],
                edges,
                is_real: vec![true; m],
                to_split: vec![Some(0); m],
            };
        }
    }

    let mut graph = GraphInternal::from_petgraph(in_graph);

    handle_duplicate_edges(&mut graph, &mut split_components);

    // first dfs, computes num, low1, low2, sub, par, deg, edge_type and fixes the edges' direction
    run_palm_dfs(&mut graph, root);

    // compute acceptable adjacency list structure
    make_adjacency_lists_acceptable(&mut graph);

    // pathfinder part: calculate high(v), newnum(v), starts_path(e) and newnum(v)
    run_pathfinder(root, &mut graph);

    // find split_components
    let mut estack = Vec::new();
    let mut tstack = Vec::new();
    find_components(
        root,
        root,
        graph.m,
        &mut graph,
        &mut estack,
        &mut tstack,
        &mut split_components,
    );

    let mut component = Component::new(ComponentType::UNSURE);
    while let Some(eid) = estack.pop() {
        component.push_edge(eid, &mut graph, false);
    }
    component.commit(&mut split_components);

    merge_components(graph.m, &mut split_components);

    let mut is_real_edge = vec![false; graph.m];
    let mut real_to_split_component = vec![None; graph.m];

    let mut edges_occs = vec![0; graph.m];
    for (i, c) in split_components.iter().enumerate() {
        for &eid in &c.edges {
            edges_occs[eid] += 1;
            is_real_edge[eid] = true;
            real_to_split_component[eid] = Some(i);

            if edges_occs[eid] > 1 {
                is_real_edge[eid] = false; // this is a virtual edge
                real_to_split_component[eid] = None;
            }
        }
    }

    // on make_adjacency_lists_acceptable we renumbered the edges, we remap them now.

    let mut pair_to_indices: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    let mut vedges = Vec::new();
    for (eid, (s, t)) in graph.edges.iter().enumerate() {
        let (s, t) = if s < t { (*s, *t) } else { (*t, *s) };

        if is_real_edge[eid] {
            pair_to_indices.entry((s, t)).or_default().push(eid);
        } else if edges_occs[eid] != 0 {
            vedges.push(eid);
        }
    }

    let mut new_edges = Vec::with_capacity(graph.m);
    let mut old_eid_to_new = vec![0; graph.m];
    for eid in in_graph.edge_references() {
        let (mut s, mut t) = (eid.source().index(), eid.target().index());

        if s > t {
            std::mem::swap(&mut s, &mut t);
        }

        let take = pair_to_indices.get_mut(&(s, t)).unwrap().pop().unwrap();
        old_eid_to_new[take] = eid.id().index();
        new_edges.push((s, t));
    }

    // vedges remain
    for &eid in &vedges {
        let (s, t) = graph.edges[eid];
        old_eid_to_new[eid] = new_edges.len();
        new_edges.push((s, t));
    }

    // remap indices
    for c in &mut split_components {
        for i in 0..c.edges.len() {
            c.edges[i] = old_eid_to_new[c.edges[i]];
        }
    }
    let mut new_is_real_edge = vec![false; new_edges.len()];
    for i in 0..new_edges.len() {
        if edges_occs[i] == 1 {
            // a real edge
            new_is_real_edge[old_eid_to_new[i]] = is_real_edge[i];
        }
    }
    let mut new_real_to_split_component = vec![None; new_edges.len()];
    for i in 0..new_edges.len() {
        if edges_occs[i] == 1 {
            new_real_to_split_component[old_eid_to_new[i]] = real_to_split_component[i];
        }
    }

    TriconnectedComponents {
        comp: split_components,
        edges: new_edges,
        is_real: new_is_real_edge,
        to_split: new_real_to_split_component,
    }
}

#[cfg(test)]
mod tests {
    use petgraph::visit::{IntoNodeReferences, NodeIndexable};

    use crate::testing::random_graphs::random_biconnected_graph;

    use super::*;

    fn are_triconnected_brute(in_graph: &UnGraph) -> Vec<Vec<bool>> {
        let n = in_graph.node_references().count();
        let mut res: Vec<Vec<bool>> = vec![vec![false; n]; n];
        let mut cap = vec![vec![0; n * 2]; n * 2]; // indices from 0 to n-1 are 'ins', rest are 'outs'

        for (u, v) in in_graph
            .edge_references()
            .map(|e| (e.source().index(), e.target().index()))
        {
            cap[u + n][v] += 1;
            cap[v + n][u] += 1;
        }
        for u in 0..n {
            cap[u][u + n] += 1; // ins to outs
        }

        fn is_3_conn(s: usize, t: usize, cap: &Vec<Vec<usize>>) -> bool {
            let mut cap = cap.clone();
            let mut vis = vec![false; cap.len()];
            fn dfs(u: usize, t: usize, cap: &mut Vec<Vec<usize>>, vis: &mut Vec<bool>) -> bool {
                vis[u] = true;
                if u == t {
                    return true;
                }
                for v in 0..cap.len() {
                    if !vis[v] && cap[u][v] > 0 {
                        if dfs(v, t, cap, vis) {
                            cap[u][v] -= 1;
                            cap[v][u] += 1;
                            return true;
                        }
                    }
                }
                false
            }
            for _ in 0..3 {
                if !dfs(s + cap.len() / 2, t, &mut cap, &mut vis) {
                    return false;
                }
                vis.fill(false);
            }
            true
        }

        for u in 0..n {
            for v in 0..n {
                if u == v {
                    continue;
                }
                res[u][v] = is_3_conn(u, v, &cap);
            }
        }

        res
    }

    fn answer_fast(
        n: usize,
        m: usize,
        split_components: &Vec<Component>,
        edges: &Vec<(usize, usize)>,
    ) -> Vec<Vec<bool>> {
        if n == 2 && m <= 2 {
            return vec![vec![false, false], vec![false, false]];
        }
        let mut res = vec![vec![false; n]; n];

        for c in split_components {
            if c.comp_type == ComponentType::S {
                // not triconnected
                continue;
            }

            let mut vertex_set = Vec::new();
            for e in c.edges.iter() {
                let (u, v) = edges[*e];
                vertex_set.push(u);
                vertex_set.push(v);
            }
            vertex_set.sort();
            vertex_set.dedup();

            for &x in &vertex_set {
                for &y in &vertex_set {
                    if x != y {
                        res[x][y] = true;
                    }
                }
            }
        }

        res
    }
    fn is_splitpair(in_graph: &UnGraph, s: usize, t: usize) -> bool {
        let n = in_graph.node_references().count();
        let mut vis = vec![false; n];
        fn dfs(u: usize, in_graph: &UnGraph, vis: &mut Vec<bool>) {
            vis[u] = true;
            for v in in_graph.neighbors(in_graph.from_index(u)) {
                if !vis[v.index()] {
                    dfs(v.index(), in_graph, vis);
                }
            }
        }

        vis[s] = true;
        vis[t] = true;

        for i in 0..n {
            if i == s || i == t {
                continue;
            }
            dfs(i, in_graph, &mut vis);
            break;
        }

        let mut direct_cnt = 0;
        for v in in_graph.neighbors(in_graph.from_index(s)) {
            if v.index() == t {
                direct_cnt += 1;
            }
        }

        vis.iter().any(|&v| !v) || direct_cnt > 1
    }
    fn verify_components(
        in_graph: &UnGraph,
        split_components: &Vec<Component>,
        edges: &Vec<(usize, usize)>,
    ) {
        let n = in_graph.node_references().count();
        let m = edges.len();

        let mut edges_occs = vec![0; m];
        for c in split_components {
            for &eid in &c.edges {
                edges_occs[eid] += 1;
            }

            if c.comp_type == ComponentType::P {
                let mut nodes = vec![];
                for &eid in &c.edges {
                    let (s, t) = edges[eid];
                    nodes.push(s);
                    nodes.push(t);
                }
                nodes.sort();
                nodes.dedup();

                assert!(nodes.len() == 2);
            } else if c.comp_type == ComponentType::S {
                let mut nodes = vec![];
                for &eid in &c.edges {
                    let (s, t) = edges[eid];
                    nodes.push(s);
                    nodes.push(t);
                }
                nodes.sort();
                nodes.dedup();

                assert!(nodes.len() >= 3);
                assert!(c.edges.len() == nodes.len());

                let mut deg = vec![0; n];
                for &eid in &c.edges {
                    let (s, t) = edges[eid];
                    deg[s] += 1;
                    deg[t] += 1;
                }

                assert!(deg.iter().all(|&d| d == 0 || d == 2));
            } else if c.comp_type == ComponentType::R {
                let mut nodes = vec![];
                for &eid in &c.edges {
                    let (s, t) = edges[eid];
                    nodes.push(s);
                    nodes.push(t);
                }
                nodes.sort();
                nodes.dedup();

                assert!(nodes.len() >= 4);
            } else {
                panic!();
            }
        }

        assert!(*edges_occs.iter().max().unwrap() <= 2);

        // if an edge occurs twice, then it's a vedge -- thus, a split pair.
        for (eid, cnt) in edges_occs.iter().enumerate() {
            let (s, t) = edges[eid];
            if *cnt == 2 {
                assert!(is_splitpair(in_graph, s, t));
            } else {
                assert!(*cnt <= 1);
            }
        }
    }

    #[cfg(all(test, not(debug_assertions)))]
    #[test]
    fn test_triconnected_components() {
        for i in 0..1000 {
            let n = 2 + i / 10;
            let m: usize = 1 + i;

            let in_graph = random_biconnected_graph(n, m, i);

            let tricon = get_triconnected_components(&in_graph);
            verify_components(&in_graph, &tricon.comp, &tricon.edges);

            let n = in_graph.node_references().count();
            let m = in_graph.edge_references().count();

            let brute_mat = are_triconnected_brute(&in_graph);
            let fast_mat = answer_fast(n, m, &tricon.comp, &tricon.edges);

            assert_eq!(brute_mat, fast_mat);
        }
    }

    #[test]
    fn test_triconnected_components_light() {
        for i in 0..100 {
            let n = 2 + i / 10;
            let m: usize = 1 + i;

            let in_graph = random_biconnected_graph(n, m, i);

            let tricon = get_triconnected_components(&in_graph);
            verify_components(&in_graph, &tricon.comp, &tricon.edges);

            let n = in_graph.node_references().count();
            let m = in_graph.edge_references().count();

            let brute_mat = are_triconnected_brute(&in_graph);
            let fast_mat = answer_fast(n, m, &tricon.comp, &tricon.edges);

            assert_eq!(brute_mat, fast_mat);
        }
    }

    #[cfg(all(test, not(debug_assertions)))]
    #[test]
    fn test_triconnected_exhaustive() {
        use crate::testing::graph_enumerator::GraphEnumeratorState;

        // tests all biconnected simple graphs with n <= 7
        for n in 2..=7 {
            let mut enumerator = GraphEnumeratorState {
                n,
                mask: 0,
                last_mask: (1 << (n * (n - 1) / 2)),
            };

            while let Some(in_graph) = enumerator.next() {
                let bct = get_block_cut_tree(&in_graph);
                if bct.cut_count > 0 || bct.block_count == 0 {
                    continue; // not biconnected
                }

                let in_graph = bct.blocks[0].clone();

                let tricon = get_triconnected_components(&in_graph);
                verify_components(&in_graph, &tricon.comp, &tricon.edges);

                let n = in_graph.node_references().count();
                let m = in_graph.edge_references().count();

                let brute_mat = are_triconnected_brute(&in_graph);
                let fast_mat = answer_fast(n, m, &tricon.comp, &tricon.edges);

                assert_eq!(brute_mat, fast_mat);
            }
        }
    }
}

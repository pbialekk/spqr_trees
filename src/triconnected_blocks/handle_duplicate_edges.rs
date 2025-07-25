use crate::triconnected_blocks::{
    graph_internal::GraphInternal,
    outside_structures::{Component, ComponentType},
};

/// Handles duplicate edges and self-loops in the graph.
///
/// This function modifies the graph in place by removing self-loops and merging duplicate edges,
/// creating a `P` component for each set of duplicates. The adjacency list is updated accordingly.
pub fn handle_duplicate_edges(graph: &mut GraphInternal, split_components: &mut Vec<Component>) {
    // stable sort by second
    let mut cnt = vec![0; graph.n];

    for &(_, t) in graph.edges.iter() {
        cnt[t] += 1;
    }
    for i in 1..graph.n {
        cnt[i] += cnt[i - 1];
    }

    let mut tmp = vec![(0, 0); graph.m];
    for &(s, t) in graph.edges.iter().rev() {
        cnt[t] -= 1;
        tmp[cnt[t]] = (s, t);
    }

    // stable sort by first
    cnt.fill(0);
    for &(s, _) in tmp.iter() {
        cnt[s] += 1;
    }
    for i in 1..graph.n {
        cnt[i] += cnt[i - 1];
    }

    for &(s, t) in tmp.iter().rev() {
        cnt[s] -= 1;
        graph.edges[cnt[s]] = (s, t);
    }

    debug_assert!(graph.edges.is_sorted());

    graph.adj = vec![Vec::new(); graph.n]; // reset adjacency list

    let mut i = 0;
    let len = graph.m;

    while i < len {
        let (s, t) = graph.edges[i];
        if s == t {
            // self-loop, we don't care about them
            i += 1;
            continue;
        }

        if i + 1 < len && graph.edges[i] == graph.edges[i + 1] {
            let mut component = Component::new(ComponentType::P);

            let (s, t) = graph.edges[i];
            let evirt = graph.new_edge(s, t, None);
            graph.adj[t].push(evirt); // add t->s edge as well, since we are not rooted yet

            component.push_edge(evirt, graph, true);

            component.push_edge(i, graph, false);

            while i + 1 < len && graph.edges[i + 1] == graph.edges[i] {
                i += 1;

                component.push_edge(i, graph, false);
            }

            split_components.push(component);
        } else {
            graph.adj[s].push(i);
            graph.adj[t].push(i); // add both directions, since we are not rooted yet
        }

        i += 1;
    }
}

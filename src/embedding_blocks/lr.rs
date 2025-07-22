use crate::embedding_blocks::structures::{ConflictPair, GraphInternal, Interval, LrOrientation};

/// Implements the DFS2 algorithm from the reference.
pub fn dfs2(g: &GraphInternal, lr_stuff: &mut LrOrientation, u: usize) -> bool {
    for &eid in g.adj[u].iter() {
        let to = g.get_other_vertex(eid, u);

        lr_stuff.stack_bottom[eid] = lr_stuff.stack.last().cloned();

        if g.parent[to] == Some(eid) {
            if !dfs2(g, lr_stuff, to) {
                return false;
            }
        } else {
            // a back edge
            lr_stuff.lowpt_edge[eid] = eid;
            lr_stuff.stack.push(ConflictPair {
                l: Interval::empty(),
                r: Interval::new(eid, eid),
            });
        }

        if g.low1[eid] < g.height[u] {
            // not a bridge
            let par_eid = g.parent[u].unwrap();

            if eid == g.adj[u][0] {
                lr_stuff.lowpt_edge[par_eid] = lr_stuff.lowpt_edge[eid];
            } else if !lr_stuff.merge(g, eid) {
                // merge constraints due to this subtree
                return false;
            }
        }
    }

    // trim intervals
    if let Some(par_eid) = g.parent[u] {
        lr_stuff.trim(g, par_eid);
        let to = g.get_other_vertex(par_eid, u);

        if g.low1[par_eid] < g.height[to] {
            // we need to compute ref[par_eid]
            let top_s = lr_stuff.stack.last().unwrap();
            let hl = top_s.l.ends;
            let hr = top_s.r.ends;

            if let Some((_, hl)) = hl {
                if hr.is_none() {
                    lr_stuff.ref_edge[par_eid] = hl;
                } else if let Some((_, hr)) = hr {
                    if g.low1[hl] > g.low1[hr] {
                        lr_stuff.ref_edge[par_eid] = hl;
                    } else {
                        lr_stuff.ref_edge[par_eid] = hr;
                    }
                } else {
                    assert!(false);
                }
            } else if let Some((_, hr)) = hr {
                lr_stuff.ref_edge[par_eid] = hr;
            } else {
                assert!(false);
            }
        }
    }

    true
}

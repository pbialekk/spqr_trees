#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import networkx as nx
import sys

def read_tests(filename):
    """
    Reads multiple tests from the input file.
    Each test consists of:
      - First line: n m (number of vertices and edges)
      - Next m lines: ui,vi (edges)
      - A line with '+' or '-' (planar or not)
      - If '+', lines with embedding: u v1 v2 ... (neighbors in CCW order)
    Returns a list of dicts: {'edges': [...], 'expected': '+/-', 'embedding': {...} or None}
    """
    tests = []
    with open(filename, 'r') as f:
        lines = [line.strip() for line in f if line.strip()]
    i = 0
    while i < len(lines):
        n, m = map(int, lines[i].split())
        i += 1
        edges = []
        for _ in range(m):
            u, v = map(int, lines[i].split(','))
            edges.append((u, v))
            i += 1
        if i >= len(lines):
            break
        expected = lines[i][0]
        i += 1
        embedding = None
        if expected == '+':
            embedding = {}
            # Read embedding lines until next test or EOF
            for _ in range(n):
                parts = lines[i].split(' ', 1)
                u_str = parts[0]
                neighbors_str = parts[1] if len(parts) > 1 else ''
                u = int(u_str.strip())
                neighbors = [int(v) for v in neighbors_str.strip().split()]
                embedding[u] = neighbors
                i += 1
        tests.append({'edges': edges, 'expected': expected, 'embedding': embedding})
    return tests

def verify_embedding(edges, embedding):
    """
    Verifies if the given embedding is a valid planar embedding for the graph.
    Returns True if valid, False otherwise.
    """
    G = nx.PlanarEmbedding()
    for u, neighbors in embedding.items():
        for i, v in enumerate(neighbors):
            if i == 0:
                G.add_half_edge(u, v)
            else:
                G.add_half_edge_ccw(u, v, neighbors[i - 1])
    try:
        G.check_structure()
        # Check if all edges are present in the embedding
        edge_set = set(map(frozenset, edges))
        emb_edges = set(frozenset((u, v)) for u in embedding for v in embedding[u])
        return edge_set == emb_edges
    except nx.NetworkXException:
        return False

def main():
    tests = read_tests('assets/python_input.in')
    print(f"Read {len(tests)} tests from input file.")
    for idx, test in enumerate(tests, 1):
        if idx % 10000 == 0:
            print(f"Processing test {idx}...")
        G = nx.Graph()
        G.add_edges_from(test['edges'])
        is_planar, embedding = nx.check_planarity(G)
        expected_planar = (test['expected'] == '+')
        if is_planar != expected_planar:
            print(f"Test {idx}: Planarity mismatch (expected {expected_planar}, got {is_planar})")
            sys.exit(1)
            continue
        if expected_planar and test['embedding'] is not None:
            if not verify_embedding(test['edges'], test['embedding']):
                print(f"Test {idx}: Embedding invalid")
                sys.exit(2)

if __name__ == "__main__":
    main()
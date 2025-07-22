#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import networkx as nx

def read_edges(filename):
    edges = []
    with open(filename, 'r') as f:
        for line in f:
            line = line.strip()
            if line and ',' in line:
                u, v = map(int, line.split(','))
                edges.append((u, v))
    return edges

def main():
    edges = read_edges('assets/test_graph.in')
    G = nx.Graph()
    G.add_edges_from(edges)
    is_planar, _ = nx.check_planarity(G)
    print("true" if is_planar else "false")

if __name__ == "__main__":
    main()
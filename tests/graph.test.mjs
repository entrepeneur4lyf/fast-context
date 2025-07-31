import test from 'ava';
import { RustworkxGraph, RustworkxDiGraph, completeGraph, pathGraph, cycleGraph } from '../index.js';

test('RustworkxGraph creation and basic operations', t => {
  const graph = new RustworkxGraph();
  
  t.is(graph.nodeCount(), 0);
  t.is(graph.edgeCount(), 0);
  
  const nodeA = graph.addNode("A");
  const nodeB = graph.addNode("B");
  const nodeC = graph.addNode("C");
  
  t.is(graph.nodeCount(), 3);
  
  const edge1 = graph.addEdge(nodeA, nodeB, 1.5);
  const edge2 = graph.addEdge(nodeB, nodeC, 2.0);
  const edge3 = graph.addEdge(nodeA, nodeC, 3.0);
  
  t.is(graph.edgeCount(), 3);
  t.not(edge1, null);
  t.not(edge2, null);
  t.not(edge3, null);
});

test('RustworkxGraph betweenness centrality', t => {
  const graph = new RustworkxGraph();
  
  const nodeA = graph.addNode("A");
  const nodeB = graph.addNode("B");
  const nodeC = graph.addNode("C");
  
  graph.addEdge(nodeA, nodeB, 1.0);
  graph.addEdge(nodeB, nodeC, 1.0);
  
  const centrality = graph.betweennessCentrality(true, false);
  
  t.is(centrality.length, 3);
  t.true(centrality[nodeB] > centrality[nodeA]); // B is more central
  t.true(centrality[nodeB] > centrality[nodeC]); // B is more central
});

test('RustworkxGraph dijkstra shortest paths', t => {
  const graph = new RustworkxGraph();
  
  const nodeA = graph.addNode("A");
  const nodeB = graph.addNode("B");
  const nodeC = graph.addNode("C");
  
  graph.addEdge(nodeA, nodeB, 1.0);
  graph.addEdge(nodeB, nodeC, 2.0);
  graph.addEdge(nodeA, nodeC, 5.0); // Longer direct path
  
  const distances = graph.dijkstraShortestPaths(nodeA);
  
  t.is(distances.length, 3);
  t.is(distances[nodeA], 0.0);
  t.is(distances[nodeB], 1.0);
  t.is(distances[nodeC], 3.0); // A->B->C is shorter than A->C
});

test('RustworkxDiGraph creation and cycle detection', t => {
  const digraph = new RustworkxDiGraph();
  
  const nodeA = digraph.addNode("A");
  const nodeB = digraph.addNode("B");
  const nodeC = digraph.addNode("C");
  
  // Create a DAG first
  digraph.addEdge(nodeA, nodeB, 1.0);
  digraph.addEdge(nodeB, nodeC, 1.0);
  
  t.false(digraph.isCyclic());
  
  // Add edge to create cycle
  digraph.addEdge(nodeC, nodeA, 1.0);
  
  t.true(digraph.isCyclic());
});

test('RustworkxGraph edge validation', t => {
  const graph = new RustworkxGraph();
  
  const nodeA = graph.addNode("A");
  
  // Try to add edge to non-existent node
  const invalidEdge = graph.addEdge(nodeA, 999, 1.0);
  
  t.is(invalidEdge, null);
});

test('Complete graph generator', t => {
  const graph = completeGraph(4);
  
  t.is(graph.nodeCount(), 4);
  t.is(graph.edgeCount(), 6); // n*(n-1)/2 for complete graph
});

test('Path graph generator', t => {
  const graph = pathGraph(5);
  
  t.is(graph.nodeCount(), 5);
  t.is(graph.edgeCount(), 4); // n-1 edges for path graph
});

test('Cycle graph generator', t => {
  const graph = cycleGraph(4);
  
  t.is(graph.nodeCount(), 4);
  t.is(graph.edgeCount(), 4); // n edges for cycle graph
});

test('Empty graph generators', t => {
  const completeEmpty = completeGraph(0);
  const pathEmpty = pathGraph(0);
  const cycleEmpty = cycleGraph(0);
  
  t.is(completeEmpty.nodeCount(), 0);
  t.is(pathEmpty.nodeCount(), 0);
  t.is(cycleEmpty.nodeCount(), 0);
  
  t.is(completeEmpty.edgeCount(), 0);
  t.is(pathEmpty.edgeCount(), 0);
  t.is(cycleEmpty.edgeCount(), 0);
});

test('Single node graphs', t => {
  const completeSingle = completeGraph(1);
  const pathSingle = pathGraph(1);
  const cycleSingle = cycleGraph(1);
  
  t.is(completeSingle.nodeCount(), 1);
  t.is(pathSingle.nodeCount(), 1);
  t.is(cycleSingle.nodeCount(), 1);
  
  t.is(completeSingle.edgeCount(), 0);
  t.is(pathSingle.edgeCount(), 0);
  t.is(cycleSingle.edgeCount(), 0);
});
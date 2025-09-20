package graph

import (
	"errors"
	"fmt"
	"math"
	"sort"
)

// Graph represents an undirected graph
type Graph interface {
	AddNode(id string, weight float64) error
	AddEdge(from, to string, weight float64) error
	RemoveNode(id string) error
	RemoveEdge(from, to string) error
	HasNode(id string) bool
	HasEdge(from, to string) bool
	GetNode(id string) (*Node, error)
	GetEdge(from, to string) (*Edge, error)
	GetNodes() []*Node
	GetEdges() []*Edge
	NodeCount() int
	EdgeCount() int
	GetDegree(nodeID string) (int, error)
	GetNeighbors(nodeID string) ([]*Node, error)
	GetNodeWeight(id string) (float64, error)
	GetEdgeWeight(from, to string) (float64, error)
	IsConnected() bool
	Copy() Graph
}

// DiGraph represents a directed graph
type DiGraph interface {
	Graph
	AddDirectedEdge(from, to string, weight float64) error
	GetInDegree(nodeID string) (int, error)
	GetOutDegree(nodeID string) (int, error)
	GetPredecessors(nodeID string) ([]*Node, error)
	GetSuccessors(nodeID string) ([]*Node, error)
	IsStronglyConnected() bool
	TopologicalSort() ([]string, error)
	GetStronglyConnectedComponents() [][]string
}

// Node represents a graph node
type Node struct {
	ID     string  `json:"id"`
	Weight float64 `json:"weight"`
	Data   interface{} `json:"data,omitempty"`
}

// Edge represents a graph edge
type Edge struct {
	From   string  `json:"from"`
	To     string  `json:"to"`
	Weight float64 `json:"weight"`
	Data   interface{} `json:"data,omitempty"`
}

// PathResult represents the result of a pathfinding algorithm
type PathResult struct {
	Path        []string `json:"path"`
	Distance    float64  `json:"distance"`
	VisitCount  int      `json:"visitCount"`
	Found       bool     `json:"found"`
}

// CentralityResult represents centrality measures for nodes
type CentralityResult struct {
	NodeID             string  `json:"nodeId"`
	DegreeCentrality   float64 `json:"degreeCentrality"`
	BetweennessCentrality float64 `json:"betweennessCentrality"`
	ClosenessCentrality float64 `json:"closenessCentrality"`
	EigenvectorCentrality float64 `json:"eigenvectorCentrality"`
	PageRank          float64 `json:"pageRank"`
}

// Component represents a connected component
type Component struct {
	ID      string   `json:"id"`
	Nodes   []string `json:"nodes"`
	Size    int      `json:"size"`
	IsTree  bool     `json:"isTree"`
}

// SimpleGraph implements the Graph interface
type SimpleGraph struct {
	nodes map[string]*Node
	edges map[string]map[string]*Edge
}

// SimpleDiGraph implements the DiGraph interface
type SimpleDiGraph struct {
	nodes map[string]*Node
	edges map[string]map[string]*Edge // from -> to -> edge
}

// NewGraph creates a new undirected graph
func NewGraph() Graph {
	return &SimpleGraph{
		nodes: make(map[string]*Node),
		edges: make(map[string]map[string]*Edge),
	}
}

// NewDiGraph creates a new directed graph
func NewDiGraph() DiGraph {
	return &SimpleDiGraph{
		nodes: make(map[string]*Node),
		edges: make(map[string]map[string]*Edge),
	}
}

// SimpleGraph implementation

func (g *SimpleGraph) AddNode(id string, weight float64) error {
	if id == "" {
		return errors.New("node ID cannot be empty")
	}
	
	if _, exists := g.nodes[id]; exists {
		return fmt.Errorf("node %s already exists", id)
	}
	
	g.nodes[id] = &Node{
		ID:     id,
		Weight: weight,
	}
	g.edges[id] = make(map[string]*Edge)
	
	return nil
}

func (g *SimpleGraph) AddEdge(from, to string, weight float64) error {
	if !g.HasNode(from) {
		return fmt.Errorf("node %s does not exist", from)
	}
	if !g.HasNode(to) {
		return fmt.Errorf("node %s does not exist", to)
	}
	if from == to {
		return errors.New("cannot add self-loop in undirected graph")
	}
	
	g.edges[from][to] = &Edge{From: from, To: to, Weight: weight}
	g.edges[to][from] = &Edge{From: to, To: from, Weight: weight}
	
	return nil
}

func (g *SimpleGraph) RemoveNode(id string) error {
	if !g.HasNode(id) {
		return fmt.Errorf("node %s does not exist", id)
	}
	
	// Remove all edges connected to this node
	for neighbor := range g.edges[id] {
		delete(g.edges[neighbor], id)
	}
	
	delete(g.nodes, id)
	delete(g.edges, id)
	
	return nil
}

func (g *SimpleGraph) RemoveEdge(from, to string) error {
	if !g.HasEdge(from, to) {
		return fmt.Errorf("edge %s-%s does not exist", from, to)
	}
	
	delete(g.edges[from], to)
	delete(g.edges[to], from)
	
	return nil
}

func (g *SimpleGraph) HasNode(id string) bool {
	_, exists := g.nodes[id]
	return exists
}

func (g *SimpleGraph) HasEdge(from, to string) bool {
	if _, exists := g.edges[from]; exists {
		_, edgeExists := g.edges[from][to]
		return edgeExists
	}
	return false
}

func (g *SimpleGraph) GetNode(id string) (*Node, error) {
	node, exists := g.nodes[id]
	if !exists {
		return nil, fmt.Errorf("node %s not found", id)
	}
	return node, nil
}

func (g *SimpleGraph) GetEdge(from, to string) (*Edge, error) {
	if !g.HasEdge(from, to) {
		return nil, fmt.Errorf("edge %s-%s not found", from, to)
	}
	return g.edges[from][to], nil
}

func (g *SimpleGraph) GetNodes() []*Node {
	nodes := make([]*Node, 0, len(g.nodes))
	for _, node := range g.nodes {
		nodes = append(nodes, node)
	}
	return nodes
}

func (g *SimpleGraph) GetEdges() []*Edge {
	edges := make([]*Edge, 0)
	seen := make(map[string]bool)
	
	for from, neighbors := range g.edges {
		for to, edge := range neighbors {
			edgeKey := edgeKey(from, to)
			if !seen[edgeKey] {
				edges = append(edges, edge)
				seen[edgeKey] = true
			}
		}
	}
	
	return edges
}

func (g *SimpleGraph) NodeCount() int {
	return len(g.nodes)
}

func (g *SimpleGraph) EdgeCount() int {
	count := 0
	for _, neighbors := range g.edges {
		count += len(neighbors)
	}
	return count / 2 // Divide by 2 since undirected
}

func (g *SimpleGraph) GetDegree(nodeID string) (int, error) {
	if !g.HasNode(nodeID) {
		return 0, fmt.Errorf("node %s not found", nodeID)
	}
	return len(g.edges[nodeID]), nil
}

func (g *SimpleGraph) GetNeighbors(nodeID string) ([]*Node, error) {
	if !g.HasNode(nodeID) {
		return nil, fmt.Errorf("node %s not found", nodeID)
	}
	
	neighbors := make([]*Node, 0, len(g.edges[nodeID]))
	for neighborID := range g.edges[nodeID] {
		neighbor, _ := g.GetNode(neighborID)
		neighbors = append(neighbors, neighbor)
	}
	
	return neighbors, nil
}

func (g *SimpleGraph) GetNodeWeight(id string) (float64, error) {
	node, err := g.GetNode(id)
	if err != nil {
		return 0, err
	}
	return node.Weight, nil
}

func (g *SimpleGraph) GetEdgeWeight(from, to string) (float64, error) {
	edge, err := g.GetEdge(from, to)
	if err != nil {
		return 0, err
	}
	return edge.Weight, nil
}

func (g *SimpleGraph) IsConnected() bool {
	if g.NodeCount() == 0 {
		return true
	}
	
	visited := make(map[string]bool)
	startNode := g.GetNodes()[0].ID
	
	var component []string
	g.dfs(startNode, visited, &component)
	
	return len(visited) == g.NodeCount()
}

func (g *SimpleGraph) Copy() Graph {
	newGraph := NewGraph()
	
	// Copy nodes
	for _, node := range g.GetNodes() {
		newGraph.AddNode(node.ID, node.Weight)
	}
	
	// Copy edges
	for _, edge := range g.GetEdges() {
		newGraph.AddEdge(edge.From, edge.To, edge.Weight)
	}
	
	return newGraph
}

// SimpleDiGraph implementation

func (g *SimpleDiGraph) AddNode(id string, weight float64) error {
	if id == "" {
		return errors.New("node ID cannot be empty")
	}
	
	if _, exists := g.nodes[id]; exists {
		return fmt.Errorf("node %s already exists", id)
	}
	
	g.nodes[id] = &Node{
		ID:     id,
		Weight: weight,
	}
	g.edges[id] = make(map[string]*Edge)
	
	return nil
}

func (g *SimpleDiGraph) AddEdge(from, to string, weight float64) error {
	return g.AddDirectedEdge(from, to, weight)
}

func (g *SimpleDiGraph) AddDirectedEdge(from, to string, weight float64) error {
	if !g.HasNode(from) {
		return fmt.Errorf("node %s does not exist", from)
	}
	if !g.HasNode(to) {
		return fmt.Errorf("node %s does not exist", to)
	}
	
	g.edges[from][to] = &Edge{From: from, To: to, Weight: weight}
	
	return nil
}

func (g *SimpleDiGraph) RemoveNode(id string) error {
	if !g.HasNode(id) {
		return fmt.Errorf("node %s does not exist", id)
	}
	
	// Remove all outgoing edges
	delete(g.edges, id)
	
	// Remove all incoming edges
	for from := range g.edges {
		delete(g.edges[from], id)
	}
	
	delete(g.nodes, id)
	
	return nil
}

func (g *SimpleDiGraph) RemoveEdge(from, to string) error {
	if !g.HasEdge(from, to) {
		return fmt.Errorf("edge %s->%s does not exist", from, to)
	}
	
	delete(g.edges[from], to)
	
	return nil
}

func (g *SimpleDiGraph) HasNode(id string) bool {
	_, exists := g.nodes[id]
	return exists
}

func (g *SimpleDiGraph) HasEdge(from, to string) bool {
	if _, exists := g.edges[from]; exists {
		_, edgeExists := g.edges[from][to]
		return edgeExists
	}
	return false
}

func (g *SimpleDiGraph) GetNode(id string) (*Node, error) {
	node, exists := g.nodes[id]
	if !exists {
		return nil, fmt.Errorf("node %s not found", id)
	}
	return node, nil
}

func (g *SimpleDiGraph) GetEdge(from, to string) (*Edge, error) {
	if !g.HasEdge(from, to) {
		return nil, fmt.Errorf("edge %s->%s not found", from, to)
	}
	return g.edges[from][to], nil
}

func (g *SimpleDiGraph) GetNodes() []*Node {
	nodes := make([]*Node, 0, len(g.nodes))
	for _, node := range g.nodes {
		nodes = append(nodes, node)
	}
	return nodes
}

func (g *SimpleDiGraph) GetEdges() []*Edge {
	edges := make([]*Edge, 0)
	for _, neighbors := range g.edges {
		for _, edge := range neighbors {
			edges = append(edges, edge)
		}
	}
	return edges
}

func (g *SimpleDiGraph) NodeCount() int {
	return len(g.nodes)
}

func (g *SimpleDiGraph) EdgeCount() int {
	count := 0
	for _, neighbors := range g.edges {
		count += len(neighbors)
	}
	return count
}

func (g *SimpleDiGraph) GetDegree(nodeID string) (int, error) {
	return g.GetOutDegree(nodeID)
}

func (g *SimpleDiGraph) GetInDegree(nodeID string) (int, error) {
	if !g.HasNode(nodeID) {
		return 0, fmt.Errorf("node %s not found", nodeID)
	}
	
	count := 0
	for from := range g.edges {
		if _, exists := g.edges[from][nodeID]; exists {
			count++
		}
	}
	
	return count, nil
}

func (g *SimpleDiGraph) GetOutDegree(nodeID string) (int, error) {
	if !g.HasNode(nodeID) {
		return 0, fmt.Errorf("node %s not found", nodeID)
	}
	return len(g.edges[nodeID]), nil
}

func (g *SimpleDiGraph) GetNeighbors(nodeID string) ([]*Node, error) {
	return g.GetSuccessors(nodeID)
}

func (g *SimpleDiGraph) GetPredecessors(nodeID string) ([]*Node, error) {
	if !g.HasNode(nodeID) {
		return nil, fmt.Errorf("node %s not found", nodeID)
	}
	
	predecessors := make([]*Node, 0)
	for from := range g.edges {
		if _, exists := g.edges[from][nodeID]; exists {
			node, _ := g.GetNode(from)
			predecessors = append(predecessors, node)
		}
	}
	
	return predecessors, nil
}

func (g *SimpleDiGraph) GetSuccessors(nodeID string) ([]*Node, error) {
	if !g.HasNode(nodeID) {
		return nil, fmt.Errorf("node %s not found", nodeID)
	}
	
	successors := make([]*Node, 0, len(g.edges[nodeID]))
	for successorID := range g.edges[nodeID] {
		successor, _ := g.GetNode(successorID)
		successors = append(successors, successor)
	}
	
	return successors, nil
}

func (g *SimpleDiGraph) GetNodeWeight(id string) (float64, error) {
	node, err := g.GetNode(id)
	if err != nil {
		return 0, err
	}
	return node.Weight, nil
}

func (g *SimpleDiGraph) GetEdgeWeight(from, to string) (float64, error) {
	edge, err := g.GetEdge(from, to)
	if err != nil {
		return 0, err
	}
	return edge.Weight, nil
}

func (g *SimpleDiGraph) IsConnected() bool {
	// For directed graphs, we check weak connectivity
	undirectedCopy := NewGraph()
	
	// Copy all nodes
	for _, node := range g.GetNodes() {
		undirectedCopy.AddNode(node.ID, node.Weight)
	}
	
	// Copy all edges as undirected
	for _, edge := range g.GetEdges() {
		undirectedCopy.AddEdge(edge.From, edge.To, edge.Weight)
	}
	
	return undirectedCopy.IsConnected()
}

func (g *SimpleDiGraph) IsStronglyConnected() bool {
	if g.NodeCount() == 0 {
		return true
	}
	
	// Check if every node can reach every other node
	for _, startNode := range g.GetNodes() {
		reachable := g.getReachableNodes(startNode.ID)
		if len(reachable) != g.NodeCount() {
			return false
		}
	}
	
	return true
}

func (g *SimpleDiGraph) Copy() Graph {
	newGraph := NewDiGraph()
	
	// Copy nodes
	for _, node := range g.GetNodes() {
		newGraph.AddNode(node.ID, node.Weight)
	}
	
	// Copy edges
	for _, edge := range g.GetEdges() {
		newGraph.AddDirectedEdge(edge.From, edge.To, edge.Weight)
	}
	
	return newGraph
}

func (g *SimpleDiGraph) TopologicalSort() ([]string, error) {
	// Kahn's algorithm for topological sort
	
	// Calculate in-degrees
	inDegree := make(map[string]int)
	for nodeID := range g.nodes {
		inDegree[nodeID] = 0
	}
	
	for _, neighbors := range g.edges {
		for to := range neighbors {
			inDegree[to]++
		}
	}
	
	// Find nodes with no incoming edges
	queue := []string{}
	for nodeID, degree := range inDegree {
		if degree == 0 {
			queue = append(queue, nodeID)
		}
	}
	
	result := []string{}
	processed := 0
	
	for len(queue) > 0 {
		node := queue[0]
		queue = queue[1:]
		result = append(result, node)
		processed++
		
		// Remove edges from this node and update in-degrees
		for neighbor := range g.edges[node] {
			inDegree[neighbor]--
			if inDegree[neighbor] == 0 {
				queue = append(queue, neighbor)
			}
		}
	}
	
	if processed != g.NodeCount() {
		return nil, errors.New("graph has a cycle, topological sort not possible")
	}
	
	return result, nil
}

func (g *SimpleDiGraph) GetStronglyConnectedComponents() [][]string {
	// Kosaraju's algorithm for finding strongly connected components
	
	visited := make(map[string]bool)
	order := []string{}
	
	// First pass: get finishing times
	for _, node := range g.GetNodes() {
		if !visited[node.ID] {
			g.dfsFirstPass(node.ID, visited, &order)
		}
	}
	
	// Reverse the graph
	reversed := g.reverseGraph()
	
	// Second pass: process nodes in reverse order of finishing times
	visited = make(map[string]bool)
	components := [][]string{}
	
	for i := len(order) - 1; i >= 0; i-- {
		nodeID := order[i]
		if !visited[nodeID] {
			component := []string{}
			node, _ := reversed.GetNode(nodeID)
			reversed.(*SimpleDiGraph).dfsSecondPass(node, visited, &component)
			components = append(components, component)
		}
	}
	
	return components
}

// Graph algorithms

// DijkstraShortestPath finds the shortest path between two nodes
func DijkstraShortestPath(g Graph, start, end string) (*PathResult, error) {
	if !g.HasNode(start) {
		return nil, fmt.Errorf("start node %s not found", start)
	}
	if !g.HasNode(end) {
		return nil, fmt.Errorf("end node %s not found", end)
	}
	
	distances := make(map[string]float64)
	previous := make(map[string]string)
	visited := make(map[string]bool)
	
	// Initialize distances
	for _, node := range g.GetNodes() {
		distances[node.ID] = math.Inf(1)
	}
	distances[start] = 0
	
	// Priority queue (simplified - using slice)
	nodes := []string{start}
	
	for len(nodes) > 0 {
		// Find node with minimum distance
		sort.Slice(nodes, func(i, j int) bool {
			return distances[nodes[i]] < distances[nodes[j]]
		})
		
		current := nodes[0]
		nodes = nodes[1:]
		
		if visited[current] {
			continue
		}
		
		visited[current] = true
		
		if current == end {
			break
		}
		
		neighbors, _ := g.GetNeighbors(current)
		for _, neighbor := range neighbors {
			if visited[neighbor.ID] {
				continue
			}
			
			edgeWeight, _ := g.GetEdgeWeight(current, neighbor.ID)
			newDistance := distances[current] + edgeWeight
			
			if newDistance < distances[neighbor.ID] {
				distances[neighbor.ID] = newDistance
				previous[neighbor.ID] = current
				
				// Add to queue if not already there
				found := false
				for _, node := range nodes {
					if node == neighbor.ID {
						found = true
						break
					}
				}
				if !found {
					nodes = append(nodes, neighbor.ID)
				}
			}
		}
	}
	
	// Reconstruct path
	path := []string{}
	current := end
	for current != "" {
		path = append([]string{current}, path...)
		current = previous[current]
	}
	
	if path[0] != start {
		// No path found
		return &PathResult{
			Path:       []string{},
			Distance:   math.Inf(1),
			VisitCount: len(visited),
			Found:      false,
		}, nil
	}
	
	return &PathResult{
		Path:       path,
		Distance:   distances[end],
		VisitCount: len(visited),
		Found:      true,
	}, nil
}

// FloydWarshallAllPairs computes shortest paths between all pairs of nodes
func FloydWarshallAllPairs(g Graph) (map[string]map[string]float64, map[string]map[string]string) {
	nodes := g.GetNodes()
	nodeIDs := make([]string, len(nodes))
	for i, node := range nodes {
		nodeIDs[i] = node.ID
	}
	
	// Initialize distance and next matrices
	dist := make(map[string]map[string]float64)
	next := make(map[string]map[string]string)
	
	for _, i := range nodeIDs {
		dist[i] = make(map[string]float64)
		next[i] = make(map[string]string)
		for _, j := range nodeIDs {
			if i == j {
				dist[i][j] = 0
				next[i][j] = j
			} else {
				dist[i][j] = math.Inf(1)
				next[i][j] = ""
			}
		}
	}
	
	// Fill in direct edges
	for _, edge := range g.GetEdges() {
		dist[edge.From][edge.To] = edge.Weight
		next[edge.From][edge.To] = edge.To
	}
	
	// Floyd-Warshall algorithm
	for _, k := range nodeIDs {
		for _, i := range nodeIDs {
			for _, j := range nodeIDs {
				if dist[i][k]+dist[k][j] < dist[i][j] {
					dist[i][j] = dist[i][k] + dist[k][j]
					next[i][j] = next[i][k]
				}
			}
		}
	}
	
	return dist, next
}

// ConnectedComponents finds all connected components in an undirected graph
func ConnectedComponents(g Graph) []*Component {
	visited := make(map[string]bool)
	components := []*Component{}
	
	for _, node := range g.GetNodes() {
		if !visited[node.ID] {
			componentNodes := []string{}
			g.(*SimpleGraph).dfs(node.ID, visited, &componentNodes)
			
			component := &Component{
				ID:    fmt.Sprintf("component_%d", len(components)),
				Nodes: componentNodes,
				Size:  len(componentNodes),
			}
			
			// Check if component is a tree
			component.IsTree = isTree(g, componentNodes)
			
			components = append(components, component)
		}
	}
	
	return components
}

// BetweennessCentrality calculates betweenness centrality for all nodes
func BetweennessCentrality(g Graph) map[string]float64 {
	nodes := g.GetNodes()
	nodeIDs := make([]string, len(nodes))
	for i, node := range nodes {
		nodeIDs[i] = node.ID
	}
	
	centrality := make(map[string]float64)
	for _, nodeID := range nodeIDs {
		centrality[nodeID] = 0
	}
	
	// For each pair of nodes
	for _, s := range nodeIDs {
		for _, t := range nodeIDs {
			if s == t {
				continue
			}
			
			// Find all shortest paths from s to t
			paths := findAllShortestPaths(g, s, t)
			
			if len(paths) > 0 {
				// Count how many paths each node is on
				for _, nodeID := range nodeIDs {
					if nodeID == s || nodeID == t {
						continue
					}
					
					count := 0
					for _, path := range paths {
						if containsNode(path, nodeID) {
							count++
						}
					}
					
					centrality[nodeID] += float64(count) / float64(len(paths))
				}
			}
		}
	}
	
	return centrality
}

// ClosenessCentrality calculates closeness centrality for all nodes
func ClosenessCentrality(g Graph) map[string]float64 {
	nodes := g.GetNodes()
	centrality := make(map[string]float64)
	
	for _, node := range nodes {
		totalDistance := 0.0
		reachableCount := 0
		
		for _, otherNode := range nodes {
			if node.ID == otherNode.ID {
				continue
			}
			
			result, err := DijkstraShortestPath(g, node.ID, otherNode.ID)
			if err == nil && result.Found {
				totalDistance += result.Distance
				reachableCount++
			}
		}
		
		if reachableCount > 0 {
			centrality[node.ID] = float64(reachableCount) / totalDistance
		} else {
			centrality[node.ID] = 0
		}
	}
	
	return centrality
}

// PageRank calculates PageRank for all nodes
func PageRank(g Graph, dampingFactor float64, iterations int) map[string]float64 {
	nodes := g.GetNodes()
	nodeIDs := make([]string, len(nodes))
	for i, node := range nodes {
		nodeIDs[i] = node.ID
	}
	
	pagerank := make(map[string]float64)
	
	// Initialize with equal values
	initialValue := 1.0 / float64(len(nodeIDs))
	for _, nodeID := range nodeIDs {
		pagerank[nodeID] = initialValue
	}
	
	for iter := 0; iter < iterations; iter++ {
		newPagerank := make(map[string]float64)
		
		for _, nodeID := range nodeIDs {
			sum := 0.0
			
			// Get all incoming links
			for _, otherNode := range nodes {
				if otherNode.ID == nodeID {
					continue
				}
				
				if g.HasEdge(otherNode.ID, nodeID) {
					neighbors, _ := g.GetNeighbors(otherNode.ID)
					if len(neighbors) > 0 {
						sum += pagerank[otherNode.ID] / float64(len(neighbors))
					}
				}
			}
			
			newPagerank[nodeID] = (1-dampingFactor)/float64(len(nodeIDs)) + dampingFactor*sum
		}
		
		pagerank = newPagerank
	}
	
	return pagerank
}

// Helper functions


func (g *SimpleGraph) dfs(nodeID string, visited map[string]bool, component *[]string) {
	visited[nodeID] = true
	*component = append(*component, nodeID)
	
	neighbors, _ := g.GetNeighbors(nodeID)
	for _, neighbor := range neighbors {
		if !visited[neighbor.ID] {
			g.dfs(neighbor.ID, visited, component)
		}
	}
}

func (g *SimpleDiGraph) dfsFirstPass(nodeID string, visited map[string]bool, order *[]string) {
	visited[nodeID] = true
	
	successors, _ := g.GetSuccessors(nodeID)
	for _, successor := range successors {
		if !visited[successor.ID] {
			g.dfsFirstPass(successor.ID, visited, order)
		}
	}
	
	*order = append(*order, nodeID)
}

func (g *SimpleDiGraph) dfsSecondPass(node *Node, visited map[string]bool, component *[]string) {
	visited[node.ID] = true
	*component = append(*component, node.ID)
	
	successors, _ := g.GetSuccessors(node.ID)
	for _, successor := range successors {
		if !visited[successor.ID] {
			g.dfsSecondPass(successor, visited, component)
		}
	}
}

func (g *SimpleDiGraph) reverseGraph() DiGraph {
	reversed := NewDiGraph()
	
	// Copy all nodes
	for _, node := range g.GetNodes() {
		reversed.AddNode(node.ID, node.Weight)
	}
	
	// Reverse all edges
	for _, edge := range g.GetEdges() {
		reversed.AddDirectedEdge(edge.To, edge.From, edge.Weight)
	}
	
	return reversed
}

func (g *SimpleDiGraph) getReachableNodes(startNode string) map[string]bool {
	visited := make(map[string]bool)
	stack := []string{startNode}
	
	for len(stack) > 0 {
		node := stack[len(stack)-1]
		stack = stack[:len(stack)-1]
		
		if visited[node] {
			continue
		}
		
		visited[node] = true
		
		successors, _ := g.GetSuccessors(node)
		for _, successor := range successors {
			if !visited[successor.ID] {
				stack = append(stack, successor.ID)
			}
		}
	}
	
	return visited
}

func edgeKey(from, to string) string {
	if from < to {
		return from + "_" + to
	}
	return to + "_" + from
}

func findAllShortestPaths(g Graph, start, end string) [][]string {
	result, err := DijkstraShortestPath(g, start, end)
	if err != nil || !result.Found {
		return [][]string{}
	}
	
	// Simplified - return just one path
	return [][]string{result.Path}
}

func containsNode(path []string, nodeID string) bool {
	for _, pathNode := range path {
		if pathNode == nodeID {
			return true
		}
	}
	return false
}

func isTree(g Graph, nodes []string) bool {
	if len(nodes) <= 1 {
		return true
	}
	
	// In a tree, number of edges = number of nodes - 1
	edgeCount := 0
	for _, nodeID := range nodes {
		neighbors, _ := g.GetNeighbors(nodeID)
		for _, neighbor := range neighbors {
			if containsNode(nodes, neighbor.ID) {
				edgeCount++
			}
		}
	}
	
	// Each edge counted twice, so divide by 2
	return edgeCount/2 == len(nodes)-1
}
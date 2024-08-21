use std::collections::VecDeque;

pub fn bfs(graph: &Vec<Vec<usize>>, start: usize) -> (Vec<usize>, Vec<usize>) {
    let mut parent = Vec::new();
    let mut distance = Vec::new();
    for _ in 0..graph.len() {
        parent.push(usize::MAX);
        distance.push(usize::MAX);
    }
    distance[start] = 0;
    let mut q: VecDeque<usize> = VecDeque::new();
    q.push_back(start);
    while !q.is_empty() {
        let node = q.pop_front().expect("Queue must not be empty during while loop");
        for neighbor in graph[node].clone() {
            if distance[neighbor] == usize::MAX {
                parent[neighbor] = node;
                distance[neighbor] = distance[node] + 1;
                q.push_back(neighbor);
            }
        }
    }
    (parent, distance)
}

/// Finds the shortest path between start and the first end that it connects to.<br>
/// Returns the path as a vector of the IDs in order.<br>
/// If no paths exists, returns `None`
pub fn find_first_shortest_path(graph: &Vec<Vec<usize>>, start: usize, ends: &mut Vec<usize>) -> Option<Vec<usize>> {
    let (parent, distance) = bfs(&graph, start);
    for (index, destination) in ends.iter().enumerate() {
        let destination = *destination;
        if distance[destination] < usize::MAX {
            let mut path = Vec::new();
            let mut current_node = destination;
            path.push(destination);
            while parent[current_node] != usize::MAX {
                current_node = parent[current_node];
                path.push(current_node);
            }
            debug_assert!(current_node == start);
            path.reverse();
            ends.swap_remove(index); // ensure that this destination is not used again
            return Some(path)
        }
    }
    None
}

/// This function matches each `start` with each `end`
/// on a one-to-on basis by using the shortest paths it can.
pub fn find_paths_between(starts: &Vec<usize>, ends: &Vec<usize>, graph: &Vec<Vec<usize>>) -> Vec<Vec<usize>> {
    let mut paths = Vec::new();
    let mut ends = ends.to_owned();
    for start in starts {
        if let Some(path) = find_first_shortest_path(graph, *start, &mut ends) {
            paths.push(path);
        }
    }
    paths
}
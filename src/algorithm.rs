use std::collections::VecDeque;

pub fn bfs(graph: &Vec<Vec<usize>>, start: usize, parent: &mut Vec<usize>, distance: &mut Vec<usize>) {
    let mut q: VecDeque<usize> = VecDeque::new();
    distance[start] = 0;
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
}

/// Finds the shortest path between start and the first end that it connects to.<br>
/// Returns the path as a vector of the ids in order.<br>
/// If no paths exists, returns None
pub fn find_first_shortest_path(graph: &Vec<Vec<usize>>, start: usize, ends: &Vec<usize>) -> Option<Vec<usize>> {
    let v = graph.len();
    let mut parent = Vec::new();
    let mut distance = Vec::new();
    for _ in 0..v {
        parent.push(usize::MAX);
        distance.push(usize::MAX);
    }
    bfs(&graph, start, &mut parent, &mut distance);

    for destination in ends {
        let destination = *destination;
        if distance[destination] == usize::MAX {
            continue;
        }
        let mut path = Vec::new();
        let mut current_node = destination;
        path.push(destination);
        while parent[current_node] != usize::MAX {
            path.push(parent[current_node]);
            current_node = parent[current_node];
        }
        path.reverse();
        return Some(path)
    }
    None
}
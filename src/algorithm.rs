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

pub fn find_shortest_path(graph: &Vec<Vec<usize>>, start: usize, destination: usize) -> Option<Vec<usize>> {
    let v = graph.len();
    let mut parent = Vec::<usize>::new();
    let mut distance = Vec::<usize>::new();
    for _ in 0..v {
        parent.push(usize::MAX);
        distance.push(usize::MAX);
    }
    bfs(&graph, start, &mut parent, &mut distance);

    if distance[destination] == usize::MAX {
        return None;
    }

    let mut path = Vec::<usize>::new();
    let mut current_node = destination;
    path.push(destination);
    while parent[current_node] != usize::MAX {
        path.push(parent[current_node]);
        current_node = parent[current_node];
    }
    path.reverse();
    Some(path)
}
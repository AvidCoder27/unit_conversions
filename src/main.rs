mod graphing;
use graphing::{Conversion, IDGenerator, Step, Unit};
use std::{collections::{HashMap, VecDeque}, io, usize};
use fast_float;

fn main() {
    let mut generator: IDGenerator = IDGenerator::new();
    let mut unit_ids = HashMap::<usize, Unit>::new();
    let mut aliases = HashMap::<String, usize>::new();

    {
        let mut yards = Unit::new("yards", &mut generator);
        let mut feet = Unit::new("feet", &mut generator);
        let mut meters = Unit::new("meters", &mut generator);
        let mut kilometers = Unit::new("kilometers", &mut generator);

        insert_aliases(&mut aliases, yards.get_id(), ["yds", "yards", "yard", "yd"].to_vec());
        insert_aliases(&mut aliases, feet.get_id(), ["foot", "feet", "ft"].to_vec());
        insert_aliases(&mut aliases, meters.get_id(), ["m", "meters", "meter"].to_vec());
        insert_aliases(&mut aliases, kilometers.get_id(), ["km", "kilometers", "kilometer"].to_vec());

        let yards_to_feet = Conversion::new(3.0, 1.0);
        feet.push_edge(yards.get_id(), yards_to_feet.inverse());
        yards.push_edge(feet.get_id(), yards_to_feet);

        let feet_to_meters = Conversion::new(1.0, 3.28084);
        meters.push_edge(feet.get_id(), feet_to_meters.inverse());
        feet.push_edge(meters.get_id(), feet_to_meters);

        let km_to_meters = Conversion::new(1000.0, 1.0);
        meters.push_edge(kilometers.get_id(), km_to_meters.inverse());
        kilometers.push_edge(meters.get_id(), km_to_meters);

        yards.insert_into(&mut unit_ids);
        feet.insert_into(&mut unit_ids);
        meters.insert_into(&mut unit_ids);
        kilometers.insert_into(&mut unit_ids);
    }

    loop {
        let line = read_input("\nEnter your conversion");
        if line == String::from("quit\\") {
            break;
        }

        let (value, value_size) = extract_value(&line);
        let (unit_1, unit_1_size) = extract_unit(&line[value_size..]);
        let unit_2  = extract_unit(&line[value_size + unit_1_size..]).0;
    
        //println!("The conversion to complete is {} {} to {}", value, unit_1, unit_2);

        let unit_1 = aliases.get(&unit_1).expect("No alias exists for the given unit");
        let unit_2 = aliases.get(&unit_2).expect("No alias exists for the given unit");
        let unit_1 = unit_ids.get(unit_1).expect("No unit exists with the given id");
        let unit_2 = unit_ids.get(unit_2).expect("No unit exists with the given id");

        match convert(&value, unit_1, unit_2, &unit_ids, &generator) {
            None => println!("That conversion is impossible!"),
            Some((steps, answer)) => print_steps(value, unit_1, steps, answer, unit_2, &unit_ids)
        }
        println!()
    }
}

fn print_steps(initial_value: f64, starting_unit: &Unit, steps: Vec<Step>, answer: f64, final_unit: &Unit, unit_ids: &HashMap<usize, Unit>) {
    //println!("The final solution is as follows:");
    print!("{} {}", initial_value, starting_unit.get_name());
    for step in steps {
        step.print(unit_ids);
    }
    print!("{} {}", answer, final_unit.get_name());
}

fn convert(value: &f64, 
        start: &Unit, 
        destination: &Unit, 
        unit_ids: &HashMap<usize, Unit>, 
        generator: &IDGenerator) -> Option<(Vec<Step>, f64)>
    {
    let mut graph = Vec::new();
    for id in 0..=generator.max() {
        let mut new_node = Vec::new();
        for neighbor in unit_ids.get(&id).expect("The UnitIDs HashMap must have an entry for all ids generated").connected_ids() {
            new_node.push(*neighbor);
        }
        graph.push(new_node);
    }

    let path = match find_shortest_path(&graph, start.get_id(), destination.get_id()) {
        None => return None,
        Some(path) => path
    };

    let mut steps = Vec::<Step>::new();
    let mut running_answer = *value;

    for (index, this_id) in path.iter().enumerate() {
        let next_id = match path.get(index + 1) {
            // eventually we will be on the last id in the path and there is no next one so we break early
            None => break, 
            Some(next) => *next
        };
        let conversion = unit_ids.get(this_id)
            .expect("The UnitIDs HashMap must have an entry for all ids in the path")
            .convert(next_id)
            .expect("The path must go along units that can convert along the path");
        running_answer = conversion.apply(running_answer);
        steps.push(Step::of(conversion, *this_id, next_id));
    }

    Some((steps, running_answer))
}

fn bfs(graph: &Vec<Vec<usize>>, start: usize, parent: &mut Vec<usize>, distance: &mut Vec<usize>) {
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

fn find_shortest_path(graph: &Vec<Vec<usize>>, start: usize, destination: usize) -> Option<Vec<usize>> {
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

fn insert_aliases<'a>(aliases: &mut HashMap<String, usize>, unit_id: usize, news: Vec<&str>) {
    for n in news {
        aliases.insert(String::from(n), unit_id);
    }
}

fn extract_value(line: &str) -> (f64, usize) {
    fast_float::parse_partial::<f64, _>(line).expect("Line doesn't have a parseable value")
}

fn extract_unit(line: &str) -> (String, usize) {
    let mut unit: String = String::new();
    let mut size: usize = 0;
    for c in line.chars() {
        size += 1;
        if c == '\\' {
            return (unit.trim().to_string(), size);
        } else {
            unit.push(c);
        }
    }

    panic!("Unit needs a terminating \"\\\" (backslash)");
}

fn read_input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {},
        Err(_) => {}
    }
    input = input.trim().to_string();
    input.push('\\');
    input
}

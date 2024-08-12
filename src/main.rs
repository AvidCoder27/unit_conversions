mod graphing;
use graphing::{Conversion, IDGenerator, Step, Unit};
use std::{collections::{HashMap, VecDeque}, fs, io, path::Path};
use fast_float;

const NO_UNIT_ID_ERR: &str = "No unit exists with the given ID";

fn main() {
    let file_path = Path::new(r#".\conversions.txt"#);
    let mut generator: IDGenerator = IDGenerator::new();
    let mut unit_ids = HashMap::<usize, Unit>::new();
    let mut aliases = HashMap::<String, usize>::new();

    //_load_units_from_file(&mut generator, &mut aliases, &mut unit_ids, file_path);
    _manually_create_units(&mut generator, &mut aliases, &mut unit_ids);

    loop {
        let line = read_input("\nEnter your conversion");
        if line == String::from("quit:") {
            break;
        }
        if line == String::from("help:") {
            print_help_page();
            continue;
        }
        if line == String::from("list:") {
            print_all_units(&generator, &unit_ids);
            continue;
        }

        match line.chars().next() {
            None => panic!("Line must not be empty"),
            Some('#') => create_unit(&mut generator, &mut aliases, &mut unit_ids, line),
            Some('$') => create_conversion(&mut aliases, &mut unit_ids, line),
            _ => attempt_conversion(line, &aliases, &unit_ids, &generator)
        };
    }
}

fn create_unit(generator: &mut IDGenerator, aliases: &mut HashMap<String, usize>, unit_ids: &mut HashMap<usize, Unit>, line: String) {
    let mut names: Vec<String> = Vec::new();
    let mut word = String::new();
    let mut state: u8 = 0;
    let mut move_next_word_up = false;

    for c in line.chars() {
        match state {
            0 => {
                assert!(c == '#', "Cannot create unit from a line that does not begin with '#'");
                state = 1;
            },
            1 => {
                // waiting for a unit alias to begin
                if c.is_alphabetic() {
                    word.push(c);
                    state = 2;
                }
            },
            2.. => {
                // waiting for a unit alias to end
                match c {
                    '|' => {
                        names.push(word.clone());
                        move_next_word_up = true;
                    },
                    ',' => {
                        if move_next_word_up {
                            names.insert(names.len() - 1, word.clone());
                        } else {
                            names.push(word.clone());
                        }
                        word.clear();
                        state = 1;
                    },
                    ':' => {
                        break;
                    }
                    _ => {
                        word.push(c);
                    }
                };
            }
        };
    }
    if word.len() > 0 {
        names.push(word);
    }
    
    let name = names.first().expect("Unit definition must contain at least one alias");
    let unit = Unit::new(name, generator);

    for n in names.iter() {
        aliases.insert(n.to_string(), unit.get_id());
    }

    unit.insert_into(unit_ids);
    println!("Created new unit {}", name)
}

fn create_conversion(aliases: &mut HashMap<String, usize>, unit_ids: &mut HashMap<usize, Unit>, line: String) {
    let line = line.strip_prefix('$').expect("Command for creating conversion must begin with '$'").trim();
    let (value_1, size) = match fast_float::parse_partial(line) {
        Err(_) => (1.0, 0),
        Ok(thing) => thing
    };
    let line = &line[size..];
    let (unit_1, size) = extract_unit(line, '=').expect("Conversion must contain '=' to terminate first half");
    let line = &line.trim()[size..];
    let (value_2, size) = match fast_float::parse_partial(line) {
        Err(_) => (1.0, 0),
        Ok(thing) => thing
    };
    let line = &line[size..];
    let (unit_2, _) = extract_unit(line, ':').expect("Conversion must contain ':' to terminate second half");
    let one_to_two = Conversion::new(value_2, value_1);

    let unit_1 = aliases.get(&unit_1).expect("Units used in conversion must be aliased");
    let unit_2 = aliases.get(&unit_2).expect("Units used in conversion must be aliased");
    let mut unit_1 = unit_ids.remove(unit_1).expect("UnitIDs must have a definition for units used in conversions");
    let mut unit_2 = unit_ids.remove(unit_2).expect("UnitIDs must have a definition for units used in conversions");
    unit_2.push_edge(&unit_1, one_to_two.inverse());
    unit_1.push_edge(&unit_2, one_to_two);
    unit_1.insert_into(unit_ids);
    unit_2.insert_into(unit_ids);
}

fn _load_units_from_file(generator: &mut IDGenerator, aliases: &mut HashMap<String, usize>, unit_ids: &mut HashMap<usize, Unit>, file_path: &Path) {
    let file_contents = fs::read_to_string(file_path).expect("File read must not fail");
    println!("{file_contents}");
}

fn print_all_units(generator: &IDGenerator, unit_ids: &HashMap<usize, Unit>) {
    println!("All currently registered units:");
    for id in 0..=generator.max() {
        println!("\t{}", unit_ids.get(&id).expect("UnitIDs HashMap must have definitions for all ids generated").get_name());
    }
}

fn print_help_page() {
    println!("There are three types of commands that can be used:");
    println!("\t1. A conversion is denoted in the following form: [a value, like 1.3] [a unit, like meters] : [a unit to convert into]");
    println!("\t\tExample: 1.3 meter : feet");
    println!("\n\t2. You can register a new unit by typing a '#' and then all the aliases of the unit separated by commas.");
    println!("\t\tExample: # meter|s, m");
    println!("\t\tUsing '|s' at the end of an alias will register the singular and the plural form of the word.");
    println!("\n\t3. You can register a new conversion by typing a '$' and then an equation that states the conversion factor.");
    println!("\t\tExample: $ 1 meter = 100 cm");
    println!("\t\tYou can use any alias of a unit to define its conversion factor.");
    println!("\nThere are also a few single word commands:");
    println!("\t'help' will bring up this page.");
    println!("\t'list' will print out all the units currently registered.");
    println!("\t'quit' will quit out of the program.");
}

fn attempt_conversion(line: String, aliases: &HashMap<String, usize>, unit_ids: &HashMap<usize, Unit>, generator: &IDGenerator) {
    let (value, value_size) = match fast_float::parse_partial(&line) {
        Err(_)=> (1.0, 0),
        Ok(thing) => thing
    };
    let (unit_1, unit_1_size) = match extract_unit(&line[value_size..], ':') {
        None => {
            println!("That is not a valid conversion");
            return;
        },
        Some(thing) => thing
    };
    let (unit_2, _)  = match extract_unit(&line[value_size + unit_1_size..], ':') {
        None => {
            println!("That is not a valid conversion");
            return;
        },
        Some(thing) => thing
    };
    let unit_1 = match aliases.get(&unit_1) {
        None => {
            println!("Unit 1 ({}) is not a valid unit", unit_1);
            return;
        },
        Some(thing) => thing
    };
    let unit_2 = match aliases.get(&unit_2) {
        None => {
            println!("Unit 2 ({}) is not a valid unit", unit_2);
            return;
        },
        Some(thing) => thing
    };
    let unit_1 = unit_ids.get(unit_1).expect(NO_UNIT_ID_ERR);
    let unit_2 = unit_ids.get(unit_2).expect(NO_UNIT_ID_ERR);
    match convert(&value, unit_1, unit_2, unit_ids, generator) {
        None => print!("That conversion is impossible!"),
        Some((steps, answer)) => print_steps(value, unit_1, steps, answer, unit_2, unit_ids)
    }
    println!();
}

fn _manually_create_units(generator: &mut IDGenerator, aliases: &mut HashMap<String, usize>, unit_ids: &mut HashMap<usize, Unit>) {
    let mut yard = Unit::new("yards", generator);
    let mut foot = Unit::new("feet", generator);
    let mut centimeter = Unit::new("centimeters", generator);
    let mut meter = Unit::new("meters", generator);
    let mut kilometer = Unit::new("kilometers", generator);
    let mut inch = Unit::new("inches", generator);
    let mut mile = Unit::new("miles", generator);
    let mut second = Unit::new("seconds", generator);
    let mut millisecond = Unit::new("milliseconds", generator);
    let mut minute = Unit::new("minutes", generator);
    let mut hour = Unit::new("hours", generator);
    let mut day = Unit::new("days", generator);
    let mut week = Unit::new("weeks", generator);
    let mut calendar_year = Unit::new("calendar years", generator);

    insert_aliases(aliases, &calendar_year, vec!["calendar years", "calendar year"]);
    insert_aliases(aliases, &week, vec!["week", "weeks"]);
    insert_aliases(aliases, &day, vec!["days", "day"]);
    insert_aliases(aliases, &hour, vec!["hours", "hour", "hr", "hrs"]);
    insert_aliases(aliases, &minute, vec!["minute", "minutes", "min", "mins"]);
    insert_aliases(aliases, &millisecond, vec!["ms", "millisecond", "milliseconds"]);
    insert_aliases(aliases, &yard, vec!["yds", "yards", "yard", "yd"]);
    insert_aliases(aliases, &foot, vec!["foot", "feet", "ft"]);
    insert_aliases(aliases, &meter, vec!["m", "meters", "meter"]);
    insert_aliases(aliases, &kilometer, vec!["km", "kilometers", "kilometer"]);
    insert_aliases(aliases, &inch, vec!["in", "inch", "inches"]);
    insert_aliases(aliases, &centimeter, vec!["cm", "centimeter"]);
    insert_aliases(aliases, &mile, vec!["mi", "miles", "mile"]);
    insert_aliases(aliases, &second, vec!["seconds", "s", "sec", "secs", "second"]);

    let ms_to_seconds = Conversion::new(1.0, 1000.0);
    second.push_edge(&millisecond, ms_to_seconds.inverse());
    millisecond.push_edge(&second, ms_to_seconds);    

    let second_to_min = Conversion::new(1.0, 60.0);
    let minute_to_hour = second_to_min.clone();

    minute.push_edge(&second, second_to_min.inverse());
    second.push_edge(&minute, second_to_min);
    hour.push_edge(&minute, minute_to_hour.inverse());
    minute.push_edge(&hour, minute_to_hour);

    let hour_to_day = Conversion::new(1.0, 24.0);
    day.push_edge(&hour, hour_to_day.inverse());
    hour.push_edge(&day, hour_to_day);

    let day_to_week = Conversion::new(1.0, 7.0);
    week.push_edge(&day, day_to_week.inverse());
    day.push_edge(&week, day_to_week);

    let day_to_calendar_year = Conversion::new(1.0, 365.0);
    calendar_year.push_edge(&day, day_to_calendar_year.inverse());
    day.push_edge(&calendar_year, day_to_calendar_year);

    let yards_to_feet = Conversion::new(3.0, 1.0);
    foot.push_edge(&yard, yards_to_feet.inverse());
    yard.push_edge(&foot, yards_to_feet);

    let feet_to_meters = Conversion::new(1.0, 3.28084);
    meter.push_edge(&foot, feet_to_meters.inverse());
    foot.push_edge(&meter, feet_to_meters);

    let km_to_meters = Conversion::new(1000.0, 1.0);
    meter.push_edge(&kilometer, km_to_meters.inverse());
    kilometer.push_edge(&meter, km_to_meters);

    let inches_to_feet = Conversion::new(1.0, 12.0);
    foot.push_edge(&inch, inches_to_feet.inverse());
    inch.push_edge(&foot, inches_to_feet);

    let meters_to_centimeters = Conversion::new(100.0, 1.0);
    centimeter.push_edge(&meter, meters_to_centimeters.inverse());
    meter.push_edge(&centimeter, meters_to_centimeters);

    let feet_to_miles = Conversion::new(1.0, 5280.0);
    mile.push_edge(&foot, feet_to_miles.inverse());
    foot.push_edge(&mile, feet_to_miles);

    let inch_to_cm = Conversion::new(2.54, 1.0);
    centimeter.push_edge(&inch, inch_to_cm.inverse());
    inch.push_edge(&centimeter, inch_to_cm);

    let mile_to_km = Conversion::new(1.60934, 1.0);
    kilometer.push_edge(&mile, mile_to_km.inverse());
    mile.push_edge(&kilometer, mile_to_km);

    yard.insert_into(unit_ids);
    foot.insert_into(unit_ids);
    meter.insert_into(unit_ids);
    kilometer.insert_into(unit_ids);
    inch.insert_into(unit_ids);
    centimeter.insert_into(unit_ids);
    mile.insert_into(unit_ids);
    second.insert_into(unit_ids);
    millisecond.insert_into(unit_ids);
    minute.insert_into(unit_ids);
    hour.insert_into(unit_ids);
    day.insert_into(unit_ids);
    week.insert_into(unit_ids);
    calendar_year.insert_into(unit_ids);
}

fn print_steps(initial_value: f64, starting_unit: &Unit, steps: Vec<Step>, answer: f64, final_unit: &Unit, unit_ids: &HashMap<usize, Unit>) {
    print!("{} {}", initial_value, starting_unit.get_name());
    if steps.len() == 0 {
        print!(" is the same as ");
    }
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

fn insert_aliases(aliases: &mut HashMap<String, usize>, unit: &Unit, news: Vec<&str>) {
    for n in news {
        aliases.insert(String::from(n), unit.get_id());
    }
}

fn extract_unit(line: &str, termination_char: char) -> Option<(String, usize)> {
    let mut unit: String = String::new();
    let mut size: usize = 0;
    for c in line.chars() {
        size += 1;
        if c == termination_char {
            return Some((unit.trim().to_string(), size));
        } else {
            unit.push(c);
        }
    }

    None
}

fn read_input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {},
        Err(_) => {}
    }
    input = input.trim().to_string();
    input.push(':');
    input
}

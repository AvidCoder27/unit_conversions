mod structs;
mod algorithm;
use structs::{Conversion, IDGenerator, Step, Unit};
use std::{collections::HashMap, fs, io, path::Path};
use fast_float;

const ERR_ALIASED_ID_UNDEFINED: &str = "UnitIDs HashMap must have a definition for all aliased IDs";
const ERR_GENERATED_ID_UNDEFINED: &str = "UnitIDs HashMap must have definitions for all generated IDs";
const ERR_INVALID_CONVERSION: &str = "That is not a valid conversion";
const ERR_FILE_READ: &str = "File read must not fail";
const PREV_ANS_KEYWORD: &str = "ans";

fn main() {
    let conversions_file_path = Path::new(r#".\conversions.txt"#);
    let help_file_path = Path::new(r#".\help.txt"#);
    let mut generator: IDGenerator = IDGenerator::new();
    let mut unit_ids = HashMap::<usize, Unit>::new();
    let mut aliases = HashMap::<String, usize>::new();
    let mut previous_answer: Option<(usize, f64)> = None;

    load_units_from_file(&mut generator, &mut aliases, &mut unit_ids, conversions_file_path);

    loop {
        let line = read_input("\nEnter a command, or `help`:");
        if line == String::from("quit:") {
            break;
        }
        if line == String::from("help:") {
            print_help_page(help_file_path);
            continue;
        }
        if line == String::from("list:") {
            print_all_units(&generator, &unit_ids);
            continue;
        }
        if line == String::from("reload:") {
            unit_ids.clear();
            aliases.clear();
            generator.clear();
            previous_answer = None;
            load_units_from_file(&mut generator, &mut aliases, &mut unit_ids, conversions_file_path);
            println!("Reloaded!");
            continue;
        }

        match line.chars().next() {
            None => panic!("Line must not be empty"),
            Some('#') => create_unit(&mut generator, &mut aliases, &mut unit_ids, line, true),
            Some('$') => create_conversion(&mut aliases, &mut unit_ids, line, true),
            _ => match attempt_conversion(line, &aliases, &unit_ids, &generator, previous_answer) {
                None => {},
                Some(thing) => previous_answer = Some(thing)
            }
        };
    }
}

fn create_unit(
    generator: &mut IDGenerator, 
    aliases: &mut HashMap<String, usize>, 
    unit_ids: &mut HashMap<usize, Unit>, 
    line: String, 
    from_user: bool) 
    {
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
                        push_word_to_names(move_next_word_up, &mut names, &word);
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
        push_word_to_names(move_next_word_up, &mut names, &word);
    }
    
    let name = match names.first() {
        None => {
            println!("Unit definition must contain at least one alias");
            return;
        },
        Some(thing) => thing
    };
    let unit = Unit::new(name, generator);

    for n in names.iter() {
        aliases.insert(n.to_string(), unit.get_id());
    }

    unit.insert_into(unit_ids);
    if from_user {
        println!("Created new unit {}", name)
    }
}

fn push_word_to_names(move_next_word_up: bool, names: &mut Vec<String>, word: &String) {
    if move_next_word_up {
        names.insert(names.len() - 1, word.clone());
    } else {
        names.push(word.clone());
    }
}

fn create_conversion(aliases: &mut HashMap<String, usize>, unit_ids: &mut HashMap<usize, Unit>, line: String, from_user: bool) {
    let line = line.strip_prefix('$').expect("Command for creating conversion must begin with '$'").trim();
    let (value_1, size) = match fast_float::parse_partial(line) {
        Err(_) => (1.0, 0),
        Ok(thing) => thing
    };
    let line = &line[size..];
    let (unit_1, size) = match extract_unit(line, '=') {
        None => {
            println!("Conversion must contain '=' to demonstrate equality");
            return;
        },
        Some(thing) => thing
    };
    let line = &line.trim()[size..];
    let (value_2, size) = match fast_float::parse_partial(line) {
        Err(_) => (1.0, 0),
        Ok(thing) => thing
    };
    let line = &line[size..];
    let (unit_2, _) = extract_unit(line, ':').expect("Conversion must contain ':' to terminate second half");
    let one_to_two = Conversion::new(value_2, value_1);

    let unit_1 = match aliases.get(&unit_1) {
        None => {
            println!("The first unit in that conversion is not registered");
            return;
        },
        Some(thing) => thing
    };
    let unit_2 = match aliases.get(&unit_2) {
        None => {
            println!("The second unit in that conversion is not registered");
            return;
        },
        Some(thing) => thing
    };
    let mut unit_1 = unit_ids.remove(unit_1).expect(ERR_ALIASED_ID_UNDEFINED);
    let mut unit_2 = unit_ids.remove(unit_2).expect(ERR_ALIASED_ID_UNDEFINED);
    if from_user {
        println!("Created conversion between {} and {}", unit_1.get_name(), unit_2.get_name());
    }
    unit_2.push_edge(&unit_1, one_to_two.inverse());
    unit_1.push_edge(&unit_2, one_to_two);
    unit_1.insert_into(unit_ids);
    unit_2.insert_into(unit_ids);
}

fn load_units_from_file(
    mut generator: &mut IDGenerator, 
    mut aliases: &mut HashMap<String, usize>, 
    mut unit_ids: &mut HashMap<usize, Unit>, 
    file_path: &Path) 
    {
    let contents = fs::read_to_string(file_path).expect(ERR_FILE_READ);
    for line in contents.lines() {
        let mut line = line.to_string();
        line.push(':');
        match line.chars().next() {
            Some('#') => create_unit(&mut generator, &mut aliases, &mut unit_ids, line, false),
            Some('$') => create_conversion(&mut aliases, &mut unit_ids, line, false),
            _ => continue
        };
    }
}

fn print_help_page(file_path: &Path) {
    let contents = fs::read_to_string(file_path).expect(ERR_FILE_READ);
    print!("{contents}");
}

fn print_all_units(generator: &IDGenerator, unit_ids: &HashMap<usize, Unit>) {
    println!("All currently registered units:");
    for id in 0..generator.peek() {
        println!("\t{}", unit_ids.get(&id).expect(ERR_GENERATED_ID_UNDEFINED).get_name());
    }
}

fn attempt_conversion(
    line: String, 
    aliases: &HashMap<String, usize>, 
    unit_ids: &HashMap<usize, Unit>, 
    generator: &IDGenerator,
    previous_answer: Option<(usize, f64)>
    ) -> Option<(usize, f64)>
    {

    let (value, unit_1, unit_2) = match line.strip_prefix(PREV_ANS_KEYWORD) {
        None => {
            match extract_value_and_units(line) {
                Some(value) => value,
                None => return None,
            }
        },
        Some(line) => {
            let previous_answer = match previous_answer {
                None => {
                    println!("Cannot use `{}` because no previous answer exists", PREV_ANS_KEYWORD);
                    return None;
                },
                Some(answer) => answer
            };
            let unit_1 = unit_ids.get(&previous_answer.0).expect("Previous answer must contain a valid ID").get_name();
            let unit_1 = String::from(unit_1);

            let unit_2 = line.trim_start_matches(|c: char| c.is_whitespace() || c == ':');
            let unit_2 = match extract_unit(unit_2, ':') {
                None => {
                    println!("{ERR_INVALID_CONVERSION}");
                    return None;
                },
                Some((unit, _)) => unit
            };
            (previous_answer.1, unit_1, unit_2)
        }
    };

    let unit_1 = match aliases.get(&unit_1) {
        None => {
            println!("Unit 1 ({}) is not a valid unit", unit_1);
            return None;
        },
        Some(thing) => thing
    };
    let unit_2 = match aliases.get(&unit_2) {
        None => {
            println!("Unit 2 ({}) is not a valid unit", unit_2);
            return None;
        },
        Some(thing) => thing
    };
    let unit_1 = unit_ids.get(unit_1).expect(ERR_ALIASED_ID_UNDEFINED);
    let unit_2 = unit_ids.get(unit_2).expect(ERR_ALIASED_ID_UNDEFINED);
    return match convert(&value, unit_1, unit_2, unit_ids, generator) {
        None => {
            print!("That conversion is impossible!\n");
            None
        },
        Some((steps, answer)) => {
            print_steps(value, unit_1, steps, answer, unit_2, unit_ids);
            Some((unit_2.get_id(), answer))
        }
    }
}

fn extract_value_and_units(line: String) -> Option<(f64, String, String)> {
    let (value, size) = match fast_float::parse_partial(&line) {
        Err(_)=> (1.0, 0),
        Ok(thing) => thing
    };
    let line = &line[size..];
    let (unit_1, size) = match extract_unit(line, ':') {
        None => {
            println!("{ERR_INVALID_CONVERSION}");
            return None;
        },
        Some(thing) => thing
    };
    let line = &line[size..];
    let (unit_2, _)  = match extract_unit(line, ':') {
        None => {
            println!("{ERR_INVALID_CONVERSION}");
            return None;
        },
        Some(thing) => thing
    };
    Some((value, unit_1, unit_2))
}

fn print_steps(initial_value: f64, starting_unit: &Unit, steps: Vec<Step>, answer: f64, final_unit: &Unit, unit_ids: &HashMap<usize, Unit>) {
    let mut bottom = String::new();
    let mut middle = String::new();
    let mut top = String::new();
    
    let initial = format!("{} {} ", initial_value, starting_unit.get_name());
    let whitespace = " ".repeat(initial.len());
    top.push_str(whitespace.as_str());
    middle.push_str(initial.as_str());
    bottom.push_str(whitespace.as_str());

    if steps.len() == 0 {
        todo!()
    }

    for step in steps {

        top.push_str("⎧  ");
        middle.push_str("|--");
        bottom.push_str("⎩  ");

        let numer = step.get_top(unit_ids);
        let denom = step.get_bottom(unit_ids);
        let size = numer.len().max(denom.len());
        top.push_str(format!("{: ^size$}", numer).as_str());
        middle.push_str("-".repeat(size).as_str());
        bottom.push_str(format!("{: ^size$}", denom).as_str());

        top.push_str("  ⎫");
        middle.push_str("--|");
        bottom.push_str("  ⎭");
    }

    middle.push_str(format!(" = {} {}", answer, final_unit.get_name()).as_str());

    println!("\n{top}");
    println!("{middle}");
    println!("{bottom}\n");
}

fn convert(value: &f64, 
        start: &Unit, 
        destination: &Unit, 
        unit_ids: &HashMap<usize, Unit>, 
        generator: &IDGenerator) -> Option<(Vec<Step>, f64)>
    {
    let mut graph = Vec::new();
    for id in 0..generator.peek() {
        let mut new_node = Vec::new();
        for neighbor in unit_ids.get(&id).expect(ERR_GENERATED_ID_UNDEFINED).connected_ids() {
            new_node.push(*neighbor);
        }
        graph.push(new_node);
    }

    let path = match algorithm::find_shortest_path(&graph, start.get_id(), destination.get_id()) {
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
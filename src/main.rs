mod structs;
mod algorithm;
use structs::{Conversion, IDGenerator, Step, Unit};
use std::{collections::{HashMap, HashSet}, fs, io, path::Path};
use fast_float;
use unicode_segmentation::UnicodeSegmentation;

const ERR_ID_UNDEFINED: &str = "UnitIDs HashMap is missing a definition for an ID";
const ERR_FILE_READ: &str = "File read must not fail";

fn main() {
    let conversions_file_path = Path::new(r#"./conversions.txt"#);
    let help_file_path = Path::new(r#"./help.txt"#);
    let mut generator: IDGenerator = IDGenerator::new();
    let mut unit_ids = HashMap::<usize, Unit>::new();
    let mut aliases = HashMap::<String, usize>::new();

    load_units_from_file(&mut generator, &mut aliases, &mut unit_ids, conversions_file_path);
    main_loop(help_file_path, generator, unit_ids, aliases, conversions_file_path);
}

fn main_loop(
    help_file_path: &Path,
    mut generator: IDGenerator,
    mut unit_ids: HashMap<usize, Unit>,
    mut aliases: HashMap<String, usize>,
    conversions_file_path: &Path)
{
    let mut previous_answer: Option<String> = None;
    loop {
        let line = read_input("\nEnter a command, or `help`:");
        if line == String::from("quit;") {
            break;
        }
        if line == String::from("help;") {
            print_help_page(help_file_path);
            continue;
        }
        if line == String::from("list;") {
            print_all_units(&generator, &unit_ids);
            continue;
        }
        if line == String::from("reload;") {
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
            _ => attempt_conversion(line, &aliases, &unit_ids, &generator, &mut previous_answer)
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
    fn push_word_to_names(move_next_word_up: bool, names: &mut Vec<String>, word: &String) {
        if move_next_word_up {
            names.insert(names.len() - 1, word.clone());
        } else {
            names.push(word.clone());
        }
    }

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
                    ';' => {
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

fn create_conversion(aliases: &mut HashMap<String, usize>, unit_ids: &mut HashMap<usize, Unit>, line: String, from_user: bool) {
    let line = line.strip_prefix('$').expect("Command for creating conversion must begin with '$'").trim();
    let (value_1, size) = match fast_float::parse_partial(line) {
        Err(_) => (1.0, 0),
        Ok(thing) => thing
    };
    let line = &line[size..];
    let (unit_1, size, _) = match extract_unit(line, &HashSet::from(['='])) {
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
    let (unit_2, _, _) = extract_unit(line, &HashSet::from([';'])).expect("Conversion must contain ';' to terminate second half");
    let one_to_two = Conversion::new(value_2, value_1);

    let unit_1 = match aliases.get(&unit_1) {
        None => {
            println!("The first unit ({}) in that conversion is not registered", unit_1);
            return;
        },
        Some(thing) => thing
    };
    let unit_2 = match aliases.get(&unit_2) {
        None => {
            println!("The second unit ({}) in that conversion is not registered", unit_2);
            return;
        },
        Some(thing) => thing
    };
    let mut unit_1 = unit_ids.remove(unit_1).expect(ERR_ID_UNDEFINED);
    let mut unit_2 = unit_ids.remove(unit_2).expect(ERR_ID_UNDEFINED);
    if from_user {
        println!("Created conversion between {} and {}", unit_1.get_name(), unit_2.get_name());
    }
    unit_2.push_edge(&unit_1, one_to_two.inverse());
    unit_1.push_edge(&unit_2, one_to_two);
    unit_1.insert_into(unit_ids);
    unit_2.insert_into(unit_ids);
}

fn attempt_conversion(
    line: String, 
    aliases: &HashMap<String, usize>, 
    unit_ids: &HashMap<usize, Unit>, 
    generator: &IDGenerator,
    previous_answer: &mut Option<String>)
{
    let line = if let Some(stripped) = line.strip_prefix("ans") {
        match previous_answer {
            Some(previous_answer) => {
                let mut line = previous_answer.clone();
                line.push_str(stripped);
                line
            },
            None => {
                println!("Cannot use 'ans': no previous answer");
                return;
            }
        }
    } else {
        line
    };

    let (value,starting_numers,starting_denoms,ending_numers,ending_denoms) =
    match extract_value_and_units(line, aliases) {
        None => return,
        Some(thing) => thing
    };

    match convert_multiple(unit_ids, generator, &value, &starting_numers, &starting_denoms, &ending_numers, &ending_denoms) {
        None => print!("That conversion is impossible!\n"),
        Some((steps, answer)) => {
            print_steps(unit_ids, value, answer, steps, &starting_numers, &starting_denoms, &ending_numers, &ending_denoms);
            previous_answer.replace(convert_quantity_to_string(unit_ids, answer, &ending_numers, &ending_denoms));
        }
    }
}

fn extract_value_and_units(line: String, aliases: &HashMap<String, usize>
) -> Option<(f64, Vec<usize>, Vec<usize>, Vec<usize>, Vec<usize>)> {
    let mut running_value = 1f64;
    let mut size = 0;
    let mut starting_numers = Vec::new();
    let mut starting_denoms = Vec::new();
    let mut ending_numers = Vec::new();
    let mut ending_denoms = Vec::new();
    let mut previous_terminator = '*';
    let mut switched_to_end = false;
    loop {
        let line = line[size..].trim();
        // extract a value before the unit if it is there
        let (next_value, value_size) = match fast_float::parse_partial(&line) {
            Err(_) => (1f64, 0),
            Ok(thing) => {
                if switched_to_end {
                    println!("Invalid Conversion: Improper placement of number after the separating ':'");
                    return None;
                }
                thing
            }
        };
        let line = line[value_size..].trim();
        let (unit, unit_size, next_terminator) = 
        match extract_unit(line, &HashSet::from([';', ':', '*', '/'])) {
            None => break,
            Some(thing) => thing
        };
        if unit.len() > 0 {
            if !process_and_push_unit(unit, aliases, previous_terminator, &mut switched_to_end, &mut starting_numers, &mut ending_numers, &mut starting_denoms, &mut ending_denoms) {
                return None;
            }
        }
        if next_value != 1f64 {
            running_value *= match previous_terminator {
                '*' => next_value,
                '/' => next_value.recip(),
                _ => panic!("Previous Terminator must be '*' or '/' when updating running_value")
            }
        }
        size += unit_size + value_size;
        previous_terminator = next_terminator;
    }
        
    Some((running_value, starting_numers, starting_denoms, ending_numers, ending_denoms))
}

fn process_and_push_unit(
    unit: String,
    aliases: &HashMap<String, usize>,
    previous_terminator: char,
    switched_to_end: &mut bool,
    starting_numers: &mut Vec<usize>,
    ending_numers: &mut Vec<usize>,
    starting_denoms: &mut Vec<usize>,
    ending_denoms: &mut Vec<usize>
) -> bool {
    let (unit, exponent) = if let Some((prefix, suffix)) = unit.split_once('^') {
        match i32::from_str_radix(suffix.trim(), 10) {
            Ok(exponent) => (prefix.to_string(), exponent),
            Err(error) => {
                println!("Invalid Conversion: Improper use of exponent, {}", error);
                return false;
            }
        }
    } else {
        (unit, 1)
    };
    let id = match aliases.get(unit.as_str()) {
        None => {
            println!("Invalid Conversion: Unit '{}' is not registered.", unit);
            return false;
        },
        Some(id) => *id
    };
    let chosen_vec = match previous_terminator {
        '*' => {
            match *switched_to_end {
                false => starting_numers, 
                true => ending_numers,
            }
        },
        '/' => {
            match *switched_to_end {
                false => starting_denoms,
                true  => ending_denoms,
            }
        },
        ':' => {
            *switched_to_end = true;
            ending_numers
        }
        _ => panic!("Previous terminator must be '*', '/', or ':'")
    };
    for _ in 0..exponent {
        chosen_vec.push(id);
    }
    true
}

fn convert_quantity_to_string(unit_ids: &HashMap<usize, Unit>, value: f64, numers: &Vec<usize>, denoms: &Vec<usize>) -> String {
    let mut numer_iter = numers.iter();
    let numer = numer_iter.next().expect("Quantity must have at least one numerator unit");
    let numer = unit_ids.get(numer).expect(ERR_ID_UNDEFINED);

    let mut s = value.to_string();
    s.push(' ');
    s.push_str(numer.get_name());
    for numer in numer_iter {
        let numer = unit_ids.get(numer).expect(ERR_ID_UNDEFINED);
        s.push_str(format!(" * {}", numer.get_name()).as_str());
    }
    for denom in denoms {
        let denom = unit_ids.get(denom).expect(ERR_ID_UNDEFINED);
        s.push_str(format!(" / {}", denom.get_name()).as_str());
    }
    s
}

fn convert_multiple(
    unit_ids: &HashMap<usize, Unit>,
    generator: &IDGenerator,
    value: &f64,
    starting_numers: &Vec<usize>,
    starting_denoms: &Vec<usize>,
    ending_numers: &Vec<usize>,
    ending_denoms: &Vec<usize>
)-> Option<(Vec<Step>, f64)> {
    if starting_numers.len() != ending_numers.len() {
        print!("Starting and ending numerators must be equal in length! ");
        return None;
    }
    if starting_denoms.len() != ending_denoms.len() {
        print!("Starting and ending denominators must be equal in length! ");
        return None;
    }

    let graph = generate_graph(generator, unit_ids);
    let mut steps = Vec::<Step>::new();
    let mut running_answer = *value;

    for path in algorithm::find_paths_between(starting_numers, ending_numers, &graph) {
        add_steps(path, unit_ids, &mut running_answer, &mut steps, false);
    }
    for path in algorithm::find_paths_between(starting_denoms, ending_denoms, &graph) {
        add_steps(path, unit_ids, &mut running_answer, &mut steps, true);
    }

    match steps.len() {
        0 => None,
        1.. => Some((steps, running_answer))
    }
}

fn generate_graph(generator: &IDGenerator, unit_ids: &HashMap<usize, Unit>) -> Vec<Vec<usize>> {
    let mut graph = Vec::new();
    for id in 0..generator.peek() {
        let mut new_node = Vec::new();
        for neighbor in unit_ids.get(&id).expect(ERR_ID_UNDEFINED).connected_ids() {
            new_node.push(*neighbor);
        }
        graph.push(new_node);
    }
    graph
}

fn add_steps(path: Vec<usize>, unit_ids: &HashMap<usize, Unit>, running_answer: &mut f64, steps: &mut Vec<Step>, inverse: bool) {
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
        
        if inverse {
            let inverse = &conversion.inverse();
            inverse.apply(running_answer);
            steps.push(Step::of(inverse,next_id, *this_id));
        } else {
            conversion.apply(running_answer);
            steps.push(Step::of(conversion, *this_id, next_id));
        }
    }
}

fn print_steps(unit_ids: &HashMap<usize, Unit>, 
    initial_value: f64, 
    answer: f64, 
    steps: Vec<Step>, 
    starting_numers: &Vec<usize>,
    starting_denoms: &Vec<usize>,
    ending_numers: &Vec<usize>,
    ending_denoms: &Vec<usize>)
{
    fn convert_ids_to_string(ids: &Vec<usize>, unit_ids: &HashMap<usize, Unit>) -> String {
        let mut iter = ids.iter();
        let mut s = String::from(
            unit_ids.get(iter.next()
            .expect("Must have at least one unit in the numerator"))
            .expect(ERR_ID_UNDEFINED).get_name());
    
        for id in iter {
            s.push_str(" × ");
            s.push_str(unit_ids.get(id).expect(ERR_ID_UNDEFINED).get_name());
        }
        s
    }
    
    fn push_fraction (top: &mut String, middle: &mut String, bottom: &mut String, numer: String, denom: String) {
        top   .push_str("⎧ ");
        middle.push_str("⎪⎻");
        bottom.push_str("⎩ ");
            
        let size = numer.len().max(denom.len());
        top.push_str(format!("{: ^size$}", numer).as_str());
        middle.push_str("⎻".repeat(size).as_str());
        bottom.push_str(format!("{: ^size$}", denom).as_str());
    
        top   .push_str(" ⎫");
        middle.push_str("⎻⎪");
        bottom.push_str(" ⎭");
    }
    
    let mut bottom = String::new();
    let mut middle = String::new();
    let mut top = String::new();

    let numer = format!(
        //"{0:.3e} {1}", 
        "{} {}",
        initial_value, convert_ids_to_string(starting_numers, unit_ids));
    if starting_denoms.len() == 0 {
        let whitespace = " ".repeat(numer.graphemes(true).count());
        top.push_str(whitespace.as_str());
        middle.push_str(numer.as_str());
        bottom.push_str(whitespace.as_str());
    } else {
        let denom = convert_ids_to_string(starting_denoms, unit_ids);
        push_fraction(&mut top, &mut middle, &mut bottom, numer, denom);
    }

    for step in steps {
        let numer = step.get_top(unit_ids);
        let denom = step.get_bottom(unit_ids);
        push_fraction(&mut top, &mut middle, &mut bottom, numer, denom);
    }

    top.push_str("   ");
    middle.push_str(" = ");
    bottom.push_str("   ");

    let numer = format!(
        //"{0:.3e} {1}", 
        "{} {}",
        answer, convert_ids_to_string(ending_numers, unit_ids));
    if ending_denoms.len() == 0 {
        // let whitespace = " ".repeat(numer.len());
        // top.push_str(whitespace.as_str());
        middle.push_str(numer.as_str());
        // bottom.push_str(whitespace.as_str());
    } else {
        let denom = convert_ids_to_string(ending_denoms, unit_ids);
        push_fraction(&mut top, &mut middle, &mut bottom, numer, denom);
    }

    println!("\n{top}");
    println!("{middle}");
    println!("{bottom}\n");
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
        line.push(';');
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
        println!("\t{}: {}", id, unit_ids.get(&id).expect(ERR_ID_UNDEFINED).get_name());
    }
}

fn extract_unit(line: &str, termination_chars: &HashSet<char>) -> Option<(String, usize, char)> {
    let mut unit: String = String::new();
    let mut size: usize = 0;
    for c in line.chars() {
        size += 1;
        if termination_chars.contains(&c) {
            return Some((unit.trim().to_string(), size, c));
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
    input.push(';');
    input
}

mod structs;
mod algorithm;
use structs::{Conversion, Element, IDGenerator, Step, Unit};
use std::{collections::{HashMap, HashSet}, fs, io, path::Path};
use fast_float;
use unicode_segmentation::UnicodeSegmentation;

const ERR_ID_UNDEFINED: &str = "UnitIDs HashMap is missing a definition for an ID";
const ERR_FILE_READ: &str = "File read must not fail";
const AVAGADROS_CONSTANT: f64 = 6.02214076e23;

fn main() {
    let help_file_path = Path::new(r#"./help.txt"#);
    let conversions_file_path = Path::new(r#"./conversions.txt"#);
    let elements_file_path = Path::new(r#"./elements.txt"#);
    let mut units_generator: IDGenerator = IDGenerator::new(0);
    let mut elements_generator: IDGenerator = IDGenerator::new(1);
    let mut unit_ids = HashMap::<usize, Unit>::new();
    let mut unit_aliases = HashMap::<String, usize>::new();
    let mut element_ids = HashMap::<usize, Element>::new();
    let mut element_aliases = HashMap::<String, usize>::new();
    
    load_units_from_file(&mut units_generator, &mut unit_aliases, &mut unit_ids, conversions_file_path);
    load_elements_from_file(&mut elements_generator, &mut element_aliases, &mut element_ids, elements_file_path);

    let mut previous_answer: Option<String> = None;
    loop {
        let line = read_input("\nEnter a command, or `help`:");
        if line.eq("quit;") {
            break;
        }
        if line.eq("help;") {
            println!("{}", fs::read_to_string(help_file_path).expect(ERR_FILE_READ));
            continue;
        }
        if line.eq("list;") {
            println!("All currently registered units:");
            for id in 0..units_generator.peek() {
                println!("\t{}: {}", id, unit_ids.get(&id).expect(ERR_ID_UNDEFINED).get_name());
            }
            continue;
        }
        if line.eq("reload;") {
            units_generator.clear();
            elements_generator.clear();
            unit_ids.clear();
            unit_aliases.clear();
            element_ids.clear();
            element_aliases.clear();
            previous_answer = None;
            load_units_from_file(&mut units_generator, &mut unit_aliases, &mut unit_ids, conversions_file_path);
            load_elements_from_file(&mut elements_generator, &mut unit_aliases, &mut element_ids, elements_file_path);
            println!("Reloaded!");
            continue;
        }

        match line.chars().next() {
            None => panic!("Line must not be empty"),
            Some('#') => create_unit(&mut units_generator, &mut unit_aliases, &mut unit_ids, line, true),
            Some('$') => create_conversion(&mut unit_aliases, &mut unit_ids, line, true),
            _ => attempt_conversion(line, &unit_aliases, &element_aliases, &element_ids, &mut unit_ids, &mut units_generator, &mut previous_answer)
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
        if c.is_ascii_digit() { 
            if from_user {
                println!("Cannot create unit with a digit ({c}) in its name");
            } else {
                panic!("Cannot create unit with a digit ({c}) in its name");
            }
        }
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
    let unit = Unit::new(name.clone(), generator);

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
    unit_aliases: &HashMap<String, usize>, 
    element_aliases: &HashMap<String, usize>,
    element_ids: &HashMap<usize, Element>,
    unit_ids: &mut HashMap<usize, Unit>,
    generator: &mut IDGenerator,
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
    let (line, chemical) = try_extract_chemical(line);
    let elements = match chemical {
        Some(chemical) => Some(extract_elements(chemical.as_str(), element_aliases)),
        None => None,
    };
    let (value,starting_numers,starting_denoms,ending_numers,ending_denoms) =
    match extract_value_and_units(line, unit_aliases) {
        None => return,
        Some(thing) => thing
    };
    match convert_multiple(unit_ids, generator, element_ids, &value, &starting_numers, &starting_denoms, &ending_numers, &ending_denoms, elements) {
        None => print!("That conversion is impossible!\n"),
        Some((steps, answer)) => {
            print_steps(unit_ids, value, answer, steps, &starting_numers, &starting_denoms, &ending_numers, &ending_denoms);
            previous_answer.replace(convert_quantity_to_string(unit_ids, answer, &ending_numers, &ending_denoms));
        }
    }
}

/// Returns the content within braces at the beggining of the line.
/// The first string is the line without the chemical and the second string
/// is the chemical without braces
fn try_extract_chemical(line: String) -> (String, Option<String>) {
    match line.split_once('[') {
        None => (line, None),
        Some((prefix, suffix)) => {
            if let Some((presuffix, sufsuffix)) = suffix.split_once(']') {
                let mut line = prefix.to_string();
                line.push_str(sufsuffix);
                (line, Some(presuffix.to_string()))
            } else {
                println!("Opening brace without closing brace!");
                (line, None)
            }
        }
    }
}

fn extract_value_and_units(line: String, unit_aliases: &HashMap<String, usize>) -> Option<(f64, Vec<usize>, Vec<usize>, Vec<usize>, Vec<usize>)> {
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
        // this if,loop,if is the best way to ensure that there are no numbers in the second half of the expression
        if switched_to_end || previous_terminator == ':'{
            for c in line.chars() {
                if c.is_ascii_digit() {
                    println!("Invalid Conversion: Improper placement of number after the separating ':'");
                    return None;
                }
            }
        }
        // extract a value before the unit if it is there
        let (next_value, value_size) = match fast_float::parse_partial(&line) {
            Err(_) => (1f64, 0),
            Ok(thing) => thing
        };
        let line = line[value_size..].trim();
        let (unit, unit_size, next_terminator) = match extract_unit(line, &HashSet::from([';', ':', '*', '/'])) {
            None => break,
            Some(thing) => thing
        };
        if next_value != 1f64 {
            running_value *= match previous_terminator {
                '*' => next_value,
                '/' => next_value.recip(),
                _ => panic!("Previous Terminator ({}) must be '*' or '/' when updating running_value", previous_terminator)
            }
        }
        if unit.len() > 0 {
            if !process_and_push_unit(unit, unit_aliases, previous_terminator, &mut switched_to_end, &mut starting_numers, &mut ending_numers, &mut starting_denoms, &mut ending_denoms) {
                return None
            }
        }
        size += unit_size + value_size;
        previous_terminator = next_terminator;
    }
    Some((running_value, starting_numers, starting_denoms, ending_numers, ending_denoms))
}

fn process_and_push_unit(
    unit: String,
    unit_aliases: &HashMap<String, usize>,
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
    let id = match unit_aliases.get(unit.as_str()) {
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
        _ => panic!("Previous terminator ({}) must be '*', '/', or ':'", previous_terminator)
    };
    for _ in 0..exponent {
        chosen_vec.push(id);
    }
    return true
}

fn extract_elements(chemical: &str, aliases: &HashMap<String, usize>) -> Vec<(usize, u16)>{
    fn finish_current(current_elem: &mut String, current_num: &mut String, elements: &mut Vec<(usize, u16)>, aliases: &HashMap<String, usize>) {
        if current_elem.len() > 0 {
            let subscript = if current_num.is_empty() {
                1
            } else {
                match u16::from_str_radix(current_num.as_str(), 10) {
                    Ok(subscript) => subscript,
                    Err(err) => panic!("Error parsing num when extracting elements: {err}")
                }
            };
            elements.push((*aliases.get(current_elem.as_str()).expect("Invalid element"), subscript));
            current_elem.clear();
            current_num.clear();
        }
    }
    let mut elements = Vec::new();
    let mut current_elem = String::new();
    let mut current_num = String::new();
    for ch in chemical.chars() {
        if ch.is_ascii_uppercase() {
            finish_current(&mut current_elem, &mut current_num, &mut elements, aliases);
            current_elem.push(ch);
        } else if ch.is_ascii_lowercase() {
            current_elem.push(ch);
        } else if ch.is_ascii_digit() {
            current_num.push(ch);
        }
    }
    finish_current(&mut current_elem, &mut current_num, &mut elements, aliases);
    elements
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
    unit_ids: &mut HashMap<usize, Unit>,
    generator: &mut IDGenerator,
    element_ids: &HashMap<usize, Element>,
    value: &f64,
    starting_numers: &Vec<usize>,
    starting_denoms: &Vec<usize>,
    ending_numers: &Vec<usize>,
    ending_denoms: &Vec<usize>,
    elements: Option<Vec<(usize, u16)>>
)-> Option<(Vec<Step>, f64)> {
    if starting_numers.len() != ending_numers.len() {
        print!("Starting and ending numerators must be equal in length! ");
        return None;
    }
    if starting_denoms.len() != ending_denoms.len() {
        print!("Starting and ending denominators must be equal in length! ");
        return None;
    }
    insert_elements(generator, unit_ids, element_ids, elements);
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

fn insert_elements(generator: &mut IDGenerator, unit_ids: &mut HashMap<usize, Unit>, element_ids: &HashMap<usize, Element>, elements: Option<Vec<(usize, u16)>>) {
    let elements = match elements {
        Some(elements) => elements,
        None => return
    };
    let (molar_mass, name) = find_mm_and_name(&elements, element_ids).unwrap();
    let mut moles_name = String::from("moles [");
    moles_name.push_str(name.as_str());
    moles_name.push(']');
    let mut grams_name = String::from("grams [");
    grams_name.push_str(name.as_str());
    grams_name.push(']');
    let mut parts_name = String::from("particles [");
    parts_name.push_str(name.as_str());
    parts_name.push(']');
    let mut moles = Unit::new(moles_name, generator);
    let mut grams = Unit::new(grams_name, generator);
    let mut particles = Unit::new(parts_name, generator);
    let moles_to_grams = Conversion::new(molar_mass, 1f64);
    let moles_to_particles = Conversion::new(AVAGADROS_CONSTANT, 1f64);
    grams.push_edge(&moles, moles_to_grams.inverse());
    particles.push_edge(&moles, moles_to_particles.inverse());
    moles.push_edge(&grams, moles_to_grams);
    moles.push_edge(&particles, moles_to_particles);
    moles.insert_into(unit_ids);
    grams.insert_into(unit_ids);
    particles.insert_into(unit_ids);
}

fn find_mm_and_name(elements: &Vec<(usize, u16)>, element_ids: &HashMap<usize, Element>) -> Result<(f64, String), String>{
    let mut molar_mass = 0f64;
    let mut name = String::new();
    for (atomic_number, count) in elements {
        match element_ids.get(atomic_number) {
            None => return Err(format!("Atomic number {} is undefined in the given `element_ids`", atomic_number)),
            Some(element) => {
                molar_mass += element.molar_mass * f64::from(*count);
                name.push_str(element.symbol.as_str());
                name.push_str(subscript_number(*count).as_str())
            }
        };
    }
    Ok((molar_mass, name))
}

fn subscript_number(num: u16) -> String {
    let mut subscript = String::new();
    for char in num.to_string().chars() {
        subscript.push(match char {
            '0' => '₀',
            '1' => '₁',
            '2' => '₂',
            '3' => '₃',
            '4' => '₄',
            '5' => '₅',
            '6' => '₆',
            '7' => '₇',
            '8' => '₈',
            '9' => '₉',
            _ => panic!("All chars must be ascii digits when creating subscript")
        })
    }
    subscript
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

fn load_elements_from_file(generator: &mut IDGenerator, aliases: &mut HashMap<String, usize>, element_ids: &mut HashMap<usize, Element>, file_path: &Path) {
    let contents = fs::read_to_string(file_path).expect(ERR_FILE_READ);
    for line in contents.lines() {
        let mut line = line.trim().to_string();
        line.push(';');
        if line.len() > 0 {
            create_element(generator, aliases, element_ids, line);
        }
    }
}

fn create_element(generator: &mut IDGenerator, aliases: &mut HashMap<String, usize>, element_ids: &mut HashMap<usize, Element>, line: String) {
    let mut line = line.as_str();
    let mut new_aliases = Vec::new();
    loop {
        if let Some((alias, length, terminator)) = extract_unit(line, &HashSet::from([',', '='])) {
            line = &line[length..].trim();
            new_aliases.push(alias);
            if terminator == '=' {
                break
            }
        } else {
            panic!("Element file line must have an equals sign")
        }
    }
    let molar_mass: f64 = match fast_float::parse_partial(line) {
        Ok((molar_mass, _)) => molar_mass,
        Err(err) => panic!("new element line must have a valid number after the equals sign: fast_float says {err}")
    };
    let atomic_number = generator.next();
    let element = Element::new(new_aliases[0].clone(), atomic_number, molar_mass);
    element_ids.insert(atomic_number, element);
    for alias in new_aliases {
        aliases.insert(alias, atomic_number);
    }
}

fn load_units_from_file(
    generator: &mut IDGenerator, 
    aliases: &mut HashMap<String, usize>, 
    unit_ids: &mut HashMap<usize, Unit>, 
    file_path: &Path) 
{
    let contents = fs::read_to_string(file_path).expect(ERR_FILE_READ);
    for line in contents.lines() {
        let mut line = line.to_string();
        line.push(';');
        match line.chars().next() {
            Some('#') => create_unit(generator, aliases, unit_ids, line, false),
            Some('$') => create_conversion(aliases, unit_ids, line, false),
            _ => continue
        };
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

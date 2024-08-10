use std::io;
use fast_float;

fn main() {
    loop {
        let line = read_input("Enter your conversion");

        let (value, value_size) = extract_value(&line);
        let (unit_1, unit_1_size) = extract_unit(&line[value_size..]);
        let unit_2  = extract_unit(&line[value_size + unit_1_size..]).0;
    
        println!("The conversion to complete is {} {} to {}", value, unit_1, unit_2);
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

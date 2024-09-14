use std::collections::HashMap;
use std::collections::hash_map::Keys;
use std::hash::{Hash, Hasher};
use std::ops::{DivAssign, MulAssign};

#[derive(Clone)]
pub struct Conversion {
    numerator: f64,
    denominator: f64,
}

pub struct Unit {
    name: String,
    id: usize,
    edges: HashMap<usize, Conversion>,
}

pub struct Step {
    top_value: f64,
    top_id: usize,
    bottom_value: f64,
    bottom_id: usize,
}

pub struct IDGenerator {
    id: usize
}

impl Step {

    const ERROR: &'static str = "UnitIds HashMap must define the ids that the step contains";

    pub fn of(conversion: &Conversion, from_id: usize, to_id: usize) -> Self {
        Step{
            top_value: conversion.numerator,
            bottom_value: conversion.denominator,
            top_id: to_id,
            bottom_id: from_id
        }
    }

    pub fn get_top(&self, unit_ids: &HashMap<usize, Unit>) -> String {
        format!("{} {}", self.top_value, unit_ids.get(&self.top_id).expect(Self::ERROR).get_name())
    }

    pub fn get_bottom(&self, unit_ids: &HashMap<usize, Unit>) -> String {
        format!("{} {}", self.bottom_value, unit_ids.get(&self.bottom_id).expect(Self::ERROR).get_name())
    }
}

impl IDGenerator {
    pub fn new(initial_id: usize) -> Self {
        IDGenerator {
            id: initial_id
        }
    }

    /// Returns a new, unique id that is 1 greater than the previous id
    pub fn next(&mut self) -> usize {
        self.id += 1;
        self.id - 1
    }

    /// Returns the id that WOULD be next distributed if next() was called
    pub fn peek(&self) -> usize {
        self.id
    }

    pub fn clear(&mut self) {
        self.id = 0;
    }
}

impl Conversion {
    pub fn new(numerator: f64, denominator: f64) -> Self {
        Conversion {
            numerator,
            denominator,
        }
    }

    pub fn apply(&self, value: &mut f64) {
        value.mul_assign(self.numerator);
        value.div_assign(self.denominator);
    }

    pub fn inverse(&self) -> Conversion {
        Conversion {
            numerator: self.denominator,
            denominator: self.numerator
        }
    }
}

impl Unit {
    pub fn new(name: String, gen: &mut IDGenerator) -> Self {
        Unit {
            name,
            id: gen.next(),
            edges: HashMap::new()
        }
    }

    pub fn push_edge(&mut self, other: &Self, conversion: Conversion) {
        self.edges.insert(other.get_id(), conversion);
    }

    pub fn connected_ids(&self) -> Keys<usize, Conversion> {
        self.edges.keys()
    }

    pub fn convert(&self, other_id: usize) -> Option<&Conversion> {
        self.edges.get(&other_id)
    }

    pub fn insert_into(self, graph: &mut HashMap<usize, Unit>) {
        graph.insert(self.id, self);
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
}

impl Hash for Unit {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Unit {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }

    fn ne(&self, other: &Self) -> bool {
        self.name.ne(&other.name)
    }
}

impl Eq for Unit { }

pub struct Element {
    pub symbol: String,
    pub atomic_number: usize,
    pub molar_mass: f64
}

impl Element {
    pub fn new(symbol: String, atomic_number: usize, molar_mass: f64) -> Self {
        Element { symbol, atomic_number, molar_mass }
    }
}
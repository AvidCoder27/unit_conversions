use std::collections::HashMap;
use std::collections::hash_map::Keys;
use std::hash::{Hash, Hasher};

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
    bottom_id: usize
}

pub struct IDGenerator {
    id: usize
}

impl Step {
    pub fn of(conversion: &Conversion, from_id: usize, to_id: usize) -> Self {
        Step{
            top_value: conversion.numerator,
            bottom_value: conversion.denominator,
            top_id: to_id,
            bottom_id: from_id
        }
    }
}

impl IDGenerator {
    pub fn new() -> Self {
        IDGenerator {
            id: 0
        }
    }

    pub fn next(&mut self) -> usize {
        self.id += 1;
        self.id
    }

    pub fn max(&self) -> usize {
        self.id
    }
}

impl Conversion {
    pub fn new(numerator: f64, denominator: f64) -> Self {
        Conversion {
            numerator,
            denominator,
        }
    }

    pub fn apply(&self, value: f64) -> f64 {
        value * self.numerator / self.denominator
    }

    pub fn inverse(&self) -> Conversion {
        Conversion {
            numerator: self.denominator,
            denominator: self.numerator
        }
    }
}

impl Unit {
    pub fn new(name: &str, gen: &mut IDGenerator) -> Self {
        Unit {
            name: String::from(name),
            id: gen.next(),
            edges: HashMap::new()
        }
    }

    pub fn push_edge(&mut self, other_id: usize, conversion: Conversion) {
        self.edges.insert(other_id, conversion);
    }

    pub fn contains_edge_to(&self, goal_id: usize) -> bool {
        self.edges.contains_key(&goal_id)
    }

    pub fn iter_connected_ids(&self) -> Keys<usize, Conversion> {
        self.edges.keys()
    }

    pub fn insert_into(self, graph: &mut HashMap<usize, Unit>) {
        graph.insert(self.id, self);
    }

    pub fn get_id(&self) -> usize {
        self.id
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

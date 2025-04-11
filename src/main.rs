use std::{
    collections::{HashMap, HashSet},
    error::Error,
    io::{self, Read},
};

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug, Default)]
struct ColourKey {
    idx: usize,
}

#[derive(Copy, Clone)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Colour {
    pub const RESET: &'static str = "\x1B[0m";

    pub fn ansify(&self) -> String {
        "\x1B[38;2;".to_string()
            + &self.r.to_string()
            + ";"
            + &self.g.to_string()
            + ";"
            + &self.b.to_string()
            + "m"
    }

    pub fn from_hsl(h: f64, s: f64, l: f64) -> Self {
        let rgb = hsl::HSL { h, s, l }.to_rgb();
        Self {
            r: rgb.0,
            g: rgb.1,
            b: rgb.2,
        }
    }
}

pub struct ColorIterator {
    index: usize,
    total: usize,
}

impl ColorIterator {
    pub fn new(total: usize) -> Self {
        Self { index: 0, total }
    }
}

impl Iterator for ColorIterator {
    type Item = Colour;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.total {
            return None;
        }

        let golden_angle = (1.0 + f64::sqrt(5.0)) * 60.0;
        let hue = (self.index as f64 * golden_angle) % 360.0;

        self.index += 1;

        Some(Colour::from_hsl(hue, 0.9, 0.6))
    }
}

impl ColourKey {
    pub fn next(&self) -> Self {
        Self { idx: self.idx + 1 }
    }

    pub fn reify(&self, table: &[Colour]) -> Colour {
        table[self.idx % table.len()]
    }
}

#[derive(Clone)]
struct ColouredString {
    word: String,
    colour: Option<ColourKey>,
}

impl ColouredString {
    pub fn ansify(&self, table: &[Colour]) -> String {
        match self.colour {
            Some(colour) => colour.reify(table).ansify() + &self.word,
            None => self.word.clone(),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct Isomorph {
    good: bool,
    signature: Vec<u8>,
}

impl Isomorph {
    pub fn from_str(word: &str) -> Self {
        let mut signature = Vec::with_capacity(word.len());
        let mut char_map = HashMap::new();
        let mut idx = 0;
        for char in word.chars() {
            if let Some(&idx) = char_map.get(&char) {
                signature.push(idx);
            } else {
                char_map.insert(char, idx);
                signature.push(idx);
                idx += 1;
            }
        }

        let good = signature.iter().collect::<HashSet<_>>().len() != signature.len();

        Isomorph { signature, good }
    }
}

struct IsomorphManager {
    words: Vec<ColouredString>,
}

impl IsomorphManager {
    pub fn colour<'a, I: Iterator<Item = &'a str>>(words: I) -> Self {
        let isomorphic_words = words
            .map(|word| (word, Isomorph::from_str(word)))
            .collect::<Vec<_>>();

        let mut counters = HashMap::new();

        isomorphic_words
            .iter()
            .for_each(|(_, e)| match counters.get_mut(e) {
                Some(x) => *x = e.good,
                None => {
                    counters.insert(e, false);
                }
            });

        let mut colour = ColourKey::default();
        let words = isomorphic_words
            .iter()
            .map(|(s, e)| ColouredString {
                word: s.to_string(),
                colour: match counters.get(e) {
                    None => unreachable!(),
                    Some(false) => None,
                    Some(true) => Some({
                        let c = colour;
                        colour = colour.next();
                        c
                    }),
                },
            })
            .collect();

        Self { words }
    }

    pub fn iter<'a>(&'a mut self) -> impl Iterator<Item = &'a ColouredString> {
        self.words.iter()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut stdin = io::stdin();
    let mut buf = String::new();
    stdin.read_to_string(&mut buf)?;

    let table = ColorIterator::new(1000).collect::<Vec<_>>();
    let coloured = IsomorphManager::colour(buf.split(' '))
        .iter()
        .map(|e| e.ansify(&table))
        .map(|e| e + Colour::RESET)
        .fold("".to_string(), |acc, e| acc + &e + " ");

    println!("{coloured}");

    Ok(())
}

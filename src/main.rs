use std::{
    collections::{HashMap, HashSet},
    error::Error,
    io::{self, Read},
};

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug, Default)]
struct ColourKey {
    idx: usize,
}

const RESET: &'static str = "\x1B[0m";
trait Colour: Clone {
    fn ansify(&self) -> String;
}

#[derive(Copy, Clone)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Colour for RGB {
    fn ansify(&self) -> String {
        "\x1B[38;2;".to_string()
            + &self.r.to_string()
            + ";"
            + &self.g.to_string()
            + ";"
            + &self.b.to_string()
            + "m"
    }
}

impl RGB {
    pub fn from_hsl(h: f64, s: f64, l: f64) -> Self {
        let rgb = hsl::HSL { h, s, l }.to_rgb();
        Self {
            r: rgb.0,
            g: rgb.1,
            b: rgb.2,
        }
    }
}

#[derive(Copy, Clone)]
pub struct DiscordColour {
    value: u8,
}

impl Colour for DiscordColour {
    fn ansify(&self) -> String {
        "\x1B[".to_string() + &self.value.to_string() + "m"
    }
}

pub struct DiscordColourIterator {
    index: usize,
    total: usize,
}

impl DiscordColourIterator {
    pub fn new(total: usize) -> Self {
        Self { index: 0, total }
    }
}

impl Iterator for DiscordColourIterator {
    type Item = DiscordColour;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.total {
            return None;
        }
        self.index += 1;

        let remainder = self.index % 8;
        // TODO: consider bg combos

        Some(DiscordColour {
            value: remainder as u8 + 30,
        })
    }
}

pub struct RGBIterator {
    index: usize,
    total: usize,
}

impl RGBIterator {
    pub fn new(total: usize) -> Self {
        Self { index: 0, total }
    }
}

impl Iterator for RGBIterator {
    type Item = RGB;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.total {
            return None;
        }

        let golden_angle = (1.0 + f64::sqrt(5.0)) * 60.0;
        let hue = (self.index as f64 * golden_angle) % 360.0;

        self.index += 1;

        Some(RGB::from_hsl(hue, 0.9, 0.6))
    }
}

impl ColourKey {
    pub fn next(&self) -> Self {
        Self { idx: self.idx + 1 }
    }

    pub fn reify<T: Colour>(&self, table: &[T]) -> T {
        table[self.idx % table.len()].clone()
    }
}

#[derive(Clone, Debug)]
struct ColouredString {
    word: String,
    colour: Option<ColourKey>,
}

impl ColouredString {
    pub fn ansify<T: Colour>(&self, table: &[T]) -> String {
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

        let mut colour_map: HashMap<Isomorph, ColourKey> = HashMap::new();

        let mut colour = ColourKey::default();
        let words = isomorphic_words
            .iter()
            .map(|(s, e)| ColouredString {
                word: s.to_string(),
                colour: match counters.get(e) {
                    None => unreachable!(),
                    Some(false) => None,
                    Some(true) => Some({
                        let col = match colour_map.get(e) {
                            Some(c) => c.clone(),
                            None => {
                                let c = colour;
                                colour = colour.next();
                                colour_map.insert(e.clone(), c);
                                c
                            }
                        };
                        col
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

    // TODO: cli arg to swap
    let table = DiscordColourIterator::new(1000).collect::<Vec<_>>(); //RGBIterator::new(1000).collect::<Vec<_>>();
    let coloured = IsomorphManager::colour(buf.split(' '))
        .iter()
        .map(|e| e.ansify(&table))
        .map(|e| e + RESET)
        .fold("".to_string(), |acc, e| acc + &e + " ");

    println!("{coloured}");

    Ok(())
}

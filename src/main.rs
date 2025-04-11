use std::{
    collections::{HashMap, HashSet},
    error::Error,
    io::{self, Read},
};

use rand::rng;

#[derive(PartialEq, Eq, Copy, Clone)]
struct ColourKey {
    idx: usize,
}

#[derive(Copy, Clone)]
struct Colour {
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

    pub fn random<T: rand::Rng>(mut rng: T) -> Self {
        Self {
            r: rng.random(),
            g: rng.random(),
            b: rng.random(),
        }
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
    colour: ColourKey,
}

impl ColouredString {
    pub fn ansify(&self, table: &[Colour]) -> String {
        self.colour.reify(table).ansify() + &self.word
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct Isomorph {
    signature: Vec<u8>,
}

struct IsomorphManager {
    isomorphs: HashMap<Isomorph, ColourKey>,
    last_colour: ColourKey,
    words: Vec<ColouredString>,
}

impl IsomorphManager {
    fn add_isomorph(&mut self, isomorph: Isomorph) -> ColourKey {
        if let Some(colour) = self.isomorphs.get(&isomorph) {
            *colour
        } else {
            let colour = self.last_colour;
            self.isomorphs.insert(isomorph, colour);
            self.last_colour = colour.next();
            colour
        }
    }

    fn colour_word(&mut self, word: &str) {
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
        let isomorph = Isomorph { signature };
        let colour = self.add_isomorph(isomorph);
        self.words.push(ColouredString {
            word: word.to_string(),
            colour,
        });
    }

    pub fn colour<'a, I: Iterator<Item = &'a str>>(words: I) -> Self {
        let mut this = Self {
            isomorphs: HashMap::new(),
            last_colour: ColourKey { idx: 0 },
            words: Vec::new(),
        };

        words.for_each(|word| this.colour_word(word));

        this
    }

    pub fn iter(&self) -> std::slice::Iter<'_, ColouredString> {
        self.words.iter()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut stdin = io::stdin();
    let mut buf = String::new();
    stdin.read_to_string(&mut buf)?;

    let mut rng = rng();

    let table = (0..1000)
        .map(|_| Colour::random(&mut rng))
        .collect::<Vec<_>>();

    let coloured = IsomorphManager::colour(buf.split(' '))
        .iter()
        .map(|e| e.ansify(&table))
        .map(|e| e + Colour::RESET)
        .fold("".to_string(), |acc, e| acc + &e + " ");

    println!("{coloured}");

    Ok(())
}

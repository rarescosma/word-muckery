use hashbrown::HashMap;
use itertools::{iproduct, Itertools};
use lazy_static::lazy_static;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

type WordIndex = usize;
type WordSet = Vec<WordIndex>;
type WordSlice<'a> = &'a [WordIndex];
type Letter = u8;
type Word = [Letter; 5];
type Pos = u8;
type Pattern = [Predicate; 5];

const PATTERN_NUM: usize = 3usize.pow(5_u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum LetterState {
    Absent,
    Present,
    AtPos,
}

#[derive(Debug, Copy, Clone, Eq)]
struct Predicate {
    letter: Letter,
    state: LetterState,
    pos: Pos,
}

impl Hash for Predicate {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u8(self.state as u8 & 0b11);
        state.write_u8(self.pos << 2 & 0b11100);
        state.write_u16((self.letter as u16) << 5);
    }
}

impl PartialEq<Self> for Predicate {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos && self.letter == other.letter && self.state == other.state
    }
}

impl Predicate {
    fn apply(&self, ws: impl Iterator<Item = WordIndex>) -> WordSet {
        ws.filter(|&idx| {
            let word = WORDS[idx];

            match self.state {
                LetterState::Absent => ALL_POSITIONS
                    .iter()
                    .all(|&p| word[p as usize] != self.letter),
                LetterState::Present => {
                    ALL_POSITIONS
                        .iter()
                        .any(|&p| word[p as usize] == self.letter)
                        && word[self.pos as usize] != self.letter
                }
                LetterState::AtPos => word[self.pos as usize] == self.letter,
            }
        })
        .collect()
    }
}

const ALL_STATES: [LetterState; 3] = [
    LetterState::Absent,
    LetterState::Present,
    LetterState::AtPos,
];
const ALL_POSITIONS: [Pos; 5] = [0, 1, 2, 3, 4];

lazy_static! {
    static ref ALL_LETTERS: [Letter; 26] = (0..26u8).collect::<Vec<_>>().try_into().unwrap();
    static ref ALL_PREDICATES: [Predicate; 390] = {
        let v: Vec<Predicate> =
            iproduct!(ALL_LETTERS.iter(), ALL_STATES.iter(), ALL_POSITIONS.iter())
                .map(|(&letter, &state, &pos)| Predicate { letter, state, pos })
                .collect();
        v.try_into().unwrap()
    };
    static ref WORDS: Vec<Word> = include_str!("dict.txt").lines().map(into_word).collect();
    static ref WORD_NUM: f32 = WORDS.len() as f32;
}

lazy_static! {
    static ref PREDICATE_BINS: HashMap<Predicate, WordSet> = {
        ALL_PREDICATES
            .iter()
            .map(|p| (*p, p.apply(0..WORDS.len())))
            .collect()
    };
    static ref LAYER_ONE: HashMap<(Predicate, Predicate), WordSet> = {
        let v = ALL_PREDICATES
            .iter()
            .tuple_combinations()
            .collect::<Vec<_>>();

        v.into_par_iter()
            .flat_map(|(p0, p1)| {
                let res = intersect(&PREDICATE_BINS[p0], &PREDICATE_BINS[p1]);
                [((*p0, *p1), res.clone()), ((*p1, *p0), res)]
            })
            .collect::<Vec<_>>()
            .into_iter()
            .collect()
    };
    static ref ALL_PATTERNS: [[LetterState; 5]; PATTERN_NUM] = {
        let mut res = [[LetterState::Absent; 5]; PATTERN_NUM];

        for i in 0..5 {
            for (j, item) in res.iter_mut().enumerate().take(PATTERN_NUM) {
                item[i] = match (j / (3usize.pow(i as u32))) % 3 {
                    0 => LetterState::Absent,
                    1 => LetterState::Present,
                    2 => LetterState::AtPos,
                    _ => unreachable!(),
                }
            }
        }
        res
    };
}

fn into_word<T: AsRef<str>>(t: T) -> Word {
    let v: Vec<Letter> = t.as_ref().chars().map(|c| (c as u8 - b'a')).collect();
    v.try_into().unwrap()
}

fn to_str(w: Word) -> String {
    let mut res = String::default();
    w.iter().for_each(|x| res.push((*x + b'a') as char));
    res
}

fn word_patterns(w: &Word) -> Vec<Pattern> {
    ALL_PATTERNS
        .iter()
        .map(|x| {
            let v: Vec<Predicate> = x
                .iter()
                .enumerate()
                .map(|(pos, &state)| Predicate {
                    letter: w[pos],
                    state,
                    pos: pos as u8,
                })
                .collect();
            v.try_into().unwrap()
        })
        .collect()
}

fn intersect(w0: WordSlice, w1: WordSlice) -> WordSet {
    let (i_max, j_max) = (w0.len(), w1.len());
    let (mut i, mut j) = (0, 0);

    let mut res = Vec::with_capacity(1024);
    while i < i_max && j < j_max {
        match w0[i].cmp(&w1[j]) {
            Ordering::Equal => {
                res.push(w0[i]);
                i += 1;
                j += 1;
            }
            Ordering::Less => {
                i += 1;
            }
            Ordering::Greater => {
                j += 1;
            }
        }
    }
    res
}

fn entropy(bins: Vec<usize>) -> f32 {
    bins.into_iter()
        .map(|b| b as f32)
        .map(|b| -b * (b / *WORD_NUM).log2())
        .sum::<f32>()
        / *WORD_NUM
}

fn main() {
    let now = std::time::Instant::now();

    let mut res = Vec::<(String, f32)>::default();

    for word in WORDS.iter() {
        let patterns = word_patterns(word);
        let _entropy = entropy(
            patterns
                .into_par_iter()
                .flat_map(|pat| {
                    let partial_0 = &LAYER_ONE[&(pat[0], pat[1])];
                    let partial_1 = &LAYER_ONE[&(pat[3], pat[4])];
                    let partial_2 = &LAYER_ONE[&(pat[1], pat[2])];
                    let bin_len = intersect(&intersect(partial_0, partial_1), partial_2).len();
                    if bin_len > 0 {
                        Some(bin_len)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
        );
        res.push((to_str(*word), _entropy));
    }
    res.sort_unstable_by(|w0, w1| w1.1.partial_cmp(&w0.1).unwrap());
    dbg!(&res[..15]);

    let time = now.elapsed().as_millis();
    println!("Time: {}ms", time);
}

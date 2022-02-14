mod ascii_bit_set;
mod fivegram;

use ascii_bit_set::AsciiBitSet;
use bitvec_simd::BitVec;
use fivegram::Fivegram;
use itertools::iproduct;
use lazy_static::lazy_static;
use rayon::prelude::*;

type WordBitMap = BitVec;
type Letter = u8;
type Word = [Letter; 5];
type Pos = u8;
type Pattern = [Predicate; 5];

const PATTERN_NUM: usize = 3usize.pow(5_u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum LetterState {
    Absent,
    Present,
    AtPos,
}

#[derive(Debug, Copy, Clone)]
pub struct Predicate {
    letter: Letter,
    state: LetterState,
    pos: Pos,
}

impl Predicate {
    fn cached_bitmap(&self) -> &WordBitMap {
        PRED_BIN[self.index()].as_ref().unwrap()
    }

    fn index(&self) -> usize {
        let mut res = (self.state as u8 & 0b11 | (self.pos & 0b111) << 2) as u16;
        res |= (((self.letter - b'a') & 0b11111) as u16) << 5;
        res as usize
    }

    fn matches_index(&self, idx: usize) -> bool {
        let ascii_bit_set = ASCII_BIT_SETS[idx];
        let fivegram = FIVEGRAMS[idx];

        match self.state {
            LetterState::Absent => !ascii_bit_set.has_letter(self.letter),
            LetterState::Present => {
                ascii_bit_set.has_letter(self.letter)
                    && !fivegram.has_letter(self.letter, self.pos as usize)
            }
            LetterState::AtPos => fivegram.has_letter(self.letter, self.pos as usize),
        }
    }

    fn as_bitmap(&self) -> WordBitMap {
        let mut ret = BitVec::zeros(*WORD_NUM);
        for idx in 0..*WORD_NUM {
            if self.matches_index(idx) {
                ret.set(idx, true);
            }
        }
        ret
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
    static ref WORDS: Vec<Word> = include_str!("dict.txt")
        .lines()
        .flat_map(|l| l.as_bytes().try_into())
        .collect();
    static ref FIVEGRAMS: Vec<Fivegram> = WORDS.iter().map(|w| Fivegram::from_letters(w)).collect();
    static ref ASCII_BIT_SETS: Vec<AsciiBitSet> =
        WORDS.iter().map(|w| AsciiBitSet::from_letters(w)).collect();
    static ref WORD_NUM: usize = WORDS.len();
    static ref WORD_NUM_F: f32 = WORDS.len() as f32;
}

lazy_static! {
    static ref PRED_BIN: [Option<WordBitMap>; 1024] = {
        let mut res = vec![None; 1024];

        for p in iproduct!(ALL_LETTERS.iter(), ALL_STATES.iter(), ALL_POSITIONS.iter())
            .map(|(&letter, &state, &pos)| Predicate { letter, state, pos })
        {
            res[p.index()] = Some(p.as_bitmap());
        }
        res.try_into().unwrap()
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

fn to_str(w: &Word) -> String {
    String::from_utf8_lossy(w).to_string()
}

fn word_patterns(w: &Word) -> Vec<Pattern> {
    ALL_PATTERNS
        .iter()
        .map(|states| {
            let mut pos = 0;
            states.map(|state| {
                let p = Predicate {
                    letter: w[pos as usize] - b'a',
                    state,
                    pos,
                };
                pos += 1;
                p
            })
        })
        .collect()
}

#[inline]
fn intersect(pattern: &Pattern) -> WordBitMap {
    let mut res = pattern[0].cached_bitmap().clone();
    for pred in &pattern[1..] {
        res.and_inplace(pred.cached_bitmap())
    }
    res
}

fn entropy(word: &Word) -> f32 {
    let patterns = word_patterns(word);
    patterns
        .into_par_iter()
        .filter_map(|ref pat| match intersect(pat).count_ones() {
            x if x > 0 => Some(x as f32),
            _ => None,
        })
        .map(|b| -b * (b / *WORD_NUM_F).log2())
        .sum::<f32>()
        / *WORD_NUM_F
}

fn main() {
    let now = std::time::Instant::now();

    let mut res: Vec<(Word, f32)> = WORDS
        .clone()
        .into_par_iter()
        .map(|w| (w, entropy(&w)))
        .collect();

    res.sort_unstable_by(|w0, w1| w1.1.partial_cmp(&w0.1).unwrap());
    for (word, i_e) in &res[..10] {
        println!("E['{}'] = {}", to_str(word), i_e)
    }

    let time = now.elapsed().as_millis();
    println!("Time: {}ms", time);
}

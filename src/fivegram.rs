/**
    Bit-packed 5-letter a-z ASCII word:

    empty = 0b00000
    a     = 0b00001
    ...
    z     = 0b11010
**/
#[derive(Default, Copy, Clone)]
pub struct Fivegram {
    word: u32,
}

impl Fivegram {
    pub fn set_letter(&mut self, l: u8, pos: usize) {
        self.word |= ((l + 1) as u32) << (pos * 5)
    }

    #[inline]
    pub fn has_letter(&self, l: u8, pos: usize) -> bool {
        (self.word >> (pos * 5)) & 0b11111 == (l + 1) as u32
    }

    pub fn from_letters(bytes: &[u8]) -> Self {
        let mut res = Self::default();
        for (i, b) in bytes.iter().enumerate() {
            res.set_letter(b - b'a', i);
        }

        res
    }

    #[cfg(test)]
    #[inline]
    pub fn partial_match(&self, pattern: &Self) -> bool {
        self.word & pattern.word == pattern.word
    }
}

#[cfg(test)]
mod tests {
    use super::Fivegram;

    #[test]
    fn test_from_bytes() {
        let fg = Fivegram::from_letters("abcde".as_bytes());

        assert_eq!(fg.word, 0b00_00000_00101_00100_00011_00010_00001);
    }

    #[test]
    fn test_has_letter() {
        let fg = Fivegram::from_letters("abcde".as_bytes());

        assert!(fg.has_letter(b'a', 0));
        assert!(fg.has_letter(b'b', 1));
        assert!(fg.has_letter(b'c', 2));
        assert!(fg.has_letter(b'd', 3));
        assert!(fg.has_letter(b'e', 4));
    }

    #[test]
    fn test_matches_full() {
        let l = Fivegram::from_letters("abcde".as_bytes());
        let r = Fivegram::from_letters("abcde".as_bytes());

        assert!(l.partial_match(&r));
    }

    #[test]
    fn test_matches_prefix() {
        let l = Fivegram::from_letters("abcde".as_bytes());
        let r = Fivegram::from_letters("abc".as_bytes());

        assert!(l.partial_match(&r));
    }

    #[test]
    fn test_matches_with_holes() {
        let l = Fivegram::from_letters("abcde".as_bytes());
        let mut r = Fivegram::default();
        r.set_letter(b'b', 1);
        r.set_letter(b'd', 3);

        assert!(l.partial_match(&r));
    }
}

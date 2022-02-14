/**
    Lower-case ASCII bit-set, to quickly check if letter i
**/
#[derive(Default, Copy, Clone)]
pub struct AsciiBitSet {
    set: u32,
}

impl AsciiBitSet {
    #[cfg(test)]
    pub fn set_letter(&mut self, l: u8) {
        self.set |= 1 << l
    }

    pub fn from_letters(bytes: &[u8]) -> Self {
        let mut set = Self::default();
        for b in bytes {
            set.set |= 1 << (b - b'a');
        }

        set
    }

    #[inline]
    pub fn has_letter(&self, letter: u8) -> bool {
        (self.set >> letter) & 1 == 1
    }

    #[cfg(test)]
    #[inline]
    pub fn intersect(&self, set: &Self) -> bool {
        self.set & set.set != 0
    }
}

#[cfg(test)]
mod tests {
    use super::AsciiBitSet;

    #[test]
    fn test_from_bytes() {
        let s = AsciiBitSet::from_letters("abcde".as_bytes());
        assert_eq!(s.set, 0b11111);

        let s = AsciiBitSet::from_letters("zyxwv".as_bytes());
        assert_eq!(s.set, 0b0011_1110_0000_0000_0000_0000_0000);

        let s = AsciiBitSet::from_letters("abcdefghijklmnopqrstuvwxyz".as_bytes());
        assert_eq!(s.set, 0b0011_1111_1111_1111_1111_1111_1111);
    }

    #[test]
    fn test_has_letter() {
        let mut s = AsciiBitSet::default();
        s.set_letter(b'a');
        s.set_letter(b'z');
        s.set_letter(b'k');

        assert!(s.has_letter(b'a'));
        assert!(s.has_letter(b'z'));
        assert!(s.has_letter(b'k'));
    }

    #[test]
    fn test_intersect() {
        let l = AsciiBitSet::from_letters("abcde".as_bytes());
        let r = AsciiBitSet::from_letters("bdz".as_bytes());

        assert!(l.intersect(&r));
    }
}

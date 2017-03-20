use std::fmt;
use std::marker;

/// A trait that all HTTP/2 frame header flags need to implement.
pub trait Flag : fmt::Debug + Copy + Clone + Sized {
    /// Returns a bit mask that represents the flag.
    fn bitmask(&self) -> u8;

    fn flags() -> &'static [Self];

    fn to_flags(&self) -> Flags<Self> {
        Flags::new(self.bitmask())
    }
}

/// A helper struct that can be used by all frames that do not define any flags.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NoFlag;
impl Flag for NoFlag {
    fn bitmask(&self) -> u8 {
        0
    }

    fn flags() -> &'static [Self] {
        static FLAGS: &'static [NoFlag] = &[NoFlag];
        FLAGS
    }
}


#[derive(PartialEq, Eq, Copy, Clone)]
pub struct Flags<F : Flag + 'static>(pub u8, marker::PhantomData<F>);

impl<F : Flag> Flags<F> {
    pub fn new(value: u8) -> Flags<F> {
        Flags(value, marker::PhantomData)
    }

    pub fn is_set(&self, flag: &F) -> bool {
        (self.0 & flag.bitmask()) != 0
    }

    pub fn set(&mut self, flag: &F) {
        self.0 |= flag.bitmask()
    }

    pub fn clear(&mut self, flag: &F) {
        self.0 &= !flag.bitmask();
    }
}

impl<F : Flag> Default for Flags<F> {
    fn default() -> Self {
        Flags::new(0)
    }
}

impl<F : Flag> fmt::Debug for Flags<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 == 0 {
            return write!(f, "0");
        }

        let mut copy: Flags<F> = (*self).clone();

        let mut first = true;
        for flag in F::flags() {
            if copy.is_set(flag) {
                if !first {
                    write!(f, "|")?;
                }
                first = false;

                write!(f, "{:?}", flag)?;
                copy.clear(flag);
            }
        }

        // unknown flags
        if copy.0 != 0 {
            if !first {
                write!(f, "|")?;
            }
            write!(f, "0x{:x}", copy.0)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    enum FakeFlag {
        Foo = 0x1,
        Bar = 0x80,
    }

    impl Flag for FakeFlag {
        fn bitmask(&self) -> u8 {
            *self as u8
        }

        fn flags() -> &'static [Self] {
            static FLAGS: &'static [FakeFlag] = &[FakeFlag::Foo, FakeFlag::Bar];
            FLAGS
        }
    }

    #[test]
    fn fmt_flags() {
        assert_eq!("0", format!("{:?}", Flags::<FakeFlag>::new(0)));
        assert_eq!("Foo", format!("{:?}", Flags::<FakeFlag>::new(0x1)));
        assert_eq!("0x62", format!("{:?}", Flags::<FakeFlag>::new(0x62)));
        assert_eq!("Foo|Bar", format!("{:?}", Flags::<FakeFlag>::new(0x81)));
        assert_eq!("Foo|Bar|0x62", format!("{:?}", Flags::<FakeFlag>::new(0x81 | 0x62)));
    }
}

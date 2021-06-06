use bytes::Buf;
use std::cmp;

pub(crate) struct IterBuf<B: Buf, I: Iterator<Item = B>> {
    iter: I,
    next: Option<B>,
    rem: usize,
}

impl<B: Buf, I: Iterator<Item = B>> IterBuf<B, I> {
    pub fn new(iter: I, rem: usize) -> IterBuf<B, I> {
        let mut b = IterBuf {
            next: None,
            iter,
            rem,
        };
        b.fill_next();
        b
    }

    fn fill_next(&mut self) {
        loop {
            if let None = self.next {
                self.next = self.iter.next();
            }
            match &mut self.next {
                None => return,
                Some(b) => {
                    if b.has_remaining() {
                        return;
                    }
                }
            }
        }
    }
}

impl<B: Buf, I: Iterator<Item = B>> Buf for IterBuf<B, I> {
    fn remaining(&self) -> usize {
        self.rem
    }

    fn chunk(&self) -> &[u8] {
        match &self.next {
            Some(buf) => buf.chunk(),
            None => &[],
        }
    }

    fn advance(&mut self, cnt: usize) {
        while cnt != 0 {
            if let Some(buf) = &mut self.next {
                let min = cmp::min(cnt, buf.remaining());
                buf.advance(min);
                self.rem -= min;
                if !buf.has_remaining() {
                    self.next = None;
                } else {
                    return;
                }
            } else {
                panic!("overflow");
            }
            debug_assert!(self.next.is_none());
            self.fill_next();
        }
    }
}

#[cfg(test)]
mod test {}

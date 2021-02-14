use std::mem::{replace, MaybeUninit};

pub struct RingBufferU<T> {
    start: usize,
    size: usize,
    buffer: Vec<MaybeUninit<T>>,
}

impl<T> RingBufferU<T> {
    pub fn with_capacity(cap: usize) -> Self {
        let buffer = {
            let mut buffer = Vec::with_capacity(cap);
            for _ in 0..cap {
                buffer.push(MaybeUninit::uninit());
            }
            buffer
        };
        RingBufferU {
            start: 0,
            size: 0,
            buffer,
        }
    }

    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }

    pub fn push(&mut self, element: T) {
        let idx = (self.start + self.size) % self.capacity();
        if self.size == self.capacity() {
            unsafe {
                let _ =
                    replace(self.buffer.get_mut(idx).unwrap(), MaybeUninit::uninit()).assume_init();
            } // Drop element that will be overwritten
            self.start += 1
        } else {
            self.size += 1
        }
        self.buffer[idx] = MaybeUninit::new(element);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.size == 0 {
            return None;
        }

        let idx = self.start;
        self.start = (self.start + 1) % self.capacity();
        self.size -= 1;
        Some(unsafe {
            replace(self.buffer.get_mut(idx).unwrap(), MaybeUninit::uninit()).assume_init()
        })
    }
}

impl<T> IntoIterator for RingBufferU<T> {
    type Item = T;
    type IntoIter = RBUIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        RBUIter(self)
    }
}

impl<T> Drop for RingBufferU<T> {
    fn drop(&mut self) {
        while self.size > 0 {
            self.pop();
        }
    }
}

pub struct RBUIter<T>(RingBufferU<T>);

impl<T> Iterator for RBUIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.size == 0 {
            return None;
        }
        let idx = self.0.start;
        self.0.start = (self.0.start + 1) % self.0.capacity();
        self.0.size -= 1;
        Some(unsafe {
            replace(self.0.buffer.get_mut(idx).unwrap(), MaybeUninit::uninit()).assume_init()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop_test() {
        let mut rb = RingBufferU::with_capacity(2);
        assert_eq!(rb.pop(), None);
        rb.push(3);
        assert_eq!(rb.pop(), Some(3));
        assert_eq!(rb.pop(), None);
    }

    #[test]
    fn overwrite_test() {
        let mut rb = RingBufferU::with_capacity(3);
        rb.push(1);
        rb.push(2);
        rb.push(3);
        rb.push(4);
        assert_eq!(rb.pop(), Some(2));
        assert_eq!(rb.pop(), Some(3));
        assert_eq!(rb.pop(), Some(4));
    }

    #[test]
    fn iter_test() {
        use std::iter::FromIterator;
        let mut rb = RingBufferU::with_capacity(7);
        for i in 0..7 {
            rb.push(i)
        }
        assert_eq!(rb.pop(), Some(0));
        assert_eq!(rb.pop(), Some(1));
        rb.push(7);
        assert_eq!(Vec::from_iter(rb.into_iter()), vec![2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn zero_sized_types_test() {
        use std::iter::FromIterator;
        struct ZST;
        let mut rb = RingBufferU::with_capacity(3);
        rb.push(ZST {});
        rb.push(ZST {});
        rb.push(ZST {});
        rb.pop();
        assert_eq!(rb.capacity(), usize::MAX);
        assert_eq!(Vec::from_iter(rb.into_iter()).len(), 2);
    }
}

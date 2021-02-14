#[derive(Clone, Debug)]
enum RawRingBuffer<T> {
    Sized(Vec<Option<T>>),
    Zerosized(Vec<T>),
}

impl<T> RawRingBuffer<T> {
    fn capacity(&self) -> usize {
        match &self {
            RawRingBuffer::Sized(vo) => vo.capacity(),
            RawRingBuffer::Zerosized(v) => v.capacity(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RingBuffer<T> {
    start: usize,
    size: usize,
    buffer: RawRingBuffer<T>,
}

impl<T> RingBuffer<T> {
    pub fn with_capacity(cap: usize) -> Self {
        let buffer = if std::mem::size_of::<T>() > 0 {
            let mut buffer = Vec::with_capacity(cap);
            for _ in 0..cap {
                buffer.push(None);
            }
            RawRingBuffer::Sized(buffer)
        } else {
            RawRingBuffer::Zerosized(Vec::new())
        };
        Self {
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
        match &mut self.buffer {
            RawRingBuffer::Sized(vo) => vo[idx] = Some(element),
            RawRingBuffer::Zerosized(v) => v.push(element),
        }
        if self.size == self.capacity() {
            self.start += 1; // Overwrote first element;
        } else {
            self.size += 1;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.size == 0 {
            return None;
        }
        let idx = self.start;
        self.start = (self.start + 1) % self.capacity();
        self.size -= 1;
        match &mut self.buffer {
            RawRingBuffer::Sized(vo) => vo.get_mut(idx).unwrap().take(),
            RawRingBuffer::Zerosized(v) => v.pop(),
        }
    }
}

pub struct RBIter<T>(RingBuffer<T>);

impl<T> Iterator for RBIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.0.size == 0 {
            return None;
        }
        let idx = self.0.start;
        self.0.start = (self.0.start + 1) % self.0.capacity();
        self.0.size -= 1;
        match &mut self.0.buffer {
            RawRingBuffer::Sized(vo) => vo.get_mut(idx).unwrap().take(),
            RawRingBuffer::Zerosized(v) => v.pop(),
        }
    }
}

impl<T> IntoIterator for RingBuffer<T> {
    type Item = T;
    type IntoIter = RBIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        RBIter(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop_test() {
        let mut rb = RingBuffer::with_capacity(2);
        assert_eq!(rb.pop(), None);
        rb.push(3);
        assert_eq!(rb.pop(), Some(3));
        assert_eq!(rb.pop(), None);
    }

    #[test]
    fn overwrite_test() {
        let mut rb = RingBuffer::with_capacity(3);
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
        let mut rb = RingBuffer::with_capacity(7);
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
        let mut rb = RingBuffer::with_capacity(3);
        rb.push(ZST {});
        rb.push(ZST {});
        rb.push(ZST {});
        rb.pop();
        assert_eq!(rb.capacity(), usize::MAX);
        assert_eq!(Vec::from_iter(rb.into_iter()).len(), 2);
    }
}

pub trait SourceReader<'a> {
    fn is_eof(&self) -> bool;
    fn peek(&self) -> Option<char>;
    fn next(&mut self) -> Option<char>;
    fn read_until(&mut self, cond: fn(char) -> bool) -> Option<&'a str>;
}

#[derive(Debug)]
pub struct StrReader<'a> {
    ptr: usize,
    source: &'a str,
}

impl<'a> StrReader<'a> {
    pub fn new(source: &'a str) -> StrReader {
        StrReader { ptr: 0, source }
    }
}

impl<'a> SourceReader<'a> for StrReader<'a> {
    fn is_eof(&self) -> bool {
        self.ptr >= self.source.len()
    }

    fn peek(&self) -> Option<char> {
        self.source.chars().nth(self.ptr)
    }

    fn next(&mut self) -> Option<char> {
        let out = self.peek();
        self.ptr += 1;
        out
    }

    fn read_until(&mut self, cond: fn(char) -> bool) -> Option<&'a str> {
        let i = self.ptr as i64;

        loop {
            match self.peek() {
                Some(c) => {
                    if cond(c) {
                        self.ptr += 1;
                    } else {
                        break;
                    }
                }
                None => break,
            };
        }

        let j = self.ptr as i64 - 1;
        if j >= i {
            Some(&self.source[(i as usize)..=(j as usize)])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::source_reader::*;

    #[test]
    fn test_is_eof() {
        let mut reader = StrReader::new("12");
        assert!(!reader.is_eof());

        assert_eq!(Some('1'), reader.next());
        assert_eq!(Some('2'), reader.next());

        assert!(reader.is_eof());

        assert_eq!(None, reader.next());
        assert_eq!(None, reader.next());

        assert!(reader.is_eof());
    }

    #[test]
    fn test_peek() {
        let mut reader = StrReader::new("12");

        assert_eq!(Some('1'), reader.peek());
        assert_eq!(Some('1'), reader.next());
        assert_eq!(Some('2'), reader.peek());
    }

    #[test]
    fn test_next() {
        let mut reader = StrReader::new("12");

        assert_eq!(Some('1'), reader.next());
        assert_eq!(Some('2'), reader.next());
        assert_eq!(None, reader.next());
    }

    #[test]
    fn test_read_until() {
        let mut reader = StrReader::new("12345abc");
        assert_eq!(Some("12345"), reader.read_until(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_read_until_at_end() {
        let mut reader = StrReader::new("12345");
        assert_eq!(Some("12345"), reader.read_until(|c| c.is_ascii_digit()));
    }
}

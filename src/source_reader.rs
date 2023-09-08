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

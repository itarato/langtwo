pub trait SourceReader {
    fn is_eof(&self) -> bool;
    fn peek(&self) -> Option<char>;
    fn next(&mut self) -> Option<char>;
}

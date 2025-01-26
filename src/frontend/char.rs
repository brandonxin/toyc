pub trait Decode<R: std::io::Read> {
    fn new(input: R) -> Self;

    fn get_char(&mut self) -> Option<char>;
}

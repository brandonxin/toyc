use std::io::Read;

struct Reader<R: Read> {
    input: R,
}

impl<R: Read> Reader<R> {
    pub fn getchar(&mut self) -> Option<char> {
        let Some(raw) = self.get_raw_char() else {
            return None;
        };

        Some(char::try_from(raw).unwrap())
    }

    fn get_raw_char(&mut self) -> Option<u32> {
        let Some(a) = self.get_byte() else {
            return None;
        };

        if a & 0x80 == 0 {
            return Some(a);
        }

        if a & 0xE0 == 0xC0 {
            let b = self.get_continuation_byte();
            let code_point = ((a & 0x1F) << 6) | b;
            if code_point < 0x80 {
                panic!("invalid code point")
            }
            return Some(code_point);
        }
        if a & 0xF0 == 0xE0 {
            let b = self.get_continuation_byte();
            let c = self.get_continuation_byte();
            let code_point = ((a & 0x0F) << 12) | (b << 6) | c;
            if code_point < 0x0800 {
                panic!("invalid code point");
            }
            if (0xD800 <= code_point) && (code_point <= 0xDFFF) {
                panic!("invalid scalar value for lone surrogate");
            }
            return Some(code_point);
        }
        if a & 0xF8 == 0xF0 {
            let b = self.get_continuation_byte();
            let c = self.get_continuation_byte();
            let d = self.get_continuation_byte();
            let code_point = ((a & 0x07) << 18) | (b << 12) | (c << 6) | d;
            if code_point < 0x010000 || code_point > 0x10FFFF {
                panic!("invalid code point");
            }
            return Some(code_point);
        }

        panic!("invalid byte sequence");
    }

    fn get_continuation_byte(&mut self) -> u32 {
        let Some(byte) = self.get_byte() else {
            panic!("invalid byte sequence");
        };

        if (byte & 0xC0) == 0x80 {
            return byte & 0x3F;
        } else {
            panic!("invalid continuation byte");
        }
    }

    fn get_byte(&mut self) -> Option<u32> {
        let mut buf = [0 as u8; 1];
        match self.input.read(&mut buf).unwrap() {
            0 => None,
            _ => Some(buf[0] as u32),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_0() {
        let input = String::from("");
        let mut reader = Reader {
            input: input.as_bytes(),
        };

        assert_eq!(reader.getchar(), None);
        assert_eq!(reader.getchar(), None);
    }

    #[test]
    fn ascii_1() {
        let input = String::from("a");
        let mut reader = Reader {
            input: input.as_bytes(),
        };

        assert_eq!(reader.getchar(), Some('a'));
        assert_eq!(reader.getchar(), None);
        assert_eq!(reader.getchar(), None);
    }

    #[test]
    fn ascii_n() {
        let input = String::from("abcd");
        let mut reader = Reader {
            input: input.as_bytes(),
        };

        assert_eq!(reader.getchar(), Some('a'));
        assert_eq!(reader.getchar(), Some('b'));
        assert_eq!(reader.getchar(), Some('c'));
        assert_eq!(reader.getchar(), Some('d'));
        assert_eq!(reader.getchar(), None);
        assert_eq!(reader.getchar(), None);
    }

    #[test]
    fn utf8_n() {
        let input = String::from("上山打老虎🐯");
        let mut reader = Reader {
            input: input.as_bytes(),
        };
        assert_eq!(reader.getchar(), Some('上'));
        assert_eq!(reader.getchar(), Some('山'));
        assert_eq!(reader.getchar(), Some('打'));
        assert_eq!(reader.getchar(), Some('老'));
        assert_eq!(reader.getchar(), Some('虎'));
        assert_eq!(reader.getchar(), Some('🐯'));
        assert_eq!(reader.getchar(), None);
        assert_eq!(reader.getchar(), None);
    }

    #[test]
    #[should_panic]
    fn invalid_utf8_1() {
        let input: [u8; 1] = [0x80];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn invalid_utf8_2() {
        let input: [u8; 2] = [0xC0, 0x00];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn invalid_utf8_3() {
        let input: [u8; 3] = [0xC0, 0x80, 0x00];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn invalid_utf8_4() {
        let input: [u8; 4] = [0xC0, 0x80, 0x80, 0x00];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    fn utf8_6() {
        let input: [u8; 6] = [0xF0, 0x9F, 0x90, 0xAF, 0xC0, 0x00];
        let mut reader = Reader { input: &input[..] };
        assert_eq!(reader.getchar(), Some('🐯'));
    }

    #[test]
    #[should_panic]
    fn invalid_utf8_6() {
        let input: [u8; 6] = [0xF0, 0x9F, 0x90, 0xAF, 0xC0, 0x00];
        let mut reader = Reader { input: &input[..] };
        assert_eq!(reader.getchar(), Some('🐯'));
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn byte_never_appear_case_11() {
        let input: [u8; 1] = [0xC0];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn byte_never_appear_case_12() {
        let input: [u8; 1] = [0xC1];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn byte_never_appear_case_13() {
        let input: [u8; 1] = [0xF5];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn byte_never_appear_case_14() {
        let input: [u8; 1] = [0xFF];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn byte_never_appear_case_2() {
        let input: [u8; 1] = [0xBF];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn byte_never_appear_case_41() {
        // An overlong encoding - 0xE0 followed by less than 0xA0
        // In that case, for a 3-byte encoding 1110wwww 10xxxxyy 10yyzzz,
        // there is wwww = 0, xxxx <= 0111, which can be encoded by
        // 2-byte sequence 110xxxyy 10yyzzz
        let input: [u8; 3] = [0xE0, 0x9F, 0x00];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn byte_never_appear_case_42() {
        // An overlong encoding - 0xF0 followed by less than 0x90
        // In that case, for a 4-byte encoding:
        // 11110uvv 10vvwwww 10xxxxyy 10yyzzz,
        // there is u = 0,vvvv = 0, which can be encoded by 3-byte sequence
        //
        let input: [u8; 4] = [0xF0, 0x8F, 0x00, 0x00];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }

    #[test]
    #[should_panic]
    fn byte_never_appear_case_5() {
        // A 4-byte sequence that decodes to a value greater that U+10FFFF (0xF4
        // followed by 0x90 or greater)
        let input: [u8; 4] = [0xF4, 0x90, 0x00, 0x00];
        let mut reader = Reader { input: &input[..] };
        reader.getchar();
    }
}

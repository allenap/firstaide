use std::borrow::Borrow;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;

pub fn escape<T: Into<OsString>>(s: T) -> Vec<u8> {
    let sin = s.into().into_vec();
    if let Some(esc) = _prepare(&sin) {
        // Maybe pointless optimisation, but here we calculate the memory we need to
        // avoid reallocations as we construct the output string. Since we now know
        // we're going to use Bash's $'...' string notation, we also add 3 bytes.
        let size: usize = esc.iter().map(EscapeAs::size).sum();
        let mut sout = Vec::with_capacity(size + 3);
        _escape_into(esc, &mut sout); // Do the work.
        sout
    } else {
        sin
    }
}

pub fn escape_into<T: Into<OsString>>(s: T, sout: &mut Vec<u8>) {
    let sin = s.into().into_vec();
    if let Some(esc) = _prepare(&sin) {
        // Maybe pointless optimisation, but here we calculate the memory we need to
        // avoid reallocations as we construct the output string. Since we now know
        // we're going to use Bash's $'...' string notation, we also add 3 bytes.
        let size: usize = esc.iter().map(EscapeAs::size).sum();
        sout.reserve(size + 3);
        _escape_into(esc, sout); // Do the work.
    } else {
        sout.extend(sin);
    }
}

fn _prepare(sin: &Vec<u8>) -> Option<Vec<EscapeAs>> {
    let esc: Vec<_> = sin.iter().map(EscapeAs::from).collect();
    // An optimisation: if the string only contains "safe" characters we can
    // avoid further work.
    if esc.iter().all(EscapeAs::is_literal) {
        None
    } else {
        Some(esc)
    }
}

fn _escape_into(esc: Vec<EscapeAs>, sout: &mut Vec<u8>) {
    // Push a Bash-style $'...' escaped string into `sout`.
    sout.extend(b"$'");
    for mode in esc {
        use EscapeAs::*;
        match mode {
            Bell => sout.extend(b"\\a"),
            Backspace => sout.extend(b"\\b"),
            Escape => sout.extend(b"\\e"),
            FormFeed => sout.extend(b"\\f"),
            NewLine => sout.extend(b"\\n"),
            CarriageReturn => sout.extend(b"\\r"),
            HorizontalTab => sout.extend(b"\\t"),
            VerticalTab => sout.extend(b"\\v"),
            Backslash => sout.extend(b"\\\\"),
            SingleQuote => sout.extend(b"\\'"),
            ByValue(ch) => sout.extend(format!("\\x{:02X}", ch).bytes()),
            Literal(ch) => sout.push(ch),
            Quoted(ch) => sout.push(ch),
        }
    }
    sout.push(b'\'');
}

//
// From bash(1):
//
//   Words of the form $'string' are treated specially. The word expands to
//   string, with backslash- escaped characters replaced as specified by the ANSI
//   C standard. Backslash escape sequences, if present, are decoded as follows:
//
//     \a     alert (bell)
//     \b     backspace
//     \e     an escape character
//     \f     form feed
//     \n     new line
//     \r     carriage return
//     \t     horizontal tab
//     \v     vertical tab
//     \\     backslash
//     \'     single quote
//     \nnn   the eight-bit character whose value is the octal value nnn (one to
//            three digits)
//     \xHH   the eight-bit character whose value is the hexadecimal value HH (one
//            or two hex digits)
//     \cx    a control-x character
//

#[derive(PartialEq)]
enum EscapeAs {
    Bell,
    Backspace,
    Escape,
    FormFeed,
    NewLine,
    CarriageReturn,
    HorizontalTab,
    VerticalTab,
    Backslash,
    SingleQuote,
    ByValue(u8),
    Literal(u8),
    Quoted(u8),
}

impl EscapeAs {
    fn from<T: Borrow<u8>>(ch: T) -> Self {
        let ch = *ch.borrow();
        use EscapeAs::*;
        match ch {
            // Backslash sequences.
            BEL => Bell,
            BS => Backspace,
            ESC => Escape,
            FF => FormFeed,
            LF => NewLine,
            CR => CarriageReturn,
            TAB => HorizontalTab,
            VT => VerticalTab,
            b'\\' => Backslash,
            b'\'' => SingleQuote,

            // ASCII letters, numbers, and "safe" punctuation.
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' => Literal(ch),
            b',' | b'.' | b'/' | b'_' | b'-' => Literal(ch),

            // ASCII punctuation which can be special to Bash.
            b'|' | b'&' | b';' | b'(' | b')' | b'<' | b'>' => Quoted(ch),
            b' ' | b'?' | b'[' | b']' | b'{' | b'}' | b'`' => Quoted(ch),
            b'~' | b'!' | b'$' | b'@' | b'+' | b'=' => Quoted(ch),
            b'*' | b'"' | b'%' | b'#' | b':' | b'^' => Quoted(ch),

            // Other control characers.
            0x00..=0x06 | 0x0E..=0x1A | 0x1C..=0x1F | 0x7f => ByValue(ch),

            // High bytes.
            0x80..=0xff => ByValue(ch),
        }
    }

    fn is_literal(&self) -> bool {
        match self {
            EscapeAs::Literal(_) => true,
            _ => false,
        }
    }

    fn size(&self) -> usize {
        use EscapeAs::*;
        match self {
            Bell => 2,
            Backspace => 2,
            Escape => 2,
            FormFeed => 2,
            NewLine => 2,
            CarriageReturn => 2,
            HorizontalTab => 2,
            VerticalTab => 2,
            Backslash => 2,
            SingleQuote => 2,
            ByValue(_) => 4,
            Literal(_) => 1,
            Quoted(_) => 1,
        }
    }
}

const BEL: u8 = 0x07; // -> \a
const BS: u8 = 0x08; // -> \b
const TAB: u8 = 0x09; // -> \t
const LF: u8 = 0x0A; // -> \n
const VT: u8 = 0x0B; // -> \v
const FF: u8 = 0x0C; // -> \f
const CR: u8 = 0x0D; // -> \r
const ESC: u8 = 0x1B; // -> \e

#[cfg(test)]
mod tests {
    use super::escape;
    use super::escape_into;

    #[test]
    fn test_lowercase_ascii() {
        assert_eq!(
            escape("abcdefghijklmnopqrstuvwxyz"),
            "abcdefghijklmnopqrstuvwxyz".as_bytes()
        );
    }

    #[test]
    fn test_uppercase_ascii() {
        assert_eq!(
            escape("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ".as_bytes()
        );
    }

    #[test]
    fn test_numbers() {
        assert_eq!(escape("0123456789"), "0123456789".as_bytes());
    }

    #[test]
    fn test_punctuation() {
        assert_eq!(escape("-_=/,.+"), "$'-_=/,.+'".as_bytes());
    }

    #[test]
    fn test_basic_escapes() {
        assert_eq!(escape(r#"woo"wah""#), r#"$'woo"wah"'"#.as_bytes());
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_control_characters() {
        assert_eq!(escape(&"\x07"), "$'\\a'".as_bytes());
        assert_eq!(escape(&"\x00"), "$'\\x00'".as_bytes());
        assert_eq!(escape(&"\x06"), "$'\\x06'".as_bytes());
        assert_eq!(escape(&"\x7F"), "$'\\x7F'".as_bytes());
    }

    #[test]
    fn test_escape_into_plain() {
        let mut buffer = Vec::new();
        escape_into("hello", &mut buffer);
        assert_eq!(buffer, b"hello");
    }

    #[test]
    fn test_escape_into_with_escapes() {
        let mut buffer = Vec::new();
        escape_into("-_=/,.+", &mut buffer);
        assert_eq!(buffer, b"$'-_=/,.+'");
    }
}

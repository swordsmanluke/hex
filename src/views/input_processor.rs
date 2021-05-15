use std::io;

use regex::Regex;
use termion::cursor::Goto;
use crate::views::{DimConstraint, TermLocation, CharDims};
use regex::internal::Char;
use std::borrow::Borrow;

const ESC: char = '\u{1B}';
const CSI: &str = r"\u{1B}\[";
const VT100_PATTERN: &'static str = r"((\u001b\[|\u009b)[\u0030-\u003f]*[\u0020-\u002f]*[\u0040-\u007e])+";

pub struct InputProcessor {
    esc_seq: String,
    buffer: String,
}

impl InputProcessor {
    pub fn new() -> InputProcessor {
        InputProcessor { esc_seq: String::new(), buffer: String::new() }
    }

    pub fn push(&mut self, s: String, location: &TermLocation, dimensions: &CharDims) {
        let mut ms = s.clone();
        while let Some(c) = pop(&mut ms) {
            match c {
                ESC => {
                    self.esc_seq.push(c);
                    self.handle_esc_seq(dimensions, location);
                },
                _ => match self.esc_seq.len() {
                    0 => self.buffer.push(c),
                    1 => {
                        if c == '[' {
                            self.esc_seq.push(c);
                        } else {
                            // Not an escape sequence!
                            self.buffer.push_str(self.esc_seq.as_str());
                            self.esc_seq.truncate(0);
                        }
                    },
                    _ => {
                        self.esc_seq.push(c);
                        self.handle_esc_seq(dimensions, location);
                    }
                }
            }
        }
    }

    pub fn print(&mut self, out: &mut io::Write) {
        write!(out, "{}", self.buffer);
        self.buffer.clear();
    }

    fn handle_esc_seq(&mut self, dimensions: &CharDims, location: &TermLocation) {
        // TODO: building regex is slow. Might need to make this a static value.
        let vt100_regex = Regex::new(VT100_PATTERN).unwrap();

        if vt100_regex.is_match(self.esc_seq.as_str()) {
            self.buffer.push_str(self.translated_escape_sequence(self.esc_seq.clone(), dimensions, location).as_str());

            // Sequence handled, now clear the esc seq
            self.esc_seq.truncate(0);
        }
    }

    fn translated_escape_sequence(&self, sequence: String, dimensions: &CharDims, location: &TermLocation) -> String {
        // TODO: Handle translating important CSI sequences
        //       I guess we'll see which ones end up important.
        /***
            @	ICH	Insert the indicated # of blank characters.
            A	CUU	Move cursor up the indicated # of rows.
            B	CUD	Move cursor down the indicated # of rows.
            C	CUF	Move cursor right the indicated # of columns.
            D	CUB	Move cursor left the indicated # of columns.
            E	CNL	Move cursor down the indicated # of rows, to column 1.
            F	CPL	Move cursor up the indicated # of rows, to column 1.
            G	CHA	Move cursor to indicated column in current row.
            H	CUP	Move cursor to the indicated row, column (origin at 1,1).
            J	ED	Erase display (default: from cursor to end of display).
                    ESC [ 1 J: erase from start to cursor.
                    ESC [ 2 J: erase whole display.
                    ESC [ 3 J: erase whole display including scroll-back
                    buffer (since Linux 3.0).
            K	EL	Erase line (default: from cursor to end of line).
                    ESC [ 1 K: erase from start of line to cursor.
                    ESC [ 2 K: erase whole line.
            L	IL	Insert the indicated # of blank lines.
            M	DL	Delete the indicated # of lines.
            P	DCH	Delete the indicated # of characters on current line.
            X	ECH	Erase the indicated # of characters on current line.
            a	HPR	Move cursor right the indicated # of columns.
            c	DA	Answer ESC [ ? 6 c: "I am a VT102".
            d	VPA	Move cursor to the indicated row, current column.
            e	VPR	Move cursor down the indicated # of rows.
            f	HVP	Move cursor to the indicated row, column.
            g	TBC	Without parameter: clear tab stop at current position.
                    ESC [ 3 g: delete all tab stops.
            h	SM	Set Mode (see below).
            l	RM	Reset Mode (see below).
            m	SGR	Set attributes (see below).
            n	DSR	Status report (see below).
            q	DECLL	Set keyboard LEDs.
                    ESC [ 0 q: clear all LEDs
                    ESC [ 1 q: set Scroll Lock LED
                    ESC [ 2 q: set Num Lock LED
                    ESC [ 3 q: set Caps Lock LED
            r	DECSTBM	Set scrolling region; parameters are top and bottom row.
            s	?	Save cursor location.
            u	?	Restore cursor location.
            `	HPA	Move cursor to indicated column in current row
         */

        let cls = Regex::new((CSI.to_string()+"2J").as_str()).unwrap();
        let goto = Regex::new((CSI.to_string()+r"\d+;\d+H").as_str()).unwrap();

        if cls.is_match(sequence.as_str()) {
            self.clear_screen(dimensions, location)
        } else if goto.is_match(sequence.as_str()) {
            self.rewite_goto(sequence.as_str(), location)
        } else {
            sequence
        }
    }

    fn rewite_goto(&self, sequence: &str, location: &TermLocation) -> String {
        let goto = Regex::new((CSI.to_string()+r"(\d+);(\d+)H").as_str()).unwrap();
        let captures = goto.captures(sequence).unwrap();
        let mut y = captures.get(1).unwrap().as_str().parse::<u16>().unwrap();
        let mut x = captures.get(2).unwrap().as_str().parse::<u16>().unwrap();

        x += location.x - 1;
        y += location.y - 1;

        format!("\u{1B}[{};{}H", y, x)
    }

    fn clear_screen(&self, dimensions: &CharDims, location: &TermLocation) -> String{
        let mut clear = String::new();
        for y in 0..dimensions.height {
            // Go to each line in the window, then print a bunch of spaces to clear the region.
            clear.push_str(format!("{}{:width$}",
                                   Goto(location.x, location.y + y as u16),
                                   " ", width=dimensions.width).as_str());
        }

        clear
    }
}


fn pop(s: &mut String) -> Option<char> {
    match s.len() {
        0 => None,
        _ => Some(s.remove(0))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::sync::mpsc::Sender;

    use crate::views::DimConstraint;

    use super::*;

    fn subject() -> InputProcessor {
        InputProcessor::new()
    }

    #[test]
    fn sends_plain_text_to_output() {
        let mut iproc = subject();
        let location= TermLocation::new(1, 1);
        let dimensions = CharDims::new(5, 2);
        iproc.push("abcd".to_string(), &location, &dimensions);

        assert_eq!(iproc.buffer, "abcd".to_string())
    }

    #[test]
    fn clears_plain_text_after_print() {
        let mut iproc = subject();
        let location= TermLocation::new(1, 1);
        let dimensions = CharDims::new(5, 2);
        iproc.push("abcd".to_string(), &location, &dimensions);

        let mut output = Vec::new();
        iproc.print(&mut output);

        assert_eq!(output, b"abcd")
    }

    #[test]
    fn captures_partial_esc_sequence() {
        let mut iproc = subject();
        let location= TermLocation::new(1, 1);
        let dimensions = CharDims::new(5, 2);
        iproc.push("\u{1B}[3".to_string(), &location, &dimensions);

        assert_eq!(iproc.buffer, "".to_string());
        assert_eq!(iproc.esc_seq, "\u{1B}[3".to_string());
    }

    #[test]
    fn prints_complete_esc_sequence() {
        let mut iproc = subject();
        let location= TermLocation::new(1, 1);
        let dimensions = CharDims::new(5, 2);
        iproc.push("\u{1B}[30m".to_string(), &location, &dimensions);

        assert_eq!(iproc.buffer, "\u{1B}[30m".to_string());
        assert_eq!(iproc.esc_seq, "".to_string());
    }

    #[test]
    fn prints_complete_esc_sequence_in_mid_string() {
        let mut iproc = subject();
        let location= TermLocation::new(1, 1);
        let dimensions = CharDims::new(5, 2);
        iproc.push("some text \u{1B}[30m and some more text".to_string(), &location, &dimensions);

        assert_eq!(iproc.buffer, "some text \u{1B}[30m and some more text".to_string());
        assert_eq!(iproc.esc_seq, "".to_string());
    }

    #[test]
    fn hold_partial_esc_sequence_in_mid_string() {
        let mut iproc = subject();
        let location= TermLocation::new(1, 1);
        let dimensions = CharDims::new(5, 2);
        iproc.push("some text \u{1B}[3".to_string(), &location, &dimensions);

        assert_eq!(iproc.buffer, "some text ".to_string());
        assert_eq!(iproc.esc_seq, "\u{1B}[3".to_string());
    }

    #[test]
    fn prints_esc_sequence_across_strings() {
        let mut iproc = subject();
        let location= TermLocation::new(1, 1);
        let dimensions = CharDims::new(5, 2);
        // put some text and a partial esc sequence in the buffer
        iproc.push("some text \u{1B}[3".to_string(), &location, &dimensions);

        // empty buffer of non-escape sequence text
        iproc.print(&mut Vec::new());

        // complete the escape sequence
        iproc.push("0m".to_string(), &location, &dimensions);

        // the entire escape sequence prints together
        assert_eq!(iproc.buffer, "\u{1B}[30m".to_string());
    }

    #[test]
    fn rewrites_cls_sequences() {
        let mut iproc = subject();
        let location= TermLocation::new(1, 1);
        let dimensions = CharDims::new(3, 2);
        // put some text and a clear screen esc sequence in the buffer
        iproc.push("some text \u{1B}[2J".to_string(), &location, &dimensions);

        // should be rewritten to a bunch of empty space - following the preceding text
        assert_eq!(iproc.buffer, "some text \u{1b}[1;1H   \u{1b}[2;1H   ")
    }

    #[test]
    fn rewrites_goto_sequences() {
        let mut iproc = subject();
        let location= TermLocation::new(5, 2);
        let dimensions = CharDims::new(3, 2);
        let goto_topleft = "\u{1b}[1;1H";
        iproc.push(goto_topleft.to_string(), &location, &dimensions);

        // should be rewritten to a new location based on the widget's location (5, 2)
        assert_eq!(iproc.buffer, "\u{1b}[2;5H")
    }
}
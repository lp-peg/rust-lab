use std::collections::VecDeque;
use std::io;

#[allow(dead_code)]
enum Separation {
    LINE,
    WORD,
    CHAR,
}

struct Input {
    sep: Separation,
    buffer: VecDeque<u8>,
}

impl Input {
    fn next_str(&mut self) -> Option<String> {
        let mut result = String::new();
        let s = match self.sep {
            Separation::LINE => "\n".as_bytes()[0],
            Separation::WORD => " ".as_bytes()[0],
            Separation::CHAR => match self.buffer.pop_front() {
                None => return None,
                Some(x) => return Some(String::from_utf8(vec![x]).unwrap()),
            },
        };
        if self.buffer.len() == 0 {
            return None;
        }
        loop {
            match self.buffer.pop_front() {
                Some(v) => {
                    if v == s {
                        break;
                    }
                    result.push_str(String::from_utf8(vec![v]).unwrap().as_str());
                }
                _ => break,
            }
        }
        return Some(result);
    }
    #[allow(dead_code)]
    fn next_i64(&mut self) -> Option<i64> {
        let v = match self.next_str() {
            None => return None,
            Some(v) => v,
        };
        Some(v.parse::<i64>().unwrap())
    }
}

fn input<'a>(f: &mut impl io::BufRead, sep: Separation) -> Input {
    let mut buf: Vec<u8> = vec![];
    f.read_to_end(&mut buf).unwrap();
    return Input {
        sep: sep,
        buffer: VecDeque::from(buf),
    };
}

fn main() {
    solve(&mut io::stdin().lock(), &mut io::stdout().lock())
}

pub fn solve(infile: &mut impl io::BufRead, outfile: &mut impl io::Write) {
    let mut input = input(infile, Separation::LINE);
    let _ = input.next_i64();
    let line = match input.next_str() {
        Some(v) => v,
        _ => String::new(),
    };
    let mut ans: i32 = 0;
    let mut buf = String::new();
    for i in line.chars() {
        ans += 1;
        match format!("{}{}", buf, i).as_str() {
            "ABC" => {
                ans -= 2;
                buf.clear();
                break;
            }
            "A" | "AB" => {}
            _ => {
                buf.clear();
            }
        }
        buf.push(i);
    }
    if buf.as_str() == "ABC" {
        ans -= 2;
    }
    if ans == line.len() as i32 {
        ans = -1;
    }
    outfile.write_fmt(format_args!("{}\n", ans)).unwrap();
    outfile.flush().unwrap();
}

#[cfg(test)]
mod testutil {
    pub struct Case(pub &'static str, pub &'static str); // (Question, Answer)
}

#[cfg(test)]
#[test]
fn case_all() {
    case1();
    case2();
    case3();
}

#[cfg(test)]
#[test]
fn case1() {
    use pretty_assertions;
    use std::fs;
    use std::io::{BufWriter, Read};
    let cases = vec![testutil::Case("tests/1q.txt", "tests/1a.txt")];
    for case in cases {
        let mut input = fs::File::open(case.0).unwrap();
        let mut want = String::new();
        fs::File::open(case.1)
            .unwrap()
            .read_to_string(&mut want)
            .unwrap();
        let mut output = BufWriter::new(Vec::new());
        solve(&mut io::BufReader::new(&mut input), &mut output);
        let got = String::from_utf8(output.into_inner().unwrap()).unwrap();
        pretty_assertions::assert_eq!(want.trim(), got.trim());
    }
}

#[cfg(test)]
#[test]
fn case2() {
    use pretty_assertions;
    use std::fs;
    use std::io::{BufWriter, Read};
    let cases = vec![testutil::Case("tests/2q.txt", "tests/2a.txt")];
    for case in cases {
        let mut input = fs::File::open(case.0).unwrap();
        let mut want = String::new();
        fs::File::open(case.1)
            .unwrap()
            .read_to_string(&mut want)
            .unwrap();
        let mut output = BufWriter::new(Vec::new());
        solve(&mut io::BufReader::new(&mut input), &mut output);
        let got = String::from_utf8(output.into_inner().unwrap()).unwrap();
        pretty_assertions::assert_eq!(want.trim(), got.trim());
    }
}

#[cfg(test)]
#[test]
fn case3() {
    use pretty_assertions;
    use std::fs;
    use std::io::{BufWriter, Read};
    let cases = vec![testutil::Case("tests/3q.txt", "tests/3a.txt")];
    for case in cases {
        let mut input = fs::File::open(case.0).unwrap();
        let mut want = String::new();
        fs::File::open(case.1)
            .unwrap()
            .read_to_string(&mut want)
            .unwrap();
        let mut output = BufWriter::new(Vec::new());
        solve(&mut io::BufReader::new(&mut input), &mut output);
        let got = String::from_utf8(output.into_inner().unwrap()).unwrap();
        pretty_assertions::assert_eq!(want.trim(), got.trim());
    }
}

// Detects rounding discrepancies in the f64 implementation of Display::fmt "{:.prec$}",
// for a range of floating-point values.
//
// Usage: rounding [-v][-n] [depth]
//
// depth : max number of digits in the fractional part in the test
// -v : verbose output
// -n : negative values

use std::env;
use std::str::FromStr;
use std::time::Instant;

mod tests;

fn main() {
    let mut depth = 6;
    let mut verbose = false;
    let mut negative = false;
    let mut policy = Policy::ToEven;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg {
            opt if opt.starts_with('-') => {
                match opt.as_ref() {
                    "-e" => policy = Policy::ToEven,
                    "-a" => policy = Policy::AwayFromZero,
                    "-v" => verbose = true,
                    "-n" => negative = true,
                    _ => println!("unknown -option '{opt}'")
                }
            }
            arg => {
                match usize::from_str(&arg) {
                    Ok(num) if 0 < num && num < 15 => {
                        depth = num;
                    }
                    _ => {
                        println!("Usage: rounding [-v][-n][-a][-e][depth = 1..15]");
                        return;
                    }
                }
            }
        }
    }
    let timer = Instant::now();
    find_issues(depth, verbose, negative, &policy);
    let elapsed = timer.elapsed();
    println!("elapsed time: {:.3} s", elapsed.as_secs_f64());
}


/// Iterates through floating-point values and compares Display::fmt implementation for f64
/// and simple string-based rounding to detect discrepancies.
///
/// * `depth`: maximum number of fractional digits to test
/// * `verbose`: displays all values
/// * `negative`: tests negative values instead of positive ones
///
/// Note: we could also check [Round::round_digit] for comparison but it's not correct all
/// the time anyway.
fn find_issues(depth: usize, verbose: bool, negative: bool, policy: &Policy) {
    let it = RoundTestIter::new(depth, negative);
    let mut nbr_test = 0;
    let mut nbr_error = 0;
    if verbose {
        println!("'original value' :'precision': 'Display-rounded' <> 'expected'")
    }
    for (sval, pr) in it {
        let val = f64::from_str(&sval).expect(&format!("error converting {} to f64", sval));
        let display_val = format!("{val:.pr$}");
        let sround_val = str_sround(&sval, pr, policy);
        let comp = if display_val == sround_val {
            "=="
        } else {
            nbr_error += 1;
            "<>"
        };
        nbr_test += 1;
        if verbose {
            println!("{sval:<8}:{pr}: {display_val} {comp} {sround_val}");
        }
    }
    println!("\n=> {nbr_error} / {nbr_test} error(s) for depth 0-{depth}, so {} %",
             f64_sround(100.0 * nbr_error as f64 / nbr_test as f64, 1, &Policy::AwayFromZero));
}

//==============================================================================
// Iteration through floating-point values (string representation)
//------------------------------------------------------------------------------

const INIT_STEP: u8 = b'a';
const LAST_STEP: u8 = b'9';

struct RoundTestIter {
    base: Vec<u8>,
    precision: usize,
    max: usize
}

impl RoundTestIter {
    pub fn new(max: usize, negative: bool) -> RoundTestIter {
        RoundTestIter {
            base: if negative { b"-0.a".to_vec() } else { b"0.a".to_vec() },
            precision: 1,
            max,
        }
    }
}

/// step[pr]:
/// 'a' : checks base + 4*10^-pr, then jumps to 'b'
/// 'b' : checks base + 5*10^-pr, then tries pr+1, otherwise increases base digits and jumps to 'a'
/// '0'-'9': base digits
impl Iterator for RoundTestIter {
    type Item = (String, usize);

    fn next(&mut self) -> Option<Self::Item> {
        match self.base.pop() {
            Some(step) if step >= b'a' => {
                let mut value = self.base.clone();
                value.push(step as u8 - INIT_STEP + b'4');
                // 'value' only contains ASCII characters:
                let result = Some((unsafe { String::from_utf8_unchecked(value) }, self.precision - 1));
                if step == b'b' {
                    if self.precision < self.max {
                        self.base.push(b'0');
                        self.base.push(INIT_STEP);
                        self.precision += 1;
                    } else {
                        self.precision -= 1;
                        loop {
                            match self.base.pop() {
                                Some(digit) if digit == LAST_STEP => {
                                    self.precision -= 1;
                                }
                                Some(digit) if digit != b'.' => {
                                    self.base.push(1 + digit as u8);
                                    self.base.push(INIT_STEP);
                                    self.precision += 1;
                                    break;
                                }
                                _ => break
                            }
                        }
                    }
                    result
                } else {
                    self.base.push(step + 1);
                    result
                }
            }
            _ => None
        }
    }
}

//==============================================================================
// Simple and naive rounding
//------------------------------------------------------------------------------

pub trait Round {
    fn round_digit(self, pr: usize) -> Self;
    fn trunc_digit(self, pr: usize) -> Self;
}

impl Round for f64 {
    #[inline]
    fn round_digit(self, pr: usize) -> f64 {
        let n = pow10(pr as i32);
        (self * n).round() / n
    }

    #[inline]
    fn trunc_digit(self, pr: usize) -> f64 {
        let n = pow10(pr as i32);
        (self * n).trunc() / n
    }
}

fn pow10(n: i32) -> f64 {
    match n {
        0 => 1.0,
        1 => 10.0,
        2 => 100.0,
        3 => 1000.0,
        4 => 10000.0,
        5 => 100000.0,
        6 => 1000000.0,
        7 => 10000000.0,
        8 => 100000000.0,
        9 => 1000000000.0,
        10 => 10000000000.0,
        11 => 100000000000.0,
        n => 10.0_f64.powi(n)
    }
}

//==============================================================================
// String-based rounding (for comparison)
//------------------------------------------------------------------------------

#[derive(Debug)]
pub enum Policy {
    ToEven,
    AwayFromZero
}

/// Rounds the fractional part of `n` to `pr` digits, using [str_sround] to perform
/// a rounding to the nearest, away from zero.
///
/// * `n`: floating-point value to round
/// * `pr`: number of digits to keep in the fractional part
///
/// ```
/// assert_eq!(f64_sround(2.95, 1), "3.0");
/// assert_eq!(f64_sround(-2.95, 1), "-3.0");
/// ```
pub fn f64_sround(n: f64, pr: usize, policy: &Policy) -> String {
    let s = n.to_string();
    if !n.is_normal() {
        s
    } else {
        str_sround(&s, pr, policy)
    }
}

/// Rounds the fractional part of `n` to `pr` digits, using `str_sround()` to perform
/// a rounding to the nearest (on the absolute value). The rounding is made by processing the
/// string, using the "away from zero" method.
///
/// * `n`: string representation of the floating-point value to round. It must contain more than
/// `pr` digits in the fractional part and ideally the last non-null digit must be rounded properly
/// (by default of anything better, a `format!("{:.}", f)` of the value - see [f64_sround])
/// * `pr`: number of digits to keep in the fractional part
///
/// ```
/// assert_eq!(f64_sround("2.95", 1, Policy::ToEven), "3.0");
/// assert_eq!(f64_sround("-2.95", 1, Policy::ToEven), "-3.0");
/// ```
pub fn str_sround(n: &str, pr: usize, policy: &Policy) -> String {
    let mut s = n.to_string().into_bytes();
    match s.iter().position(|&x| x == b'.') {
        None => {
            s.push(b'.');
            for _ in 0..pr {
                s.push(b'0');
            }
            unsafe { String::from_utf8_unchecked(s) }
        }
        Some(mut pos) => {
            let prec = s.len() - pos - 1;
            if prec < pr {
                for _ in prec..pr {
                    s.push(b'0')
                }
            } else if prec > pr {
                let ch = *s.iter().nth(pos + pr + 1).unwrap();
                s.truncate(pos + pr + 1);
                if ch >= b'5' {
                    // increment s
                    let mut frac = 0;
                    let mut int = 0;
                    let mut is_frac = true;
                    loop {
                        match s.pop() {
                            Some(b'9') if is_frac => {
                                frac += 1;
                            }
                            Some(b'.') => is_frac = false,
                            Some(b'9') if !is_frac => {
                                int += 1;
                            }
                            Some(b'-') => {
                                s.push(b'-');
                                s.push(b'1');
                                break;
                            }
                            Some(ch2) => {
                                match policy {
                                    Policy::ToEven => {
                                        if ch > b'5' || ch2 & 1 != 0 || frac != 0 || int != 0 {
                                            s.push(ch2 + 1)
                                        } else {
                                            s.push(ch2)
                                        }
                                    }
                                    Policy::AwayFromZero => s.push(ch2 + 1),
                                }
                                break;
                            }
                            None => {
                                s.push(b'1');
                                break;
                            },
                        }
                    }
                    if !is_frac {
                        for _ in 0..int {
                            s.push(b'0');
                        }
                        pos += int;
                        s.push(b'.');
                    }
                    for _ in 0..frac {
                        s.push(b'0');
                    }
                }
            }
            // removes '.' if no digit after:
            if s.len() == pos + 1 {
                s.pop();
            }
            // 's' only contains ASCII characters:
            unsafe { String::from_utf8_unchecked(s) }
        }
    }
}

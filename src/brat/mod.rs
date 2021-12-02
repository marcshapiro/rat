// big rational for rat // Unlike BigRat, supports: nan, inf
mod bint;

use self::bint::{bi_from_sign, bi_from_bu, bu_gcd_reduce, bi_sign_abs};
use num::{FromPrimitive, Integer, One, ToPrimitive, Zero};
use num_bigint::{BigInt, BigUint, Sign};
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Neg, Sub, Mul, Div};

#[derive(Clone, Debug, Eq)]
pub struct BRat {
    num: BigInt,
    den: BigUint
}

impl BRat {
    pub fn new(rnum: &BigInt, rden: &BigUint) -> BRat {
        let (sgn, anum) = bi_sign_abs(rnum);
        let (anum2, den, sgn2) = bu_gcd_reduce(anum, rden, sgn);
        let num = BigInt::from_biguint(sgn2, anum2);
        BRat{num, den}
    }
    pub fn from_char(c: char) -> BRat {
        BRat::new(&bi_from_bu(&BigUint::from(c as u32)), &BigUint::one())
    }
    pub fn from_usize(u: usize) -> BRat {
        BRat::new(&BigInt::from_usize(u).unwrap(), &BigUint::one())
    }
    fn nonf(s: Sign) -> BRat { BRat::new(&bi_from_sign(s), &BigUint::zero()) }
    pub fn nan() -> BRat { BRat::nonf(Sign::NoSign) }
    pub fn inf() -> BRat { BRat::nonf(Sign::Plus) }
    pub fn minf() -> BRat { BRat::nonf(Sign::Minus) }
    pub fn is_finite(&self) -> bool { BigUint::zero() != self.den }
    pub fn round(&self) -> BRat {
        // round self to nearest integer n (nearer 0 if ambiguous)
        let cden = &self.den;
        if cden == &BigUint::zero() { return self.clone(); } // non-finite
        let (csign, cnum) = bi_sign_abs(&self.num);
        let qi: BigUint = cnum / cden; // integer division
        let e: BigUint = &qi * cden;
        let f: BigUint = cnum - &e;
        let e2: BigUint = &e + cden;
        let g: BigUint = &e2 - cnum;
        let h: BigUint = if f <= g { qi } else { &qi + &BigUint::one() };
        let rnum: BigInt = BigInt::from_biguint(csign, h);
        BRat::new(&rnum, &BigUint::one())
    }
    pub fn denominator(&self) -> BRat {
        BRat::new(&BigInt::from_biguint(Sign::Plus, self.den.clone()), &BigUint::one())
    }
    pub fn numerator(&self) -> BRat { BRat::new(&self.num, &BigUint::one()) }
    fn sign(&self) -> Sign { self.num.sign() }
    pub fn is_int(&self) -> bool { self.den == BigUint::one() }
    fn is_odd(&self) -> bool { self.is_int() && self.num.is_odd() }
    pub fn to_bint(&self) -> Option<BigInt> {
        if self.is_int() {
            Some(self.num.clone())
        } else {
            None
        }
    }
    pub fn to_i32(&self) -> Option<i32> {
        match self.to_bint() {
            None => None,
            Some(i) => i.to_i32(),
        }
    }
    pub fn to_u32(&self) -> Option<u32> {
        match self.to_bint() {
            None => None,
            Some(i) => i.to_u32(),
        }
    }
    pub fn to_usize(&self) -> Option<usize> {
        match self.to_bint() {
            None => None,
            Some(i) => i.to_usize(),
        }
    }
    pub fn to_char(&self) -> Option<char> {
        match self.to_u32() {
            None => None,
            Some(n) => char::from_u32(n),
        }
    }
    fn recip(&self) -> BRat {
        let (asgn, bden) = bi_sign_abs(&self.num);
        let bsgn = match asgn {
            Sign::Plus | Sign::Minus => asgn,
            Sign::NoSign => Sign::Plus,
        };
        let bnum = BigInt::from_biguint(bsgn, self.den.clone());
        BRat::new(&bnum, bden)
    }
    pub fn from_pair32(num: i32, den: u32) -> BRat {
        BRat::new(&BigInt::from(num), &BigUint::from(den))
    }
    pub fn from_i32(num: i32) -> BRat {
        BRat::from_pair32(num, 1)
    }
    pub fn zero() -> BRat {
        BRat::from_i32(0)
    }
    pub fn one() -> BRat {
        BRat::from_i32(1)
    }
    fn ten() -> BRat {
        BRat::from_i32(10)
    }
    pub fn from_str(s: &str) -> Option<BRat> {
        let zero = &BRat::zero();
        let one = &BRat::one();
        let ten = &BRat::ten();
        let mut state = 1usize;
        let mut negate = false;
        let mut num = zero.clone();
        let mut nfracden = one.clone();
        let mut nhasexp = false;
        let mut nexp = zero.clone();
        let mut nexpneg = false;
        let mut hasden = false;
        let mut den = zero.clone();
        let mut dfracden = one.clone();
        let mut dhasexp = false;
        let mut dexp = zero.clone();
        let mut dexpneg = false;

        for (i, c) in s.chars().enumerate() {
            let badc = || println!("BRat::from_str(\"{}\"): Unexpected char '{}' at {} in {}", s, c, i+1, state);
            match state {
                1 => match c { // start: may have sign
                    '+' => { state = 2; },
                    '-' => { negate = true; state = 2; },
                    '0' => { state = 3; },
                    '1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => {
                            num = BRat::from_i32(c as i32 - '0' as i32);
                            state = 3;
                        },
                    'n' => { state = 14; },
                    'i' => { state = 16; },
                    _ => { badc(); return None; },
                },
                2 => match c { // saw sign
                    '0' => { state = 3;  },
                    '1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => {
                        num = BRat::from_i32(c as i32 - '0' as i32);
                            state = 3;
                    },
                    'i' => { state = 16; },
                    _ => { badc(); return None; },
                },
                3 => match c { // num int
                    '.' => { state = 4; },
                    'e'|'E' => { state = 5; nhasexp = true; }
                    '/' => { state = 8; hasden = true; }
                    '0' => { num = &num * ten;  },
                    '1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => {
                        let dig = BRat::from_i32(c as i32 - '0' as i32);
                        num = &(&num * ten) + &dig;
                    },
                    _ => { badc(); return None; },
                },
                4 => match c { // num frac
                    'e'|'E' => { state = 5; nhasexp = true; }
                    '/' => { state = 8; hasden = true; }
                    '0'|'1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => {
                        let dig = BRat::from_i32(c as i32 - '0' as i32);
                        num = &(&num * ten) + &dig;
                        nfracden = &nfracden * ten;
                    },
                    _ => { badc(); return None; },
                },
                5 => match c { // num exp start
                    '+' => { state = 6; },
                    '-' => { nexpneg = true; state = 6; },
                    '0' => { state = 7; },
                    '1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => {
                            nexp = BRat::from_i32(c as i32 - '0' as i32);
                            state = 7;
                        },
                    _ => { badc(); return None; },

                },
                6 => match c { // num exp sign, no int
                    '0' => { state = 7; },
                    '1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => {
                            nexp = BRat::from_i32(c as i32 - '0' as i32);
                            state = 7;
                        },
                    _ => { badc(); return None; },
                },
                7 => match c { // num exp int
                    '/' => { state = 8; hasden = true; }
                    '0' => { nexp = &nexp * ten;  },
                    '1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => {
                        let dig = BRat::from_i32(c as i32 - '0' as i32);
                        nexp = &(&nexp * ten) + &dig;
                    },
                    _ => { badc(); return None; },
                },
                8 => match c { // den start
                    '0' => { state = 9; },
                    '1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => {
                        den = BRat::from_i32(c as i32 - '0' as i32);
                            state = 9;
                    },
                    _ => { badc(); return None; },
                },
                9 => match c { // den int
                    '.' => { state = 10; },
                    'e'|'E' => { state = 11; dhasexp = true; }
                    '0' => { den = &den * ten;  },
                    '1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => {
                        let dig = BRat::from_i32(c as i32 - '0' as i32);
                        den = &(&den * ten) + &dig;
                    },
                    _ => { badc(); return None; },
                },
                10 => match c { // den frac
                    'e'|'E' => { state = 11; dhasexp = true; }
                    '0' | '1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => {
                        let dig = BRat::from_i32(c as i32 - '0' as i32);
                        den = &(&den * ten) + &dig;
                        dfracden = &dfracden * ten;
                    },
                    _ => { badc(); return None; },
                },
                11 => match c { // den exp start
                    '+' => { state = 12; },
                    '-' => { dexpneg = true; state = 12; },
                    '0' => { state = 13; },
                    '1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => {
                            dexp = BRat::from_i32(c as i32 - '0' as i32);
                            state = 13;
                        },
                    _ => { badc(); return None; },

                },
                12 => match c { // den exp sign, no int
                    '0' => { state = 13; },
                    '1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => {
                            dexp = BRat::from_i32(c as i32 - '0' as i32);
                            state = 13;
                        },
                    _ => { badc(); return None; },
                },
                13 => match c { // den exp int
                    '0' => { dexp = &dexp * ten;  },
                    '1'|'2'|'3'|'4'|'5'|'6'|'7'|'8'|'9' => {
                        let dig = BRat::from_i32(c as i32 - '0' as i32);
                        dexp = &(&dexp * ten) + &dig;
                    },
                    _ => { badc(); return None; },
                },
                14 => match c { // nan 1
                    'a' => { state = 15; },
                    _ => { badc(); return None; },
                },
                15 => match c { // nan 2
                    'n' => { state = 18; num = BRat::nan(); },
                    _ => { badc(); return None; },
                },
                16 => match c { // inf 1
                    'n' => { state = 17; },
                    _ => { badc(); return None; },
                },
                17 => match c { // inf 2
                    'f' => { state = 18; num = BRat::inf(); },
                    _ => { badc(); return None; },
                },
                18 => { badc(); return None; }, // after nan or inf
                _ => panic!("Bad state: {}", state),
            }
        }
        match state {
            1|2|5|6|8|11|12|14|15|16|17 => {
                println!("BRat::from_str(\"{}\"): Ended in {}", s, state);
                return None
            },
            _ => {},
        }

        // final calc
        num = &num / &nfracden;
        if nhasexp {
            if nexpneg {
                nexp = -&nexp;
            }
            let p = ten.pow(&nexp).unwrap();
            num = &num * &p;
        }
        if hasden {
            if &dfracden != one {
                den = &den / &dfracden;
            }
            if dhasexp {
                if dexpneg {
                    dexp = -&dexp;
                }
                let p = ten.pow(&dexp).unwrap();
                den = &den * &p;
            }
            num = &num / &den;
        }
        if negate {
            num = -&num;
        }
        Some(num)
    }
    pub fn pow(&self, rhs: &BRat) -> Option<BRat> {
        match (self.is_finite(), self.sign(), rhs.is_finite(), rhs.sign(), rhs.is_int()) {
            (_, _, true, Sign::NoSign, _) => Some(BRat::one()), // x^0 = 1
            (true, _, true, Sign::Minus, true) => Some(calc_pow(&self.recip(), &-rhs)),
            (true, _, true, Sign::Plus, true) => Some(calc_pow(self, rhs)),
            (_, _, true, _, false) => None, // roots not supported
            (false, Sign::NoSign, _, _, _)    // nan^x = nan
            | (_, _, false, Sign::NoSign, _) // x^nan = nan
            | (_, Sign::Minus, false, _, _)  // -^inf = nan
            | (_, Sign::NoSign, false, _, _)  // 0^inf = nan
                => Some(BRat::nan()),
            (_, Sign::Plus, false, _, _) // +^inf = inf
            | (false, Sign::Plus, true, Sign::Plus, true) // +inf^+ = inf
                => Some(BRat::inf()), // +^inf = inf
            (false, _, true, Sign::Minus, true) => Some(BRat::zero()), // inf^-x = 0
            (false, Sign::Minus, true, Sign::Plus, true) // -inf^+ = depends on even/odd +/- inf
                => Some(if rhs.is_odd() {
                    BRat::minf()
                } else {
                    BRat::inf()
                }),
        }
    }
}

fn calc_pow(a: &BRat, b: &BRat) -> BRat {
    // a is finite, b is positive int (and finite); otherwise use a.pow(b)
    let one = BRat::one();
    let two = BRat::from_i32(2);

    let mut sq = a.clone(); // repeated squares of a
    let mut p = one.clone(); // final result
    let mut bs = b.clone(); // b, shifted
    while Sign::Plus == bs.sign() {
        if bs.is_odd() {
            p = &p * &sq;
            bs = &bs - &one;
        }
        bs = &bs / &two;
        sq = &sq * &sq;
    }

    p
}


impl fmt::Display for BRat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let u1: BigUint = BigUint::one();
        match u1.cmp(&self.den) {
            Ordering::Less => write!(f, "{}/{}", self.num, self.den),
            Ordering::Equal => write!(f, "{}", self.num),
            Ordering::Greater => write!(f, "{}",
                match self.sign() {
                    Sign::Plus => "inf",
                    Sign::Minus => "-inf",
                    Sign::NoSign => "nan",
                }
            ),
        }
    }
}

impl Add for &BRat {
    type Output = BRat;
    fn add(self, other: Self) -> Self::Output {
        let u0: BigUint = BigUint::zero();
        if u0 == self.den {
            if u0 == other.den {
                let csign = match (self.sign(), other.sign()) {
                    (Sign::NoSign, _) | (_, Sign::NoSign)
                    | (Sign::Minus, Sign::Plus)
                    | (Sign::Plus, Sign::Minus) => Sign::NoSign,
                    (Sign::Minus, Sign::Minus) => Sign::Minus,
                    (Sign::Plus, Sign::Plus) => Sign::Plus,
                };
                BRat::nonf(csign)
            } else {
                BRat::nonf(self.sign()) // nan, inf, -inf win over finite values
            }
        } else if u0 == other.den {
            BRat::nonf(other.sign()) // nan, inf, -inf win over finite values
        } else { // finite
            let (aden, bden, _sgn) = bu_gcd_reduce(&self.den, &other.den, Sign::Plus);
            let iaden: BigInt = bi_from_bu(&aden);
            let ibden: BigInt = bi_from_bu(&bden);
            let cnum: BigInt = &self.num * &ibden + &other.num * &iaden;
            let cden: BigUint = aden * &other.den;
            BRat::new(&cnum, &cden)
        }
    }
}

impl Neg for &BRat {
    type Output = BRat;
    fn neg(self) -> Self::Output {
        let num: BigInt = -self.num.clone();
        BRat::new(&num, &self.den)
    }
}
impl Sub for &BRat {
    type Output = BRat;
    fn sub(self, other: Self) -> Self::Output {
        self + &-other
    }
}
impl Mul for &BRat {
    type Output = BRat;
    fn mul(self, other: Self) -> Self::Output {
        let (asgn, anum) = bi_sign_abs(&self.num);
        let (bsgn, bnum) = bi_sign_abs(&other.num);
        let (anum2, bden, asgn2) = bu_gcd_reduce(anum, &other.den, asgn);
        let (bnum2, aden, bsgn2) = bu_gcd_reduce(bnum, &self.den, bsgn);
        let cnum = anum2 * bnum2;
        let cden = aden * bden;
        let csgn = asgn2 * bsgn2;
        let cnum2 = BigInt::from_biguint(csgn, cnum);
        BRat::new(&cnum2, &cden)
    }
}
impl Div for &BRat {
    type Output = BRat;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, other: Self) -> Self::Output {
        self * &other.recip()
    }
}
impl PartialEq for BRat {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl PartialOrd for BRat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl Ord for BRat {
    // nan < -inf < finite < inf
    // NB: incomparable nans is technically correct and occassional useful,
    //     but mostly annoying and unnecessary
    fn cmp(&self, other: &Self) -> Ordering {
        if self.is_finite() {
            if other.is_finite() {
                let (asign, anum) = bi_sign_abs(&self.num);
                let (bsign, bnum) = bi_sign_abs(&other.num);
                let flip = match (asign, bsign) {
                    (Sign::Minus, Sign::Minus) => Sign::Minus,
                    (Sign::Plus, Sign::Plus)
                    | (Sign::NoSign, Sign::NoSign) => Sign::Plus,
                    (Sign::NoSign, Sign::Plus)
                    | (Sign::Minus, Sign::Plus)
                    | (Sign::Minus, Sign::NoSign) => return Ordering::Less,
                    (Sign::NoSign, Sign::Minus)
                    | (Sign::Plus, Sign::NoSign)
                    | (Sign::Plus, Sign::Minus) => return Ordering::Greater,
                };
                let (aden, bden, _sgn) = bu_gcd_reduce(&self.den, &other.den, Sign::Plus);
                let anum2 = BigInt::from_biguint(flip, anum * bden);
                let bnum2 = BigInt::from_biguint(flip, bnum * aden);
                Ord::cmp(&anum2, &bnum2)
            } else {
                match other.num.sign() {
                    Sign::Plus => Ordering::Less,
                    _ => Ordering::Greater,
                }
            }
        } else if other.is_finite() {
            match self.num.sign() {
                Sign::Plus => Ordering::Greater,
                _ => Ordering::Less
            }
        } else {
            match (self.num.sign(), other.num.sign()) {
                (Sign::NoSign, Sign::NoSign)
                | (Sign::Minus, Sign::Minus)
                | (Sign::Plus, Sign::Plus)  => Ordering::Equal,
                (Sign::NoSign, _)
                | (Sign::Minus, Sign::Plus) => Ordering::Less,
                (_, Sign::NoSign)
                | (Sign::Plus, Sign::Minus) => Ordering::Greater,
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn ckbr(r: BRat, sx: &str) { assert_eq!(format!("{}", r), sx); }
    fn ckeq(a: BRat) {
        let b = a.clone();
        assert!(&a == &b);
        assert!(!(&a != &b));
        assert!(!(&a < &b));
        assert!(!(&a > &b));
    }
    fn ckne(a: BRat, b: BRat) {
        assert!(&a != &b);
        assert!(&b != &a);
        assert!(!(&a == &b));
        assert!(!(&b == &a));
    }
    fn cklt(a: BRat, b: BRat) {
        assert!(&a < &b);
        assert!(&b > &a);
        assert!(&a <= &b);
        assert!(&b >= &a);
        ckne(a, b);
    }

    // constructors
    #[test] fn brat1() { ckbr(BRat::zero(), "0"); }
    #[test] fn brat2() { ckbr(BRat::one(), "1"); }
    #[test] fn brat3() { ckbr(BRat::nan(), "nan"); }
    #[test] fn brat4() { ckbr(BRat::inf(), "inf"); }
    #[test] fn brat5() { ckbr(BRat::minf(), "-inf"); }
    #[test] fn brat6() { ckbr(BRat::from_char('@'), "64"); }
    #[test] fn brat7() { ckbr(BRat::from_usize(321), "321"); }
    #[test] fn brat8() { ckbr(BRat::from_pair32(-3*4*5,3*2*7), "-10/7"); }
    #[test] fn brat9() { ckbr(BRat::from_i32(-3*4*5), "-60"); }
    #[test] fn brat10() { ckbr(BRat::from_str("+0003e+1").unwrap(), "30"); }
    #[test] fn brat11() { ckbr(BRat::from_str("nan").unwrap(), "nan"); }
    #[test] fn brat12() { ckbr(BRat::from_str("inf").unwrap(), "inf"); }
    #[test] fn brat13() { ckbr(BRat::from_str("-inf").unwrap(), "-inf"); }
    #[test] fn brat14() { ckbr(BRat::from_str("1/2e-1").unwrap(), "5"); }
    #[test] fn brat15() { ckbr(BRat::from_str("1.e001").unwrap(), "10"); }
    #[test] fn brat16() { ckbr(BRat::from_str("2.0").unwrap(), "2"); }
    #[test] fn brat17() { ckbr(BRat::from_str("1e1/2.1e+01").unwrap(), "10/21"); }
    #[test] fn brat18() { ckbr(BRat::ten(), "10"); }
    #[test] fn brat19() { ckbr(BRat::from_str("1.01").unwrap(), "101/100"); }
    #[test] fn brat20() { ckbr(BRat::from_str("1e-1").unwrap(), "1/10"); }
    #[test] fn brat21() { ckbr(BRat::from_str("1./2").unwrap(), "1/2"); }
    #[test] fn brat22() { ckbr(BRat::from_str("1e+0").unwrap(), "1"); }
    #[test] fn brat23() { ckbr(BRat::from_str("1/0023e001").unwrap(), "1/230"); }
    #[test] fn brat24() { ckbr(BRat::from_str("-1/2.0").unwrap(), "-1/2"); }
    #[test] fn brat25() { ckbr(BRat::from_str("2./001.01e1").unwrap(), "20/101"); }
    #[test] fn brat26() { ckbr(BRat::from_str("1/1.01").unwrap(), "100/101"); }


    #[test] fn is_finite1() { assert_eq!(BRat::zero().is_finite(), true); }
    #[test] fn is_finite2() { assert_eq!(BRat::one().is_finite(), true); }
    #[test] fn is_finite3() { assert_eq!(BRat::from_i32(-12).is_finite(), true); }
    #[test] fn is_finite4() { assert_eq!(BRat::nan().is_finite(), false); }
    #[test] fn is_finite5() { assert_eq!(BRat::inf().is_finite(), false); }
    #[test] fn is_finite6() { assert_eq!(BRat::minf().is_finite(), false); }

    #[test] fn round1() { ckbr(BRat::zero().round(), "0"); }
    #[test] fn round2() { ckbr(BRat::one().round(), "1"); }
    #[test] fn round3() { ckbr(BRat::from_i32(-17).round(), "-17"); }
    #[test] fn round4() { ckbr(BRat::from_pair32(-17,10).round(), "-2"); }
    #[test] fn round5() { ckbr(BRat::from_pair32(-16,10).round(), "-2"); }
    #[test] fn round6() { ckbr(BRat::from_pair32(-15,10).round(), "-1"); }
    #[test] fn round7() { ckbr(BRat::from_pair32(-14,10).round(), "-1"); }
    #[test] fn round8() { ckbr(BRat::from_pair32(15,10).round(), "1"); }
    #[test] fn round9() { ckbr(BRat::from_pair32(16,10).round(), "2"); }

    #[test] fn is_int1() { assert_eq!(BRat::zero().is_int(), true); }
    #[test] fn is_int2() { assert_eq!(BRat::one().is_int(), true); }
    #[test] fn is_int3() { assert_eq!(BRat::from_i32(-123).is_int(), true); }
    #[test] fn is_int4() { assert_eq!(BRat::nan().is_int(), false); }
    #[test] fn is_int5() { assert_eq!(BRat::inf().is_int(), false); }
    #[test] fn is_int6() { assert_eq!(BRat::minf().is_int(), false); }
    #[test] fn is_int7() { assert_eq!(BRat::from_pair32(-123,2).is_int(), false); }

    #[test] fn eq1() { ckeq(BRat::zero()); }
    #[test] fn eq2() { ckeq(BRat::one()); }
    #[test] fn eq3() { ckeq(BRat::ten()); }
    #[test] fn eq4() { ckeq(BRat::inf()); }
    #[test] fn eq5() { ckeq(BRat::minf()); }
    #[test] fn eq6() { ckeq(BRat::nan()); }
    #[test] fn eq7() { ckeq(BRat::from_pair32(-123,11)); }

    #[test] fn ne1() { ckne(BRat::zero(), BRat::one()); }
    #[test] fn ne2() { ckne(BRat::zero(), BRat::ten()); }
    #[test] fn ne3() { ckne(BRat::zero(), BRat::inf()); }
    #[test] fn ne4() { ckne(BRat::zero(), BRat::minf()); }
    #[test] fn ne5() { ckne(BRat::zero(), BRat::nan()); }
    #[test] fn ne6() { ckne(BRat::zero(), BRat::from_pair32(-123,11)); }
    #[test] fn ne7() { ckne(BRat::nan(), BRat::inf()); }
    #[test] fn ne8() { ckne(BRat::nan(), BRat::minf()); }

    #[test] fn lt1() { cklt(BRat::nan(), BRat::minf()); }
    #[test] fn lt2() { cklt(BRat::minf(), BRat::from_i32(-123)); }
    #[test] fn lt3() { cklt(BRat::from_i32(-123), BRat::from_pair32(-123, 2)); }
    #[test] fn lt4() { cklt(BRat::from_pair32(-123, 2), BRat::from_i32(-1)); }
    #[test] fn lt5() { cklt(BRat::from_i32(-1), BRat::zero()); }
    #[test] fn lt6() { cklt(BRat::zero(), BRat::one()); }
    #[test] fn lt7() { cklt(BRat::one(), BRat::from_pair32(123, 2)); }
    #[test] fn lt8() { cklt(BRat::from_pair32(123, 2), BRat::inf()); }

    #[test] fn add1() { ckbr(&BRat::zero() + &BRat::zero(), "0"); }
    #[test] fn add2() { ckbr(&BRat::zero() + &BRat::one(), "1"); }
    #[test] fn add3() { ckbr(&BRat::one() + &BRat::one(), "2"); }
    #[test] fn add4() { ckbr(&BRat::zero() + &BRat::nan(), "nan"); }
    #[test] fn add5() { ckbr(&BRat::zero() + &BRat::inf(), "inf"); }
    #[test] fn add6() { ckbr(&BRat::zero() + &BRat::minf(), "-inf"); }
    #[test] fn add7() { ckbr(&BRat::one() + &BRat::nan(), "nan"); }
    #[test] fn add8() { ckbr(&BRat::one() + &BRat::inf(), "inf"); }
    #[test] fn add9() { ckbr(&BRat::one() + &BRat::minf(), "-inf"); }
    #[test] fn add10() { ckbr(&BRat::nan() + &BRat::nan(), "nan"); }
    #[test] fn add11() { ckbr(&BRat::inf() + &BRat::nan(), "nan"); }
    #[test] fn add12() { ckbr(&BRat::minf() + &BRat::nan(), "nan"); }
    #[test] fn add13() { ckbr(&BRat::minf() + &BRat::minf(), "-inf"); }
    #[test] fn add14() { ckbr(&BRat::minf() + &BRat::inf(), "nan"); }
    #[test] fn add15() { ckbr(&BRat::inf() + &BRat::inf(), "inf"); }
    #[test] fn add16() { ckbr(&BRat::inf() + &BRat::minf(), "nan"); }
    #[test]
    fn add17() {
        ckbr(&BRat::from_pair32(123, 4) + &BRat::from_pair32(123, 4), "123/2");
    }
    #[test]
    fn add18() {
        ckbr(&BRat::from_pair32(123, 2) + &BRat::from_pair32(-123, 4), "123/4");
    }
    #[test] fn add19() { ckbr(&BRat::nan() + &BRat::one(), "nan"); }
    #[test] fn add20() { ckbr(&BRat::inf() + &BRat::one(), "inf"); }
    #[test] fn add21() { ckbr(&BRat::minf() + &BRat::one(), "-inf"); }

    #[test] fn neg1() { ckbr(-&BRat::zero(), "0"); }
    #[test] fn neg2() { ckbr(-&BRat::one(), "-1"); }
    #[test] fn neg3() { ckbr(-&-&BRat::one(), "1"); }
    #[test] fn neg4() { ckbr(-&BRat::nan(), "nan"); }
    #[test] fn neg5() { ckbr(-&BRat::inf(), "-inf"); }
    #[test] fn neg6() { ckbr(-&BRat::minf(), "inf"); }
    #[test] fn neg7() { ckbr(-&BRat::from_pair32(123, 4), "-123/4"); }

    #[test] fn sub1() { ckbr(&BRat::zero() - &BRat::zero(), "0"); }
    #[test] fn sub2() { ckbr(&BRat::one() - &BRat::zero(), "1"); }
    #[test] fn sub3() { ckbr(&BRat::zero() - &BRat::one(), "-1"); }
    #[test] fn sub4() { ckbr(&BRat::zero() - &BRat::nan(), "nan"); }
    #[test] fn sub5() { ckbr(&BRat::zero() - &BRat::inf(), "-inf"); }

    #[test] fn mul1() { ckbr(&BRat::zero() * &BRat::zero(), "0"); }
    #[test] fn mul2() { ckbr(&BRat::zero() * &BRat::one(), "0"); }
    #[test] fn mul3() { ckbr(&BRat::one() * &BRat::zero(), "0"); }
    #[test] fn mul4() { ckbr(&BRat::zero() * &BRat::inf(), "nan"); }
    #[test] fn mul5() { ckbr(&BRat::zero() * &BRat::minf(), "nan"); }
    #[test] fn mul6() { ckbr(&BRat::inf() * &BRat::zero(), "nan"); }
    #[test] fn mul7() { ckbr(&BRat::minf() * &BRat::zero(), "nan"); }
    #[test] fn mul8() { ckbr(&BRat::zero() * &BRat::nan(), "nan"); }
    #[test] fn mul9() { ckbr(&BRat::one() * &BRat::one(), "1"); }
    #[test] fn mul10() { ckbr(&BRat::ten() * &BRat::ten(), "100"); }
    #[test]
    fn mul11() {
        ckbr(&BRat::from_pair32(4*9*25, 49*121)
            * &BRat::from_pair32(7*11, 2*3*5), "30/77");
    }

    #[test] fn div1() { ckbr(&BRat::zero() / &BRat::inf(), "0"); }
    #[test] fn div2() { ckbr(&BRat::zero() / &BRat::minf(), "0"); }
    #[test] fn div3() { ckbr(&BRat::zero() / &BRat::one(), "0"); }
    #[test] fn div4() { ckbr(&BRat::zero() / &BRat::ten(), "0"); }
    #[test] fn div5() { ckbr(&BRat::zero() / &BRat::zero(), "nan"); }
    #[test] fn div6() { ckbr(&BRat::zero() / &BRat::nan(), "nan"); }
    #[test] fn div7() { ckbr(&BRat::inf() / &BRat::one(), "inf"); }
    #[test] fn div8() { ckbr(&BRat::inf() / &BRat::ten(), "inf"); }
    #[test] fn div9() { ckbr(&BRat::inf() / &BRat::zero(), "inf"); }
    #[test] fn div10() { ckbr(&BRat::inf() / &BRat::inf(), "nan"); }
    #[test] fn div11() { ckbr(&BRat::inf() / &BRat::minf(), "nan"); }
    #[test] fn div12() { ckbr(&BRat::inf() / &BRat::nan(), "nan"); }
    #[test] fn div13() { ckbr(&BRat::one() / &BRat::one(), "1"); }
    #[test] fn div14() { ckbr(&BRat::one() / &BRat::ten(), "1/10"); }
    #[test] fn div15() { ckbr(&BRat::one() / &BRat::zero(), "inf"); }
    #[test] fn div16() { ckbr(&BRat::ten() / &BRat::one(), "10"); }
    #[test]
    fn div17() {
        ckbr(&BRat::from_pair32(4*9*25, 49*121)
            / &BRat::from_pair32(2*3*5, 7*11), "30/77");
    }
    #[test] fn div18() { ckbr(&BRat::nan() / &BRat::nan(), "nan"); }

    #[test] fn sign1() { assert_eq!(BRat::zero().sign(), Sign::NoSign); }
    #[test] fn sign2() { assert_eq!(BRat::nan().sign(), Sign::NoSign); }
    #[test] fn sign3() { assert_eq!(BRat::one().sign(), Sign::Plus); }
    #[test] fn sign4() { assert_eq!(BRat::inf().sign(), Sign::Plus); }
    #[test] fn sign5() { assert_eq!(BRat::from_pair32(21, 5).sign(), Sign::Plus); }
    #[test] fn sign6() { assert_eq!(BRat::minf().sign(), Sign::Minus); }
    #[test] fn sign7() { assert_eq!(BRat::from_pair32(-21, 5).sign(), Sign::Minus); }

    #[test] fn recip1() { ckbr(BRat::zero().recip(), "inf"); }
    #[test] fn recip2() { ckbr(BRat::nan().recip(), "nan"); }
    #[test] fn recip3() { ckbr(BRat::one().recip(), "1"); }
    #[test] fn recip4() { ckbr(BRat::inf().recip(), "0"); }
    #[test] fn recip5() { ckbr(BRat::minf().recip(), "0"); }
    #[test] fn recip6() { ckbr(BRat::from_i32(-3).recip(), "-1/3"); }
    #[test] fn recip7() { ckbr(BRat::from_pair32(5, 7).recip(), "7/5"); }

    #[test] fn toi1() { assert_eq!(BRat::zero().to_i32(), Some(0i32)); }
    #[test] fn toi2() { assert_eq!(BRat::one().to_i32(), Some(1i32)); }
    #[test] fn toi3() { assert_eq!(BRat::inf().to_i32(), None); }
    #[test] fn toi4() { assert_eq!(BRat::minf().to_i32(), None); }
    #[test] fn toi5() { assert_eq!(BRat::nan().to_i32(), None); }
    #[test] fn toi6() { assert_eq!(BRat::from_i32(-17).to_i32(), Some(-17i32)); }
    #[test] fn toi7() { assert_eq!(BRat::from_pair32(5, 2).to_i32(), None); }

    #[test] fn tou1() { assert_eq!(BRat::zero().to_u32(), Some(0u32)); }
    #[test] fn tou2() { assert_eq!(BRat::one().to_u32(), Some(1u32)); }
    #[test] fn tou3() { assert_eq!(BRat::inf().to_u32(), None); }
    #[test] fn tou4() { assert_eq!(BRat::minf().to_u32(), None); }
    #[test] fn tou5() { assert_eq!(BRat::nan().to_u32(), None); }
    #[test] fn tou6() { assert_eq!(BRat::from_i32(-17).to_u32(), None); }
    #[test] fn tou7() { assert_eq!(BRat::from_i32(17).to_u32(), Some(17u32)); }
    #[test] fn tou8() { assert_eq!(BRat::from_pair32(5, 2).to_u32(), None); }

    #[test] fn tousize1() { assert_eq!(BRat::zero().to_usize(), Some(0usize)); }
    #[test] fn tousize2() { assert_eq!(BRat::one().to_usize(), Some(1usize)); }
    #[test] fn tousize3() { assert_eq!(BRat::inf().to_usize(), None); }
    #[test] fn tousize4() { assert_eq!(BRat::minf().to_usize(), None); }
    #[test] fn tousize5() { assert_eq!(BRat::nan().to_usize(), None); }
    #[test] fn tousize6() { assert_eq!(BRat::from_i32(-17).to_usize(), None); }
    #[test] fn tousize7() { assert_eq!(BRat::from_i32(17).to_usize(), Some(17usize)); }
    #[test] fn tousize8() { assert_eq!(BRat::from_pair32(5, 2).to_usize(), None); }

    #[test] fn tochar1() { assert_eq!(BRat::zero().to_char(), Some('\0')); }
    #[test] fn tochar2() { assert_eq!(BRat::one().to_char(), Some('\u{1}')); }
    #[test] fn tochar3() { assert_eq!(BRat::inf().to_char(), None); }
    #[test] fn tochar4() { assert_eq!(BRat::minf().to_char(), None); }
    #[test] fn tochar5() { assert_eq!(BRat::nan().to_char(), None); }
    #[test] fn tochar6() { assert_eq!(BRat::from_i32(-17).to_char(), None); }
    #[test] fn tochar7() { assert_eq!(BRat::from_i32(65).to_char(), Some('A')); }
    #[test] fn tochar8() { assert_eq!(BRat::from_pair32(5, 2).to_char(), None); }
    #[test] fn tochar9() { assert_eq!(BRat::from_i32(55555).to_char(), None); }
    #[test] fn tochar10() { assert_eq!(BRat::from_i32(1234567).to_char(), None); }
    #[test] fn tochar11() { assert_eq!(BRat::from_i32(123456).to_char(), Some('\u{1E240}')); }

    #[test] fn fstr1() { assert_eq!(BRat::from_str(""), None); }
    #[test] fn fstr2() { assert_eq!(BRat::from_str("e1"), None); }
    #[test] fn fstr3() { assert_eq!(BRat::from_str("+/1"), None); }
    #[test] fn fstr4() { assert_eq!(BRat::from_str("1ee1"), None); }
    #[test] fn fstr5() { assert_eq!(BRat::from_str("1.+e1"), None); }
    #[test] fn fstr6() { assert_eq!(BRat::from_str("1e/2"), None); }
    #[test] fn fstr7() { assert_eq!(BRat::from_str("1+1"), None); }
    #[test] fn fstr8() { assert_eq!(BRat::from_str("1e+/2"), None); }
    #[test] fn fstr9() { assert_eq!(BRat::from_str("1e1+1"), None); }
    #[test] fn fstr10() { assert_eq!(BRat::from_str("1/-1"), None); }
    #[test] fn fstr11() { assert_eq!(BRat::from_str("1/2+1"), None); }
    #[test] fn fstr12() { assert_eq!(BRat::from_str("1/2.+1"), None); }
    #[test] fn fstr13() { assert_eq!(BRat::from_str("1/2.0+1"), None); }
    #[test] fn fstr14() { assert_eq!(BRat::from_str("1/2e/1"), None); }
    #[test] fn fstr15() { assert_eq!(BRat::from_str("1/2e-/1"), None); }
    #[test] fn fstr16() { assert_eq!(BRat::from_str("1/2e1/1"), None); }
    #[test] fn fstr17() { assert_eq!(BRat::from_str("ne"), None); }
    #[test] fn fstr18() { assert_eq!(BRat::from_str("ie"), None); }
    #[test] fn fstr19() { assert_eq!(BRat::from_str("nae"), None); }
    #[test] fn fstr20() { assert_eq!(BRat::from_str("ine"), None); }
    #[test] fn fstr21() { assert_eq!(BRat::from_str("nane"), None); }
    #[test] fn fstr22() { assert_eq!(BRat::from_str("infe"), None); }

    #[test] fn pow1() { assert_eq!(BRat::from_i32(3).pow(&BRat::from_i32(5)),
        Some(BRat::from_i32(243))); }
    #[test] fn pow2() { assert_eq!(BRat::from_i32(3).pow(&BRat::from_i32(-5)),
        Some(BRat::from_pair32(1, 243))); }
    #[test] fn pow3() { assert_eq!(BRat::from_i32(3).pow(&BRat::from_i32(0)),
        Some(BRat::from_i32(1))); }
    #[test] fn pow4() { assert_eq!(BRat::from_i32(3).pow(&BRat::from_pair32(1, 2)),
        None); }
    #[test] fn pow5() { assert_eq!(BRat::nan().pow(&BRat::one()), Some(BRat::nan())); }
    #[test] fn pow6() { assert_eq!(BRat::inf().pow(&BRat::one()), Some(BRat::inf())); }
    #[test] fn pow7() { assert_eq!(BRat::inf().pow(&-&BRat::one()), Some(BRat::zero())); }
    #[test] fn pow8() { assert_eq!(BRat::minf().pow(&BRat::from_i32(3)), Some(BRat::minf())); }
    #[test] fn pow9() { assert_eq!(BRat::minf().pow(&BRat::from_i32(4)), Some(BRat::inf())); }
}

// BigInt, BigUint utilities

use num::{Integer, One, Zero};
use num_bigint::{BigInt, BigUint, Sign};

pub fn bi_from_bu(a: &BigUint) -> BigInt {
    BigInt::from_biguint(Sign::Plus, a.clone())
}

pub fn bi_from_sign(s: Sign) -> BigInt {
    match s {
        Sign::NoSign => BigInt::zero(),
        Sign::Plus => BigInt::one(),
        Sign::Minus => BigInt::from(-1),
    }
}

pub fn bi_sign_abs(rnum: &BigInt) -> (Sign, &BigUint) {
    (rnum.sign(), rnum.magnitude())
}

pub fn bu_gcd_reduce<'a>(anum: &'a BigUint, den: &'a BigUint, sgn: Sign)
        -> (BigUint, BigUint, Sign) {
    let u0: BigUint = BigUint::zero();
    if &u0 == den {
        let (sgn2, anum2) = if &u0 == anum {
            (Sign::Plus, u0.clone())
        } else {
            (sgn, BigUint::one())
        };
        (anum2, u0, sgn2)
    } else {
        let g: BigUint = Integer::gcd(anum, den);
        if BigUint::one() < g {
            let anum2: BigUint = anum / &g;
            let den2: BigUint = den / &g;
            (anum2, den2, sgn)
        } else {
            (anum.clone(), den.clone(), sgn)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num::ToPrimitive;

    fn fsign32(s: Sign) -> i32 { bi_from_sign(s).to_i32().unwrap() }
    fn cksign_abs(i: i32) {
        let ibx = BigInt::from(i);
        let (s, ub) = bi_sign_abs(&ibx);
        assert_eq!(BigInt::from_biguint(s, ub.clone()), ibx);
    }
    fn sign32(i: i32) -> Sign {
        if i < 0 {
            Sign::Minus
        } else if 0 < i {
            Sign::Plus
        } else {
            Sign::NoSign
        }
    }
    fn ckgcd_reduce32(numi: u32, deni: u32, sgni: i32,
        numx: u32, denx: u32, sgnx: i32) {
        let (numa, dena, sgna) =
            bu_gcd_reduce(&BigUint::from(numi), &BigUint::from(deni), sign32(sgni));
        assert_eq!(numa, BigUint::from(numx));
        assert_eq!(dena, BigUint::from(denx));
        assert_eq!(sgna, sign32(sgnx));
    }

    #[test] fn bi_fsign1() { assert_eq!(fsign32(Sign::Plus), 1); }
    #[test] fn bi_fsign2() { assert_eq!(fsign32(Sign::NoSign), 0); }
    #[test] fn bi_fsign3() { assert_eq!(fsign32(Sign::Minus), -1); }
    #[test] fn sign_abs1() { cksign_abs(0); }
    #[test] fn sign_abs2() { cksign_abs(1); }
    #[test] fn sign_abs3() { cksign_abs(-1); }
    #[test] fn sign_abs4() { cksign_abs(5); }
    #[test] fn sign_abs5() { cksign_abs(-5); }
    #[test] fn gcd_reduce1() { ckgcd_reduce32(0, 0, 0,  0, 0, 1); } // nan
    #[test] fn gcd_reduce2() { ckgcd_reduce32(0, 1, 0,  0, 1, 0); } // 0
    #[test] fn gcd_reduce3() { ckgcd_reduce32(1, 0, 1,  1, 0, 1); } // +inf
    #[test] fn gcd_reduce4() { ckgcd_reduce32(1, 0, -1,  1, 0, -1); } // -inf
    #[test] fn gcd_reduce5() { ckgcd_reduce32(1, 1, 1,  1, 1, 1); } // +1
    #[test] fn gcd_reduce6() { ckgcd_reduce32(1, 1, -1,  1, 1, -1); } // -1
    #[test] fn gcd_reduce7() { ckgcd_reduce32(2, 1, 1,  2, 1, 1); } // +2
    #[test] fn gcd_reduce8() { ckgcd_reduce32(1, 2, -1,  1, 2, -1); } // -1/2
    #[test] fn gcd_reduce9() { ckgcd_reduce32(4, 2, 1,  2, 1, 1); } // +4/2 = 2
    #[test] fn gcd_reduce10() { ckgcd_reduce32(64, 48, -1,  4, 3, -1); } // -64/48 = 4/3
}

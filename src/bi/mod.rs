// built-in functions
mod ink;
use self::ink::{inkey, inline, flush_stdout};

use crate::brat::BRat;
use crate::bst::Bst;
use crate::cab::Cab;

extern crate termios;

pub fn register_bi(cab: &mut Cab) -> Result<(), String> {
    cab.add_bi("auto", "as_text(a)", bi_as_text)?;
    cab.add_bi("auto", "catenate(La, Lb)", bi_catenate)?;
    cab.add_bi("auto", "denominator(r)", bi_denominator)?;
    cab.add_bi("auto", "element(L, i)", bi_element)?;
    cab.add_bi("sys", "gbye(a)", bi_gbye)?;
    cab.add_bi("sys", "inkey(echo)", bi_inkey)?;
    cab.add_bi("auto", "inp()", bi_inp)?;
    cab.add_bi("sys", "is_char(r)", bi_is_char)?;
    cab.add_bi("sys", "is_evald(lazy a)", bi_is_evald)?;
    cab.add_bi("sys", "is_function(a)", bi_is_function)?;
    cab.add_bi("sys", "is_list(a)", bi_is_list)?;
    cab.add_bi("sys", "is_mutable(lazy n)", bi_is_mutable)?;
    cab.add_bi("sys", "is_rat(a)", bi_is_rat)?;
    cab.add_bi("sys", "is_var(a)", bi_is_var)?;
    cab.add_bi("auto", "length(L)", bi_length)?;
    cab.add_bi("auto", "numerator(r)", bi_numerator)?;
    cab.add_bi("auto", "op_add(ra, rb)", bi_op_add)?;
    cab.add_bi("auto", "op_div(ra, rb)", bi_op_div)?;
    cab.add_bi("auto", "op_le(a, b)", bi_op_le)?;
    cab.add_bi("auto", "op_mul(ra, rb)", bi_op_mul)?;
    cab.add_bi("auto", "op_neg(r)", bi_op_neg)?;
    cab.add_bi("auto", "op_pow(r, i)", bi_op_pow)?;
    cab.add_bi("auto", "out(a)", bi_out)?;
    cab.add_bi("sys", "parse(t)", bi_parse)?;
    cab.add_bi("sys", "rat(t)", bi_rat)?;
    cab.add_bi("sys", "rat_version()", bi_rat_version)?;
    cab.add_bi("auto", "reverse(L)", bi_reverse)?;
    cab.add_bi("auto", "round(r)", bi_round)?;
    cab.add_bi("auto", "sublist(L, ix, n)", bi_sublist)?;
    cab.add_bi("sys", "tree_text(lazy a)", bi_tree_text)?;
    cab.add_bi("sys", "var_name(lazy v)", bi_var_name)?;
    cab.add_bi("sys", "variable(t)", bi_variable)?;
    Ok(())
}

// is_text(as_text(a)) # eval(a) == parse(as_text(a))
fn bi_as_text(cab: &mut Cab, idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    bi_tree_text(cab, idx, args)
}

fn bi_catenate(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    let mut a = args[0].d_list("catenate arg 1")?.clone();
    let mut b = args[1].d_list("catenate arg 2")?.clone();
    a.append(&mut b);
    Ok(Bst::List(a))
}

fn bi_denominator(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    Ok(Bst::Rat(args[0].d_rat("denominator")?.denominator()))
}

fn bi_element(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    let va = args[0].d_list("element")?;
    let ui = args[1].d_rat_pos_usize("element")?;
    let na = va.len();
    if ui <= na {
        Ok(va[ui-1].clone())
    } else {
        Err(format!("element: List length {} < index {}", na, ui))
    }
}

fn bi_gbye(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    Err(args[0].d_list_text("gbye")?)
}

#[cfg(not(tarpaulin_include))] // input
fn bi_inkey(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    let echo = args[0].d_rat_bool("inkey")?;
    Ok(Bst::Rat(BRat::from_char(inkey(echo)?)))
}

#[cfg(not(tarpaulin_include))] // input
fn bi_inp(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, _args: Vec<Bst>) -> Result<Bst, String> {
    Ok(string_to_text(&inline()?))
}

fn bi_is_char(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    Ok(Bst::from_bool(match &args[0] {
        Bst::Rat(r) => r.to_char().is_some(),
        _ => false,
    }))
}

fn bi_is_evald(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    Ok(Bst::from_bool(is_evald(&args[0])))
}

fn bi_is_function(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    Ok(Bst::from_bool(matches!(&args[0], Bst::Func(..))))
}

fn bi_is_list(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    Ok(Bst::from_bool(matches!(&args[0], Bst::List(_))))
}

fn bi_is_mutable(cab: &mut Cab, idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    let name = args[0].d_closure().d_name("is_mutable")?;
    Ok(Bst::from_bool(!cab.is_const(&name, idx)?))
}

fn bi_is_rat(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    Ok(Bst::from_bool(matches!(&args[0], Bst::Rat(_))))
}

// check if var exists
fn bi_is_var(cab: &mut Cab, idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    let name = args[0].d_name("is_var")?;
    Ok(Bst::from_bool(cab.athas(&name, idx)?))
}

fn bi_length(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    let v = args[0].d_list("length")?;
    Ok(Bst::Rat(BRat::from_usize(v.len())))
}

fn bi_numerator(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    Ok(Bst::Rat(args[0].d_rat("numerator")?.numerator()))
}

fn bi_op_add(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    match (&args[0], &args[1]) {
        (Bst::Rat(a), Bst::Rat(b)) => Ok(Bst::Rat(a + b)),
        (Bst::List(v), e) => { // v+e is push // '' + a + b + c => list[a, b, c]
            let mut w = v.clone();
            w.push(e.clone());
            Ok(Bst::List(w))
        },
        _ => Err(format!("op_add: not (Rat, Rat) or (List, Any): ({}, {})",
                args[0].variant(), args[1].variant())),
    }
}

fn bi_op_div(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    let a = args[0].d_rat("op_div arg 1")?;
    let b = args[1].d_rat("op_div arg 2")?;
    Ok(Bst::Rat(a / b))
}

fn bi_op_le(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    Ok(Bst::from_bool(op_le(&args[0], &args[1])?))
}

fn bi_op_mul(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    // NB: list repeat is not useful enough for an overload
    let a = args[0].d_rat("op_mul arg 1")?;
    let b = args[1].d_rat("op_mul arg 2")?;
    Ok(Bst::Rat(a * b))
}

fn bi_op_neg(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    let a = args[0].d_rat("op_neg")?;
    Ok(Bst::Rat(-a))
}

fn bi_op_pow(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    let a = args[0].d_rat("op_pow arg 1")?;
    let b = args[1].d_rat_int("op_pow arg 2")?;
    Ok(Bst::Rat(a.pow(b).unwrap()))
}

#[cfg(not(tarpaulin_include))] // output
fn bi_out(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    let s = args[0].d_list_text("out")?;
    print!("{}", s);
    flush_stdout();
    Ok(Bst::zero())
}

// Text -> Bst
fn bi_parse(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    let s = args[0].d_list_text("parse")?;
    Bst::from_str_stmt(&s, "(parse)")
}

// Text -> Rat
fn bi_rat(cab: &mut Cab, idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    let p = bi_parse(cab, idx, args)?;
    match p {
        Bst::Rat(_) => Ok(p),
        _ => Err(format!("rat: does not parse to a Rat: {}", p.variant())),
    }
}

fn bi_rat_version(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, _args: Vec<Bst>) -> Result<Bst, String> {
    Ok(string_to_text(&format!("rat v{}", env!("CARGO_PKG_VERSION"))))
}

fn bi_reverse(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    let mut b = args[0].d_list("reverse")?.clone();
    b.reverse();
    Ok(Bst::List(b))
}

fn bi_round(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    Ok(Bst::Rat(args[0].d_rat("round")?.round()))
}

fn bi_sublist(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    let av = args[0].d_list("sublist arg 1")?;
    let ix = args[1].d_rat_pos_usize("sublist arg 2")? - 1;
    let ex = args[2].d_rat_usize("sublist arg 3")? + ix;
    let alen = av.len();
    if ex <= alen {
        Ok(Bst::List((&av[ix..ex]).to_vec()))
    } else {
        Err(format!("sublist: List length {} less than ending index {}", alen, ex))
    }
}

// Bst -> Text
fn bi_tree_text(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    Ok(string_to_text(&format!("{}", &args[0])))
}

// Bst::Name -> Text // Q: Is this always the same as tree_text when it doesn't fail?
fn bi_var_name(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    let s = args[0].d_name("var_name")?;
    Ok(string_to_text(&s))
}

// Text -> Bst::Name
fn bi_variable(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, args: Vec<Bst>) -> Result<Bst, String> {
    let s = args[0].d_list_text("variable")?;
    // TODO: check valid name
    Ok(Bst::Name(s))
}

fn op_le(x: &Bst, y: &Bst) -> Result<bool, String> {
    Ok(match (x, y) {
        (Bst::Rat(a), Bst::Rat(b)) => a <= b,
        (Bst::List(a), Bst::List(b)) => list_le(a, b)?,
        (Bst::Func(apath, afunc), Bst::Func(bpath, bfunc)) =>
            apath < bpath || (apath == bpath && afunc <= bfunc),
        // Func < Rat < List
        (Bst::Func(..), Bst::Rat(_)|Bst::List(_)) => true,
        (Bst::Rat(_)|Bst::List(_),Bst::Func(..)) => false,
        (Bst::Rat(_), Bst::List(_)) => true,
        (Bst::List(_), Bst::Rat(_)) => false,
        _ => return Err(format!("op_le: not comparable: ({}, {})",
                        x.variant(), y.variant())),
    })
}

fn list_le(a: &[Bst], b: &[Bst]) -> Result<bool, String> {
    if a.is_empty() { return Ok(true); }
    if b.is_empty() { return Ok(false); }
    let c = &a[0];
    let d = &b[0];
    let le = op_le(c, d)?;
    if le && op_le(d, c)? {
        return list_le(&a[1..], &b[1..]);
    }
    Ok(le)
}

fn is_evald(a: &Bst) -> bool {
    match a {
        Bst::Break
        | Bst::Func(..)
        | Bst::Rat(_)
            => true,
        Bst::Call(..)
        | Bst::Closure(..)
        | Bst::Each(..)
        | Bst::Let(..)
        | Bst::Loop(..)
        | Bst::Name(_) // eval is lookup
        | Bst::When(..)
            => false,
        Bst::List(es) => {
            for e in es {
                if !is_evald(e) { return false; }
            }
            true
        },
        Bst::Return(xb) => is_evald(xb),
    }
}

fn string_to_text(s: &str) -> Bst {
    Bst::List(
        s.chars().map(
            |c| Bst::Rat(BRat::from_char(c))
        ).collect()
    )
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::cab::BiType;
    use std::collections::HashMap;

    fn taxi() -> Cab {
        let mut cab = Cab::new();
        cab.push(HashMap::new());
        cab
    }
    fn hail(cab: &mut Cab, bif: BiType, targs: Vec<&str>) -> Result<Bst, String> {
        let mut bargs = vec![];
        for targ in targs.iter() {
            bargs.push(Bst::from_str_stmt(targ, "_test_").unwrap());
        }
        bif(cab, &None, bargs)
    }
    fn call(bif: BiType, targs: Vec<&str>) -> Result<Bst, String> {
        hail(&mut Cab::new(), bif, targs)
    }
    fn cnc(bif: BiType, targs: Vec<&str>, xres: &str) {
        let btext = format!("{}", call(bif, targs).unwrap());
        assert_eq!(btext, xres);
    }
    fn xnx(bif: BiType, targs: Vec<&str>, xerr: &str) {
        let e = call(bif, targs).unwrap_err();
        if 0 < xerr.len() { assert_eq!(e, xerr); }
    }

    #[test] fn astxt1() { cnc(bi_as_text, vec!["0"], "list[48]"); }
    #[test] fn cat1() { cnc(bi_catenate, vec!["list[]", "list[]"], "list[]"); }
    #[test] fn cat2() { cnc(bi_catenate, vec!["list[3]", "list[]"], "list[3]"); }
    #[test] fn cat3() { cnc(bi_catenate, vec!["list[3, 5]", "list[]"], "list[3, 5]"); }
    #[test] fn cat4() { cnc(bi_catenate, vec!["list[]", "list[2]"], "list[2]"); }
    #[test] fn cat5() { cnc(bi_catenate, vec!["list[]", "list[2, 4]"], "list[2, 4]"); }
    #[test] fn cat6() { cnc(bi_catenate, vec!["list[3]", "list[2]"], "list[3, 2]"); }
    #[test] fn cat7() { cnc(bi_catenate, vec!["list[3, 5]", "list[2]"], "list[3, 5, 2]"); }
    #[test] fn cat8() { cnc(bi_catenate, vec!["list[3]", "list[2, 4]"], "list[3, 2, 4]"); }
    #[test] fn cat9() { cnc(bi_catenate, vec!["list[3, 5]", "list[2, 4]"], "list[3, 5, 2, 4]"); }
    #[test] fn den1() { cnc(bi_denominator, vec!["4"], "1"); }
    #[test] fn den2() { cnc(bi_denominator, vec!["0"], "1"); }
    #[test] fn den3() { cnc(bi_denominator, vec!["inf"], "0"); }
    #[test] fn den4() { cnc(bi_denominator, vec!["nan"], "0"); }
    #[test] fn den5() { cnc(bi_denominator, vec!["4/2"], "1"); }
    #[test] fn den6() { cnc(bi_denominator, vec!["3/2"], "2"); }
    #[test] fn den7() { cnc(bi_denominator, vec!["-3/2"], "2"); }
    #[test] fn den8() { cnc(bi_denominator, vec!["1/3"], "3"); }
    #[test] fn elt1() { cnc(bi_element, vec!["list[4]", "1"], "4"); }
    #[test] fn elt2() { cnc(bi_element, vec!["list[4, 5, 6]", "1"], "4"); }
    #[test] fn elt3() { cnc(bi_element, vec!["list[4, 5, 6]", "2"], "5"); }
    #[test] fn elt4() { cnc(bi_element, vec!["list[4, 5, 6]", "3"], "6"); }
    #[test] fn elt5() { xnx(bi_element, vec!["list[1, 2, 3]", "4"], "element: List length 3 < index 4"); }
    #[test] fn gby1() { xnx(bi_gbye, vec!["'ylo brk rd'"], "ylo brk rd"); }
    #[test] fn isch1() { cnc(bi_is_char, vec!["65"], "1"); }
    #[test] fn isch2() { cnc(bi_is_char, vec!["65/2"], "0"); }
    #[test] fn isch3() { cnc(bi_is_char, vec!["list[65]"], "0"); }
    #[test] fn isch4() { cnc(bi_is_char, vec!["1e11"], "0"); }
    #[test] fn isevald1() { cnc(bi_is_evald, vec!["0"], "1"); }
    #[test] fn isevald2() { cnc(bi_is_evald, vec!["-inf"], "1"); }
    #[test] fn isevald3() { cnc(bi_is_evald, vec!["''"], "1"); }
    #[test] fn isevald4() { cnc(bi_is_evald, vec!["'abc'"], "1"); }
    #[test] fn isevald5() { cnc(bi_is_evald, vec!["list[1, 2, list[3, 4, 'abc'], 'def']"], "1"); }
    #[test] fn isevald6() { cnc(bi_is_evald, vec!["foo(1)"], "0"); }
    #[test] fn isevald7() { cnc(bi_is_evald, vec!["a"], "0"); }
    #[test] fn isevald8() { cnc(bi_is_evald, vec!["1+ 2"], "0"); }
    #[test] fn isevald9() { cnc(bi_is_evald, vec!["return 3"], "1"); }
    #[test] fn isevald10() { cnc(bi_is_evald, vec!["return x"], "0"); }
    #[test] fn isfn1() {
        let func = Bst::Func("hi".to_string(), "there".to_string());
        let res = bi_is_function(&mut Cab::new(), &None, vec![func]).unwrap();
        assert_eq!(res, Bst::one());
    }
    #[test] fn isfn2() {
        let it = Bst::one();
        let res = bi_is_function(&mut Cab::new(), &None, vec![it]).unwrap();
        assert_eq!(res, Bst::zero());
    }
    #[test] fn islst1() { cnc(bi_is_list, vec!["list[]"], "1"); }
    #[test] fn islst2() { cnc(bi_is_list, vec!["list[1]"], "1"); }
    #[test] fn islst3() { cnc(bi_is_list, vec!["list[list[1]]"], "1"); }
    #[test] fn islst4() { cnc(bi_is_list, vec!["1"], "0"); }
    #[test] fn islst5() { cnc(bi_is_list, vec!["'abc'"], "1"); }
    #[test] fn israt1() { cnc(bi_is_rat, vec!["0"], "1"); }
    #[test] fn israt2() { cnc(bi_is_rat, vec!["1"], "1"); }
    #[test] fn israt3() { cnc(bi_is_rat, vec!["-1"], "1"); }
    #[test] fn israt4() { cnc(bi_is_rat, vec!["1/2"], "1"); }
    #[test] fn israt5() { cnc(bi_is_rat, vec!["-5/3"], "1"); }
    #[test] fn israt6() { cnc(bi_is_rat, vec!["nan"], "1"); }
    #[test] fn israt7() { cnc(bi_is_rat, vec!["inf"], "1"); }
    #[test] fn israt8() { cnc(bi_is_rat, vec!["-inf"], "1"); }
    #[test] fn israt9() { cnc(bi_is_rat, vec!["list[]"], "0"); }
    #[test] fn len1() { cnc(bi_length, vec!["list[]"], "0"); }
    #[test] fn len2() { cnc(bi_length, vec!["list[5]"], "1"); }
    #[test] fn len3() { cnc(bi_length, vec!["list[list[]]"], "1"); }
    #[test] fn len4() { cnc(bi_length, vec!["list[5, list[]]"], "2"); }
    #[test] fn len5() { cnc(bi_length, vec!["list[11, 12, 13, 14, 15]"], "5"); }
    #[test] fn ismut1() {
        let mut cab = taxi();
        cab.atput("foo", &None, true, true, Bst::one()).unwrap();
        let res = hail(&mut cab, bi_is_mutable, vec!["foo"]).unwrap();
        assert_eq!(res, Bst::zero());
    }
    #[test] fn ismut2() {
        let mut cab = taxi();
        cab.atput("bar", &None, true, false, Bst::zero()).unwrap();
        let res = hail(&mut cab, bi_is_mutable, vec!["bar"]).unwrap();
        assert_eq!(res, Bst::one());
    }
    #[test] fn isvar1() {
        let mut cab = taxi();
        cab.atput("foo", &None, true, true, Bst::zero()).unwrap();
        let res = hail(&mut cab, bi_is_var, vec!["foo"]).unwrap();
        assert_eq!(res, Bst::one());
    }
    #[test] fn isvar2() {
        let res = hail(&mut taxi(), bi_is_var, vec!["foo"]).unwrap();
        assert_eq!(res, Bst::zero());
    }
    #[test] fn num1() { cnc(bi_numerator, vec!["0"], "0"); }
    #[test] fn num2() { cnc(bi_numerator, vec!["nan"], "0"); }
    #[test] fn num3() { cnc(bi_numerator, vec!["1"], "1"); }
    #[test] fn num4() { cnc(bi_numerator, vec!["1/2"], "1"); }
    #[test] fn num5() { cnc(bi_numerator, vec!["1/5"], "1"); }
    #[test] fn num6() { cnc(bi_numerator, vec!["inf"], "1"); }
    #[test] fn num7() { cnc(bi_numerator, vec!["-1"], "-1"); }
    #[test] fn num8() { cnc(bi_numerator, vec!["-1/2"], "-1"); }
    #[test] fn num9() { cnc(bi_numerator, vec!["-inf"], "-1"); }
    #[test] fn oadd1() { cnc(bi_op_add, vec!["0", "0"], "0"); }
    #[test] fn oadd2() { cnc(bi_op_add, vec!["0", "1"], "1"); }
    #[test] fn oadd3() { cnc(bi_op_add, vec!["1", "0"], "1"); }
    #[test] fn oadd4() { cnc(bi_op_add, vec!["-1", "1"], "0"); }
    #[test] fn oadd5() { cnc(bi_op_add, vec!["list[1, 2]", "3"], "list[1, 2, 3]"); }
    #[test] fn oadd6() { xnx(bi_op_add, vec!["1", "list[]"],
        "op_add: not (Rat, Rat) or (List, Any): (Rat, List)"); }
    #[test] fn odiv1() { cnc(bi_op_div, vec!["6", "3"], "2"); }
    #[test] fn odiv2() { cnc(bi_op_div, vec!["-6", "-3"], "2"); }
    #[test] fn odiv3() { cnc(bi_op_div, vec!["24", "9"], "8/3"); }
    #[test] fn ole1() { cnc(bi_op_le, vec!["0", "0"], "1"); }
    #[test] fn ole2() { cnc(bi_op_le, vec!["0", "1"], "1"); }
    #[test] fn ole3() { cnc(bi_op_le, vec!["1", "0"], "0"); }
    #[test] fn ole4() { cnc(bi_op_le, vec!["inf", "0"], "0"); }
    #[test] fn ole5() { cnc(bi_op_le, vec!["list[]", "list[0]"], "1"); }
    #[test] fn ole6() { cnc(bi_op_le, vec!["list[1, 2]", "list[1, 1]"], "0"); }
    #[test] fn ole7() { cnc(bi_op_le, vec!["99", "list[]"], "1"); }
    #[test] fn ole8() { cnc(bi_op_le, vec!["list[3]", "3"], "0"); }
    #[test] fn ole9() { xnx(bi_op_le, vec!["3", "break"],
        "op_le: not comparable: (Rat, Break)"); }
    #[test] fn ole10() {
        let fab = Bst::Func("a".to_string(), "b".to_string());
        let fba = Bst::Func("b".to_string(), "a".to_string());
        assert!(op_le(&fab, &fba).unwrap());
    }
    #[test] fn ole11() {
        let fab = Bst::Func("a".to_string(), "b".to_string());
        assert!(op_le(&fab, &Bst::List(vec![])).unwrap());
    }
    #[test] fn ole12() {
        let fab = Bst::Func("a".to_string(), "b".to_string());
        assert!(!op_le(&Bst::List(vec![]), &fab).unwrap());
    }
    // TODO: le on other types
    #[test] fn omul1() { cnc(bi_op_mul, vec!["3", "5"], "15"); }
    #[test] fn oneg1() { cnc(bi_op_neg, vec!["0"], "0"); }
    #[test] fn oneg2() { cnc(bi_op_neg, vec!["nan"], "nan"); }
    #[test] fn oneg3() { cnc(bi_op_neg, vec!["inf"], "-inf"); }
    #[test] fn oneg4() { cnc(bi_op_neg, vec!["-inf"], "inf"); }
    #[test] fn oneg5() { cnc(bi_op_neg, vec!["12"], "-12"); }
    #[test] fn oneg6() { cnc(bi_op_neg, vec!["-123"], "123"); }
    #[test] fn opow1() { cnc(bi_op_pow, vec!["2", "3"], "8"); }
    #[test] fn parse1() { cnc(bi_parse, vec!["'1/2'"], "1/2"); }
    #[test] fn parse2() { cnc(bi_parse, vec!["'1/2 + 2/3'"], "op_add(1/2, 2/3)"); }
    #[test] fn rat1() { cnc(bi_rat, vec!["'1/2'"], "1/2"); }
    #[test] fn rat2() { xnx(bi_rat, vec!["'list[]'"],
            "rat: does not parse to a Rat: List"); }
    #[test] fn rver1() {
        let b = call(bi_rat_version, vec![]).unwrap();
        let s = b.d_list_text("test rver1").unwrap();
        assert!(s.starts_with("rat v"));
    }
    #[test] fn rev1() { cnc(bi_reverse, vec!["list[]"], "list[]"); }
    #[test] fn rev2() { cnc(bi_reverse, vec!["list[2/5]"], "list[2/5]"); }
    #[test] fn rev3() { cnc(bi_reverse, vec!["list[2, 5]"], "list[5, 2]"); }
    #[test] fn rev4() { cnc(bi_reverse, vec!["'ABC'"], "list[67, 66, 65]"); }
    #[test] fn round1() { cnc(bi_round, vec!["3"], "3"); }
    #[test] fn round2() { cnc(bi_round, vec!["-3"], "-3"); }
    #[test] fn round3() { cnc(bi_round, vec!["3.4"], "3"); }
    #[test] fn round4() { cnc(bi_round, vec!["-3.4"], "-3"); }
    #[test] fn round5() { cnc(bi_round, vec!["3.6"], "4"); }
    #[test] fn round6() { cnc(bi_round, vec!["-3.6"], "-4"); }
    #[test] fn sublist1() { cnc(bi_sublist, vec!["list[11, 12, 13, 14]", "1", "0"], "list[]"); }
    #[test] fn sublist2() { cnc(bi_sublist, vec!["list[11, 12, 13, 14]", "4", "0"], "list[]"); }
    #[test] fn sublist3() { cnc(bi_sublist, vec!["list[11, 12, 13, 14]", "1", "1"], "list[11]"); }
    #[test] fn sublist4() { cnc(bi_sublist, vec!["list[11, 12, 13, 14]", "4", "1"], "list[14]"); }
    #[test] fn sublist5() { cnc(bi_sublist, vec!["list[11, 12, 13, 14]", "1", "2"], "list[11, 12]"); }
    #[test] fn sublist6() { cnc(bi_sublist, vec!["list[11, 12, 13, 14]", "2", "2"], "list[12, 13]"); }
    #[test] fn sublist7() { cnc(bi_sublist, vec!["list[11, 12, 13, 14]", "1", "3"], "list[11, 12, 13]"); }
    #[test] fn sublist8() { cnc(bi_sublist, vec!["list[11, 12, 13, 14]", "2", "3"], "list[12, 13, 14]"); }
    #[test] fn sublist9() { xnx(bi_sublist, vec!["list[1, 2, 3, 4, 5]", "3", "4"],
            "sublist: List length 5 less than ending index 6"); }
    // TODO: tree_text
    #[test] fn vname1() { cnc(bi_var_name, vec!["a"], "list[97]"); }
    #[test] fn var1() { cnc(bi_variable, vec!["'a_2'"], "a_2"); }
    #[test] fn regbi1() {
        let mut cab = taxi();
        register_bi(&mut cab).unwrap();
        cab.netget("auto", "denominator").unwrap();
        cab.netget("sys", "is_char").unwrap();
    }
}

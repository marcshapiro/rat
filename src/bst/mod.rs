// cleaned Ast for Rat
use std::fmt;

use crate::ast::Ast;
use crate::brat::BRat;
use crate::parse::{parse_str_to_ast_stmt, parse_str_to_ast_file};
use crate::udisp::write_vec;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Bst {
    Break,
    Call(Box<Bst>, Vec<Bst>), // func, args
    Closure(Box<Bst>, usize, BRat),
    Each(String, Box<Bst>, Vec<Bst>), // var, list, stmts
    Func(String, String), // path/func
    Let(bool, bool, bool, String, Box<Bst>), // create?, const?, strict?, var, val
    List(Vec<Bst>),
    Loop(Vec<Bst>),
    Name(String),
    Rat(BRat),
    Return(Box<Bst>),
    When(Vec<(Bst, Vec<Bst>)>, Vec<Bst>), // (cond, body), else
}

#[derive(Clone, Debug)]
pub struct FuncDecl {
    pub oname: Option<String>,
    pub named: Vec<(String, bool)>,
    pub has_dots: bool,
    pub strict_dots: bool,
}

#[derive(Debug)]
pub struct BstFile {
    pub uses: Vec<(String, String, String)>,
    pub decl: Option<FuncDecl>, // Option<(Option<String>, Vec<(String, bool)>, bool, bool)>,
    pub file: Vec<Bst>,
}

impl Bst {
    fn from_ast_stmt(a: &Ast) -> Result<Bst, String> {
        Ok(match a {
            Ast::File(_) | Ast::FnDecl(..)
            | Ast::Use(_, _, _) => return Err("from_ast: filtered branch".to_owned()),
            Ast::Break => Bst::Break,
            Ast::Call(xafunc, aargs) => Bst::Call(box_to_bst(xafunc)?, vec_to_bst(aargs)?),
            Ast::Each(v, ee, ss) => Bst::Each(v.clone(), box_to_bst(ee)?, vec_to_bst(ss)?),
            Ast::Let(create, konst, strict, var, xaval) =>
                Bst::Let(*create, *konst, *strict, var.to_owned(), box_to_bst(xaval)?),
            Ast::List(va) => Bst::List(vec_to_bst(va)?),
            Ast::Loop(va) => Bst::Loop(vec_to_bst(va)?),
            Ast::Name(s) => Bst::Name(s.to_owned()),
            Ast::RatLit(r) => Bst::Rat(r.clone()),
            Ast::Return(oxa) => match oxa {
                None => Bst::Return(Box::new(Bst::zero())),
                Some(xa) => Bst::Return(box_to_bst(xa)?),
            },
            Ast::When(vap, vae) => {
                let mut vbp = vec![];
                for (ac, vad) in vap {
                    vbp.push((Bst::from_ast_stmt(ac)?, vec_to_bst(vad)?));
                }
                Bst::When(vbp, vec_to_bst(vae)?)
            },
        })
    }
    fn from_ast_file(aa: &Ast) -> Result<BstFile, String> {
        let mut uses = vec![];
        let mut decl = None;
        let mut file = vec![];
        match aa {
            Ast::File(va) => {
                for a in va {
                    match a {
                        Ast::FnDecl(ofunc, vab, dots, strict_dots) => {
                            match decl {
                                Some(_) => return Err("from_ast_file: duplicate 'function'".to_owned()),
                                None => {
                                    decl = Some(FuncDecl { oname: ofunc.clone(),
                                        named: vab.clone(), has_dots: *dots, strict_dots: *strict_dots });
                                },
                            }
                        },
                        Ast::Use(path, func, name) => {
                            uses.push((path.to_owned(), func.to_owned(), name.to_owned()));
                        },
                        _ => { file.push( Bst::from_ast_stmt(a)?); },
                    }
                }
            },
            _ => return Err("from_ast_file: not a File".to_owned()),
        }
        Ok(BstFile{uses, decl, file})
    }
    pub fn from_str_stmt(s: &str, ctx: &str) -> Result<Bst, String> {
        let a = parse_str_to_ast_stmt(s, ctx)?;
        Bst::from_ast_stmt(&a)
    }
    pub fn from_str_file(s: &str, ctx: &str) -> Result<BstFile, String> {
        let a = parse_str_to_ast_file(s, ctx)?;
        Bst::from_ast_file(&a)
    }
    pub fn zero() -> Bst { Bst::Rat(BRat::zero()) }
    pub fn one() -> Bst { Bst::Rat(BRat::one()) }
    pub fn from_bool(b: bool) -> Bst { if b { Bst::one() } else { Bst::zero() } }
    pub fn variant(&self) -> &'static str {
        match self {
            Bst::Break => "Break",
            Bst::Call(..) => "Call",
            Bst::Closure(..) => "Closure",
            Bst::Each(..) => "Each",
            Bst::Func(..) => "Func",
            Bst::Let(..) => "Let",
            Bst::List(..) => "List",
            Bst::Loop(..) => "Loop",
            Bst::Name(..) => "Name",
            Bst::Rat(..) => "Rat",
            Bst::Return(..) => "Return",
            Bst::When(..) => "When",
        }
    }
    pub fn d_closure(&self) -> Bst {
        match self {
            Bst::Closure(x, _, _) => (**x).clone(),
            _ => self.clone()
        }
    }
    pub fn d_list<'a>(&'a self, msg: &'a str) -> Result<&'a Vec<Bst>, String> {
        match self {
            Bst::List(v) => Ok(v),
            _ => Err(format!("{}: not a List: {}", msg, self.variant())),
        }
    }
    pub fn d_list_text(&self, msg: &str) -> Result<String, String> {
        let es = self.d_list(msg)?;
        let mut cs = vec![];
        for (i, e) in es.iter().enumerate() {
            match e {
                Bst::Rat(r) => match r.to_char() {
                    None => return Err(format!("{} elt {}: not Char: {}", msg, i+1, r)),
                    Some(c) => cs.push(c),
                },
                _ => return Err(format!("{} elt {}: not a Rat: {}", msg, i+1, e.variant())),
            }
        }
        Ok(cs.iter().collect())
    }
    pub fn d_rat<'a>(&'a self, msg: &'a str) -> Result<&'a BRat, String> {
        match self {
            Bst::Rat(r) => Ok(r),
            _ => Err(format!("{}: not a Rat: {}", msg, self.variant())),
        }
    }
    pub fn d_rat_int<'a>(&'a self, msg: &'a str) -> Result<&'a BRat, String> {
        let r = self.d_rat(msg)?;
        if r.is_int() {
            Ok(r)
        } else {
            Err(format!("{}: not an Int: {}", msg, r))
        }
    }
    pub fn d_rat_nonneg_int<'a>(&'a self, msg: &'a str) -> Result<&'a BRat, String> {
        let r = self.d_rat_int(msg)?;
        if &BRat::zero() <= r {
            Ok(r)
        } else {
            Err(format!("{}: not a Nonnegative Int: {}", msg, r))
        }
    }
    pub fn d_rat_pos_int<'a>(&'a self, msg: &'a str) -> Result<&'a BRat, String> {
        let r = self.d_rat_int(msg)?;
        if &BRat::zero() < r {
            Ok(r)
        } else {
            Err(format!("{}: not a Positive Int: {}", msg, r))
        }
    }
    pub fn d_rat_usize(&self, msg: &str) -> Result<usize, String> {
        let r = self.d_rat_nonneg_int(msg)?;
        match r.to_usize() {
            None => Err(format!("{}: too large for usize: {}", msg, r)),
            Some(u) => Ok(u),
        }
    }
    pub fn d_rat_pos_usize(&self, msg: &str) -> Result<usize, String> {
        let r = self.d_rat_pos_int(msg)?;
        match r.to_usize() {
            None => Err(format!("{}: too large for usize: {}", msg, r)),
            Some(u) => Ok(u),
        }
    }
    pub fn d_rat_bool(&self, msg: &str) -> Result<bool, String> {
        let r = self.d_rat(msg)?;
        if r == &BRat::zero() {
            Ok(false)
        } else if r == &BRat::one() {
            Ok(true)
        } else {
            Err(format!("{}: not a Bool: {}", msg, r))
        }
    }
    pub fn d_name(&self, msg: &str) -> Result<String, String> {
        match self {
            Bst::Name(n) => Ok(n.clone()),
            _ => Err(format!("{}: not a Name: {}", msg, self.variant())),
        }
    }
}

fn vec_to_bst(va: &[Ast]) -> Result<Vec<Bst>, String> {
    let mut vb = vec![];
    for a in va.iter() {
        vb.push(Bst::from_ast_stmt(a)?);
    }
    Ok(vb)
}

fn box_to_bst(xa: &Ast) -> Result<Box<Bst>, String> {
    Ok(Box::new(Bst::from_ast_stmt(xa)?))
}

impl fmt::Display for Bst {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Bst::Break => write!(f, "break"),
            Bst::Call(name, args) => write_vec(f, &format!("{}(", name), args, ", ", ")"),
            Bst::Closure(xval, ix, id) => write!(f, "_closure({}, {}, {})", xval, ix, id),
            Bst::Each(name, list, stmts) => {
                write!(f, "each {} in {} ", name, list)?;
                write_vec(f, "{\n    ", stmts, ";\n    ", "\n}\n")
            },
            Bst::Func(path, func) => write!(f, "_fn({}/{})", path, func),
            Bst::Let(is_create, is_const, is_strict, name, value) => {
                let which = match (is_create, is_const) {
                    (true, true) => "let",
                    (true, false) => "mutable",
                    (false, false) => "update",
                    (false, true) => "_looplet",
                };
                let how = if *is_strict {""} else { "lazy "};
                write!(f, "{} {} = {}{}", which, name, how, value)
            },
            Bst::List(list) => write_vec(f, "list[", list, ", ", "]"),
            Bst::Loop(seq) => write_vec(f, "loop {\n    ", seq, ";\n    ", "\n}\n"),
            Bst::Name(n) => write!(f, "{}", n),
            Bst::Rat(r) => write!(f, "{}", r),
            Bst::Return(r) => write!(f, "return {}", r),
            Bst::When(pairs, elze) => {
                writeln!(f, "when {{")?;
                for (cond, thyn) in pairs {
                    write!(f, "  {} => ", cond)?;
                    write_vec(f, "{\n    ", thyn, ";\n    ", "\n  }\n")?;
                }
                write_vec(f, "}  else {\n    ", elze, ";\n    ", "\n  }\n")
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{parse_str_to_ast_stmt};
    fn pa(s: &str) -> Result<Ast, String> { parse_str_to_ast_stmt(s, "_test_") }
    fn pb(s: &str) -> Result<Bst, String> { Bst::from_str_stmt(s, "_test_") }
    fn bsf(s: &str) -> Result<BstFile, String> { Bst::from_str_file(s, "_test_") }

    #[test] fn fast1() {
        assert_eq!(pb("use sys tem").unwrap_err(), "from_ast: filtered branch");
    }
    #[test] fn fast2() {
        let x = "each x in list[3, 4, 5] {\n    let y = x\n}\n";
        assert_eq!(format!("{}", pb(x).unwrap()), x);
    }
    #[test] fn fast3() {
        let x = "when {\n  1 => {\n    0\n  }\n}  else {\n    \n  }\n";
        assert_eq!(format!("{}", pb(x).unwrap()), x);
    }
    #[test] fn fast4() {
        assert_eq!(format!("{}", pb("return").unwrap()), "return 0");
    }
    #[test] fn fast5() {
        let x = "loop {\n    say(x)\n}\n";
        assert_eq!(format!("{}", pb(x).unwrap()), x);
    }
    #[test] fn faf1() {
        let berr = bsf("function (); function ()").unwrap_err();
        assert_eq!(berr, "from_ast_file: duplicate 'function'");
    }
    #[test] fn faf2() {
        let a = pa("3").unwrap();
        let berr = Bst::from_ast_file(&a).unwrap_err();
        assert_eq!(berr, "from_ast_file: not a File");
    }
    #[test] fn faf3() {
        let sa = "use std oil; function q(a, lazy b, ...); say(b)";
        let bf = bsf(sa).unwrap();
        assert_eq!(bf.uses.len(), 1);
        let (upath, ufile, uas) = &bf.uses[0];
        assert_eq!(upath, "std");
        assert_eq!(ufile, "oil");
        assert_eq!(uas, "oil");
        //let (ofname, args, dots, sdots) = bf.decl.unwrap();
        let fdecl = bf.decl.unwrap();
        assert_eq!(fdecl.oname.unwrap(), "q");
        assert_eq!(fdecl.named.len(), 2);
        let (aname1, astrict1) = &fdecl.named[0];
        assert_eq!(aname1, "a");
        assert!(astrict1);
        let (aname2, astrict2) = &fdecl.named[1];
        assert_eq!(aname2, "b");
        assert!(!astrict2);
        assert!(fdecl.has_dots);
        assert!(fdecl.strict_dots);
        assert_eq!(bf.file.len(), 1);
        assert_eq!(format!("{}", &bf.file[0]), "say(b)");
    }
    #[test] fn var1() { assert_eq!(pb("say(1)").unwrap().variant(), "Call"); }
    #[test] fn var2() { assert_eq!(pb("each a in b {}").unwrap().variant(), "Each"); }
    #[test] fn var4() { assert_eq!(pb("let a = 1").unwrap().variant(), "Let"); }
    #[test] fn var5() { assert_eq!(pb("list[]").unwrap().variant(), "List"); }
    #[test] fn var6() { assert_eq!(pb("loop{}").unwrap().variant(), "Loop"); }
    #[test] fn var7() { assert_eq!(pb("a").unwrap().variant(), "Name"); }
    #[test] fn var8() { assert_eq!(pb("return").unwrap().variant(), "Return"); }
    #[test] fn var9() { assert_eq!(pb("when{}").unwrap().variant(), "When"); }
    #[test] fn var10() {
        let c = Bst::Closure(Box::new(Bst::one()), 0, BRat::one());
        assert_eq!(c.variant(), "Closure");
    }
    #[test] fn var11() {
        let f = Bst::Func("a".to_owned(), "b".to_owned());
        assert_eq!(f.variant(), "Func");
    }
    #[test] fn dlist1() {
        let e = Bst::one().d_list("_t").unwrap_err();
        assert_eq!(e, "_t: not a List: Rat");
    }
    #[test] fn dltext1() {
        let e = pb("list[65, -1]").unwrap().d_list_text("_t").unwrap_err();
        assert_eq!(e, "_t elt 2: not Char: -1");
    }
    #[test] fn dltext2() {
        let e = pb("list[65, list[]]").unwrap().d_list_text("_t").unwrap_err();
        assert_eq!(e, "_t elt 2: not a Rat: List");
    }
    #[test] fn drat1() {
        let e = pb("list[]").unwrap().d_rat("_t").unwrap_err();
        assert_eq!(e, "_t: not a Rat: List");
    }
    #[test] fn drint1() {
        let e = pb("1/2").unwrap().d_rat_int("_t").unwrap_err();
        assert_eq!(e, "_t: not an Int: 1/2");
    }
    #[test] fn drnnint1() {
        let e = pb("-1").unwrap().d_rat_nonneg_int("_t").unwrap_err();
        assert_eq!(e, "_t: not a Nonnegative Int: -1");
    }
    #[test] fn drpint1() {
        let e = pb("0").unwrap().d_rat_pos_int("_t").unwrap_err();
        assert_eq!(e, "_t: not a Positive Int: 0");
    }
    #[test] fn drusz1() {
        let e = pb("1.9e19").unwrap().d_rat_usize("_t").unwrap_err();
        assert_eq!(e, "_t: too large for usize: 19000000000000000000");
    }
    #[test] fn drpusz1() {
        let e = pb("1.9e19").unwrap().d_rat_pos_usize("_t").unwrap_err();
        assert_eq!(e, "_t: too large for usize: 19000000000000000000");
    }
    #[test] fn drbool1() {
        let b = pb("0").unwrap().d_rat_bool("_t").unwrap();
        assert!(!b);
    }
    #[test] fn drbool2() {
        let b = pb("1").unwrap().d_rat_bool("_t").unwrap();
        assert!(b);
    }
    #[test] fn drbool3() {
        let b = pb("2").unwrap().d_rat_bool("_t").unwrap_err();
        assert_eq!(b, "_t: not a Bool: 2");
    }
    #[test] fn drname1() {
        let b = pb("2").unwrap().d_name("_t").unwrap_err();
        assert_eq!(b, "_t: not a Name: Rat");
    }
    #[test] fn bdisp1() {
        let x = "break";
        assert_eq!(format!("{}", pb(x).unwrap()), x);
    }
    #[test] fn bdisp2() {
        let x = "mutable x = 0";
        assert_eq!(format!("{}", pb(x).unwrap()), x);
    }
    #[test] fn bdisp3() {
        let x = "update x = 0";
        assert_eq!(format!("{}", pb(x).unwrap()), x);
    }
    #[test] fn bdisp4() {
        let f = Bst::Func("a".to_owned(), "b".to_owned());
        assert_eq!(format!("{}", f), "_fn(a/b)");
    }
    #[test] fn bdisp5() {
        let c = Bst::Closure(Box::new(Bst::one()), 0, BRat::one());
        assert_eq!(format!("{}", c), "_closure(1, 0, 1)");
    }
    #[test] fn bdisp6() {
        let s = Bst::Let(false, true, true, "a".to_owned(), Box::new(Bst::one()));
        assert_eq!(format!("{}", s), "_looplet a = 1");
    }
}

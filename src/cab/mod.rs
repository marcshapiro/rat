// Cab - file cabinet - containing loaded functions
mod funct;
mod sea;

use self::funct::Funct;
use self::sea::Sea;

use crate::ast::Ast;
use crate::brat::BRat;
use crate::bst::{Bst, BstFile, FuncDecl};
use crate::parse::{parse_str_to_ast_function_decl};

use std::collections::HashMap;

pub type BiType = fn(&mut Cab, &Option<(usize, BRat)>, Vec<Bst>) -> Result<Bst, String>;

pub struct Cab {
    net: HashMap<String, HashMap<String, Funct>>, // path -> func -> Funct // loaded functions
    seas: Vec<Sea>, // call stack (top is current scope)
    nextid: BRat, // id for next Sea
}

impl Cab {
    pub fn new() -> Self {
        let net = HashMap::new();
        let seas = vec![];
        let nextid = BRat::zero();
        Cab { net, seas, nextid }
    }
    pub fn close(&self, b: Bst) -> Bst {
        match b {
            Bst::Closure(_, _, _) => b,
            _ => {
                let ix = self.topix();
                let id = self.atid(ix);
                Bst::Closure(Box::new(b), ix, id.clone())
            },
        }
    }
    pub fn push(&mut self, used: HashMap<String, (String, String)>) { // Call
        let sea = Sea::new(self.nextid.clone(), used);
        self.nextid = &self.nextid + &BRat::one();
        self.seas.push(sea);
    }
    pub fn pop(&mut self) { // Return
        self.seas.pop();
    }
    fn topix(&self) -> usize {
        self.seas.len() - 1
    }
    fn atid(&self, ix: usize) -> &BRat {
        self.seas[ix].id()
    }
    fn get_ix(&self, idx: &Option<(usize, BRat)>) -> Result<usize, String> {
        match idx {
            None => Ok(self.topix()),
            Some((ix, id)) => {
                let aid = self.atid(*ix);
                if aid != id {
                    return Err(format!("sea[{}].id = {} <> {}", ix, aid, id));
                }
                Ok(*ix)
            }
        }
    }
    pub fn atget(&self, name: &str, idx: &Option<(usize, BRat)>) -> Result<Bst, String> {
        let ix = self.get_ix(idx)?;
        self.seas[ix].get(name)
    }
    pub fn athas(&self, name: &str, idx: &Option<(usize, BRat)>) -> Result<bool, String> {
        let ix = self.get_ix(idx)?;
        Ok(self.seas[ix].has(name))
    }
    pub fn atput(&mut self, name: &str, idx: &Option<(usize, BRat)>,
            create: bool, is_const: bool, val: Bst) -> Result<(), String> {
        let ix = self.get_ix(idx)?;
        self.seas[ix].put(name, create, is_const, val)
    }
    pub fn atdel(&mut self, name: &str, idx: &Option<(usize, BRat)>) -> Result<(), String> {
        let ix = self.get_ix(idx)?;
        self.seas[ix].del(name);
        Ok(())
    }
    pub fn is_const(&mut self, name: &str, idx: &Option<(usize, BRat)>) -> Result<bool, String> {
        let ix = self.get_ix(idx)?;
        self.seas[ix].is_const(name)
    }
    fn netput(&mut self, path: &str, func: &str, funkt: Funct) -> Result<(), String> {
        if !self.nethaspath(path) {
            self.net.insert(path.to_owned(), HashMap::new());
        }
        match self.net.get_mut(path) {
            None => return Err("Inserted map is lost".to_owned()),
            Some(fmap) => { fmap.insert(func.to_owned(), funkt); },
        }
        Ok(())
    }
    fn nethaspath(&self, path: &str) -> bool {
        self.net.contains_key(path)
    }
    fn nethas(&self, path: &str, func: &str) -> bool {
        self.nethaspath(path)
            && self.net[path].contains_key(func)
    }
    pub fn netget(&self, path: &str, func: &str) -> Result<Funct, String> {
        match self.net.get(path) {
            None => Err(format!("{}/{}: No functions loaded in path {}", path, func, path)),
            Some(fmap) => match fmap.get(func) {
                None => Err(format!("{}/{}: Function {} not loaded", path, func, func)),
                Some(rf) => Ok(rf.clone()),
            }
        }
    }
    fn getremap(&self) -> HashMap<String, Vec<String>> {
        let mut remap = HashMap::new();
        for (path, fmap) in self.net.iter() {
            remap.insert(path.clone(), fmap.keys().map(|s|s.to_owned()).collect());
        }
        remap
    }
    fn getfunkts(&mut self) -> Vec<&mut Funct> {
        let mut funkts = vec![];
        for (_path, fmap) in self.net.iter_mut() {
            for (_func, funkt) in fmap.iter_mut() {
                funkts.push(funkt);
            }
        }
        funkts
    }
    pub fn postload(&mut self) -> Result<(), String> {
        let remap = self.getremap();
        for funkt in self.getfunkts() {
            match remap.get("auto") {
                None => {},
                Some(afuncs) => for afunc in afuncs.iter() {
                    funkt.uses("auto", afunc, afunc)?;
                }
            }
        }
        Ok(())
    }
    pub fn load_file(&mut self, path: &str, func: &str) -> Result<(), String> {
        if self.nethas(path, func) { // don't load twice
            return Ok(());
        }
        let filename = format!("{}/{}.rat", path, func);
        let sr = std::fs::read_to_string(filename.clone());
        let bf = match sr {
            Ok(s) => Bst::from_str_file(&s, path)?,
            Err(e) => return Err(format!("{}: failed to read file: {}", filename, e)),
        };
        self.load_file_rec(path, func, &bf, true)?;
        Ok(())
    }
    pub fn load_file_rec(&mut self, path: &str, func: &str, bf: &BstFile, load_use: bool) -> Result<Funct, String> {
        let fdecl = match &bf.decl {
            None => FuncDecl{ oname: None, named: vec![], has_dots: true, strict_dots: true },
            Some(fd) => fd.clone(),
        };
        let mut funkt = Funct::defined(bf.file.clone(), fdecl.named, fdecl.has_dots, fdecl.strict_dots);
        match &fdecl.oname {
            None => {},
            Some(asname) => {
                funkt.uses(path, func, asname)?;
            },
        }
        for (upath, ufunc, uname) in &bf.uses {
            funkt.uses(upath, ufunc, uname)?;
        }
        self.netput(path, func, funkt.clone())?;
        if load_use {
            for (upath, ufunc, _) in &bf.uses {
                self.load_file(upath, ufunc)?;
            }
        }
        Ok(funkt)
    }
    pub fn use_all_load(&mut self, path: &str) -> Result<(), String> { // FIXME: use_all_auto_load
        for func in use_all_list(path)?.iter() {
            self.load_file(path, func)?;
        }
        Ok(())
    }
    pub fn add_bi(&mut self, spath: &str, fdtxt: &str,  builtin: BiType) -> Result<(), String> {
        let fdtxt_all = format!("function {}", fdtxt);
        match parse_str_to_ast_function_decl(&fdtxt_all, "_builtin_")? {
            Ast::FnDecl(oname, named, dots, strict_dots) => {
                match oname {
                    None => Err(format!("add_bi: no name specified in decl '{}'", fdtxt_all)),
                    Some(fname) => {
                        let funkt = Funct::builtin(builtin, named, dots, strict_dots);
                        self.netput(spath, &fname, funkt)?;
                        Ok(())
                    },
                }
            },
            _ => Err("add_bi: did not parse to FnDecl".to_owned()),
        }
    }
    pub fn eval_file(&mut self, idx: &Option<(usize, BRat)>, vb: &[Bst]) -> Result<Bst, String> {
        let r = self.vec_eval(idx, vb)?;
        Ok(match r {
            Bst::Return(xr) => *xr,
            _ => r,
        })
    }
    pub fn eval(&mut self, idx: &Option<(usize, BRat)>, bbb: &Bst) -> Result<Bst, String> {
        match bbb {
            Bst::Break | Bst::Func(_, _) | Bst::Rat(_) => Ok(bbb.clone()),
            Bst::Call(xfunc, vargs) => {
                let func = self.eval(idx, xfunc)?;
                match func {
                    Bst::Func(path, fname) => {
                        let funkt = self.netget(&path, &fname)?;
                        Ok(funkt.call(vargs, self, idx)?)
                    },
                    _ => Err(format!("May only call a function: {}", func.variant())),
                }
            },
            Bst::Closure(xval, ix, id) => self.eval(&Some((*ix, id.clone())), xval),
            Bst::Each(var, blist, stmts) => {
                if self.athas(var, &None)? {
                    return Err(format!("each: {} already exists", var));
                }
                match self.eval(idx, blist)? {
                    Bst::List(list) => {
                        if !list.is_empty() {
                            for elt in list {
                                self.atput(var, idx, true, true, elt)?;
                                let r = self.vec_eval(idx, stmts)?;
                                match r {
                                    Bst::Break => return Ok(Bst::zero()),
                                    Bst::Return(_) => return Ok(r),
                                    _ => {},
                                }
                                self.atdel(var, idx)?;
                            }
                        }
                        Ok(Bst::zero())
                    },
                    _ => Err(format!("each: Not a List: {}", blist.variant())),
                }
            },
            Bst::Let(is_create, is_const, is_strict, var, xval) => {
                let val = if *is_strict {
                    self.eval(idx, xval)?
                } else {
                    (**xval).clone()
                };
                self.atput(var, idx, *is_create, *is_const, val)?;
                Ok(Bst::zero())
            },
            Bst::List(vb) => {
                let mut ve = vec![];
                for b in vb {
                    ve.push(self.eval(idx, b)?);
                }
                Ok(Bst::List(ve))
            },
            Bst::Loop(vb) => {
                loop {
                    let r = self.vec_eval(idx, vb)?;
                    match r {
                        Bst::Break => return Ok(Bst::zero()),
                        Bst::Return(..) => return Ok(r),
                        _ => {}
                    }
                }
            },
            Bst::Name(name) => self.eval(idx, &self.atget(name, idx)?),
            Bst::Return(xb) => Ok(Bst::Return(Box::new(self.eval(idx, xb)?))),
            Bst::When(vcb, velse) => {
                let t = Bst::one();
                for (bcond, vbody) in vcb {
                    if t == self.eval(idx, bcond)? {
                        return self.vec_eval(idx, vbody);
                    }
                }
                self.vec_eval(idx, velse)
            },
        }
    }
    fn vec_eval(&mut self, idx: &Option<(usize, BRat)>, stmts: &[Bst]) -> Result<Bst, String> {
        let mut lastval = Bst::zero();
        for stmt in stmts {
            lastval = self.eval(idx, stmt)?;
            match lastval {
                Bst::Break | Bst::Return(..) => break,
                _ => { }
            }
        }
        Ok(lastval)
    }

}

fn use_all_list(path: &str) -> Result<Vec<String>, String> {
    let mut vv = vec![];
    match std::fs::read_dir(path.to_owned()) {
        Ok(rd) => for oentry in rd {
            match oentry {
                Err(e) => { return Err(format!("use all {}: Bad file: {}", path, e)); },
                Ok(entry) => {
                    if let Ok(filename) = entry.file_name().into_string() {
                        if let Some(froot) = filename.strip_suffix(".rat") {
                            vv.push(froot.to_owned());
                        }
                    }
                },
            }
        },
        Err(e) => { return Err(format!("use all {}: Failed: {}", path, e)); },
    }
    Ok(vv)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bi::register_bi;

    fn taxi(use_bi: bool) -> Cab {
        let mut cab = Cab::new();
        cab.push(HashMap::new());
        if use_bi {
            register_bi(&mut cab).unwrap();
        }
        cab
    }
    fn pb(s: &str) -> Result<Bst, String> {
        let mut cab = taxi(false);
        let b = Bst::from_str_stmt(s, "_t").unwrap();
        cab.eval(&None, &b)
    }
    fn raw_call(cab: &mut Cab, bbb: Vec<Bst>, uses: Vec<(&str, &str, &str)>) -> Result<Bst, String> {
        let mut funkt = Funct::defined(bbb, vec![], false, true);
        let m = cab.net.get("auto").unwrap();
        for afunc in m.keys() {
            funkt.uses("auto", afunc, afunc)?;
        }
        for (upath, ufunc, uname) in uses {
            funkt.uses(upath, ufunc, uname)?;
        }
        funkt.call(&[], cab, &None)
    }

    #[test] fn ev1() {
        let r = pb("each e in list[1] { 1; break; 2 }").unwrap();
        assert_eq!(format!("{}", r), "0")
    }
    #[test] fn ev2() {
        let r = pb("each e in list[1, 2] { 1; 2 }").unwrap();
        assert_eq!(format!("{}", r), "0")
    }
    #[test] fn ev3() {
        let r = pb("if 1 { let x = 3; return x; }").unwrap();
        assert_eq!(format!("{}", r), "return 3")
    }
    #[test] fn ev4() {
        let e = pb("if 1 { let x = 3; let x = 4; }").unwrap_err();
        assert_eq!(format!("{}", e), "x: variable already exists");
    }
    #[test] fn ev5() {
        let e = pb("x").unwrap_err();
        assert_eq!(format!("{}", e), "x: not a variable");
    }
    #[test] fn ev6() {
        let e = pb("if 1 { let x = 3; update x = 4; }").unwrap_err();
        assert_eq!(format!("{}", e), "x: not mutable; cannot update");
    }
    #[test] fn ev7() {
        let e = pb("update x = 4").unwrap_err();
        assert_eq!(format!("{}", e), "x: not a variable; cannot update");
    }
    #[test] fn ev8() {
        let s = "if 1 { let x = 3; return x; }";
        let mut cab = taxi(false);
        let b = Bst::from_str_stmt(s, "_t").unwrap();
        let r = cab.eval(&Some((0, BRat::zero())), &b).unwrap();
        assert_eq!(format!("{}", r), "return 3")
    }
    #[test] fn ev9() {
        let s = "if 1 { let x = 3; each x in list[1] { 1; break; 2 }}";
        let e = pb(s).unwrap_err();
        assert_eq!(format!("{}", e), "each: x already exists")
    }
    #[test] fn ev10() {
        let s = "loop { 1; 2; if 1 { break; } }";
        let r = pb(s).unwrap();
        assert_eq!(format!("{}", r), "0");
    }
    #[test] fn ev11() {
        let s = "loop { 1; 2; if 1 { return 3; } }";
        let r = pb(s).unwrap();
        assert_eq!(format!("{}", r), "return 3");
    }
    #[test] fn ev12() {
        let s = "if 0 { 1 } else { 2 }";
        let r = pb(s).unwrap();
        assert_eq!(format!("{}", r), "2");
    }
    #[test] fn ev13() {
        let r = pb("each e in list[1, 2] { 1; 2; return 3 }").unwrap();
        assert_eq!(format!("{}", r), "return 3")
    }
    #[test] fn ev14() {
        let b = Bst::from_str_stmt("1 + 2", "_t").unwrap();
        let mut cab = taxi(true);
        let r = raw_call(&mut cab, vec![b], vec![]).unwrap();
        assert_eq!(format!("{}", r), "3");
    }
    #[test] fn ev15() {
        let b = Bst::from_str_stmt("is_mutable(a)", "_t").unwrap();
        let mut cab = taxi(true);
        let u1 = ("sys", "is_mutable", "is_mutable");
        let e = raw_call(&mut cab, vec![b], vec![u1]).unwrap_err();
        assert_eq!(e, "a: not a variable");
    }
    #[test] fn ev16() {
        let b = Bst::from_str_stmt("is_mutable(is_mutable)", "_t").unwrap();
        let mut cab = taxi(true);
        let u1 = ("sys", "is_mutable", "is_mutable");
        let r = raw_call(&mut cab, vec![b], vec![u1]).unwrap();
        assert_eq!(format!("{}", r), "0");
    }
    #[test] fn ev17() {
        let e = pb("each e in 1 { 1; 2; return 3 }").unwrap_err();
        assert_eq!(e, "each: Not a List: Rat")
    }
    #[test] fn ev18() {
        let r = pb("let x = lazy 1 + 2").unwrap();
        assert_eq!(format!("{}", r), "0");
    }
}


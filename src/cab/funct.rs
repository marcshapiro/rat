// Funct - wrap builtin and defined functions
use crate::brat::BRat;
use crate::bst::Bst;
use crate::cab::{BiType, Cab};

use std::collections::HashMap;

#[derive(Clone)]
enum Funky {
    Builtin(BiType),
    Defined(
        HashMap<String, (String, String)>, // used // as_name -> path/func // includes usealls after second pass
        Vec<Bst>), // function
}

#[derive(Clone)]
pub struct Funct {
    named: Vec<(String, bool)>, // name, is_strict // for built-ins, names are just for error messages
    dots: bool,
    strict_dots: bool,
    funk: Funky,
}

impl Funct {
    pub fn builtin(func: BiType, named: Vec<(String, bool)>, dots: bool, strict_dots: bool) -> Funct {
        Funct { named, dots, strict_dots, funk: Funky::Builtin(func) }
    }
    pub fn defined(func: Vec<Bst>, named: Vec<(String, bool)>,
            dots: bool, strict_dots: bool) -> Funct {
        let used = HashMap::new();
        let funk = Funky::Defined(used, func);
        Funct { named, dots, strict_dots, funk  }
    }
    pub fn uses(&mut self, path: &str, func: &str, name: &str) -> Result<(), String> {
        match &mut self.funk {
            Funky::Builtin(_) => Ok(()),
            Funky::Defined(used, _) => {
                if used.contains_key(name) {
                    Err(format!("Cannot re-use as '{}'", name))
                } else {
                    used.insert(name.to_owned(), (path.to_owned(), func.to_owned()));
                    Ok(())
                }
            },
        }
    }
    pub fn call(&self, args: &[Bst], cab: &mut Cab, idx: &Option<(usize, BRat)>) -> Result<Bst, String> {
        // validate arg count
        let n_act = args.len();
        let n_named = self.named.len();
        if self.dots {
            if n_act < n_named {
                return Err(format!("Expected at least {} args, found {}", n_named, n_act));
            }
        } else if n_named != n_act {
            return Err(format!("Expected {} args, found {}", n_named, n_act));
        }

        // evaluate or close args
        let mut ergs = vec![];
        for (i, arg) in args.iter().enumerate() {
            let is_strict =
                if i < n_named {
                    let (_, iss) = self.named[i];
                    iss
                } else {
                    self.strict_dots
                };
            ergs.push(
                if is_strict {
                    cab.eval(idx, arg)?
                } else {
                    cab.close(arg.clone())
                }
            );
        }

        Ok(match &self.funk {
            Funky::Builtin(bif) => bif(cab, idx, ergs)?, // bi run in caller cab, get args directly
            Funky::Defined(used, bst) => {
                // create function env
                cab.push(used.clone());

                // make args available to function
                for (i, (name, _)) in self.named.iter().enumerate() {
                    let erg = ergs[i].clone();
                    cab.atput(name, &None, true, true, erg)?;
                }
                if self.dots {
                    cab.atput("args", &None, true, true, Bst::List(ergs[n_named..].to_vec()))?;
                }

                // call function
                let result = cab.eval_file(&None, bst)?;

                // drop function env
                cab.pop();

                result
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bi1(_cab: &mut Cab, _idx: &Option<(usize, BRat)>, _args: Vec<Bst>)
            -> Result<Bst, String> {
        Ok(Bst::one())
    }

    #[test] fn fn2() {
        let mut f = Funct::defined(vec![Bst::one()], vec![], true, true);
        f.uses("p", "f", "a").unwrap();
        let e = f.uses("p", "g", "a").unwrap_err();
        assert_eq!(e, "Cannot re-use as 'a'");
    }
    #[test] fn fn3() {
        let f = Funct::defined(vec![Bst::one()], vec![], false, true);
        let e = f.call(&vec![Bst::one()], &mut Cab::new(), &None).unwrap_err();
        assert_eq!(e, "Expected 0 args, found 1");
    }
    #[test] fn fn4() {
        let a1 = ("x".to_owned(), true);
        let f = Funct::defined(vec![Bst::one()], vec![a1], true, true);
        let e = f.call(&vec![], &mut Cab::new(), &None).unwrap_err();
        assert_eq!(e, "Expected at least 1 args, found 0");
    }
    #[test] fn fn5() {
        let a1 = ("x".to_owned(), true);
        let f = Funct::defined(vec![Bst::one()], vec![a1], true, false);
        let r = f.call(&vec![Bst::one(), Bst::one()], &mut Cab::taxi(false), &None).unwrap();
        assert_eq!(r, Bst::one());
    }
    #[test] fn fn6() {
        let a1 = ("x".to_owned(), true);
        let f = Funct::builtin(bi1, vec![a1], true, false);
        let r = f.call(&vec![Bst::one(), Bst::one()], &mut Cab::taxi(false), &None).unwrap();
        assert_eq!(r, Bst::one());
    }
}

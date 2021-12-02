use crate::brat::BRat;
use crate::bst::Bst;

use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sea {
    id: BRat, // counter - unique identifier // checked in closure.eval
    used: HashMap<String, (String, String)>, // use and 'function' // points to Cab.net
    vars: HashMap<String, (bool, Bst)>, // local variables (check names in used first)
}

impl Sea {
    pub fn new(id: BRat, used: HashMap<String, (String, String)>) -> Sea {
        Sea { id, used, vars: HashMap::new() }
    }
    pub fn get(&self, name: &str) -> Result<Bst, String> {
        Ok(match self.used.get(name) {
            Some((path, func)) => Bst::Func(path.clone(), func.clone()),
            None => match self.vars.get(name) {
                Some((_, bst)) => bst.clone(),
                None => return Err(format!("{}: not a variable or function", name)),
            },
        })
    }
    pub fn has(&self, name: &str) -> bool {
        self.used.contains_key(name) || self.vars.contains_key(name)
    }
    pub fn put(&mut self, name: &str, create: bool, is_const: bool, val: Bst) -> Result<(), String> {
        if create {
            if self.has(name) {
                return Err(format!("{}: variable already exists", name));
            }
        } else {
            if is_const {
                return Err(format!("{}: const update", name));
            }
            match self.vars.get(name) {
                None => return Err(format!("{}: not a variable; cannot update", name)),
                Some((k, _)) => if *k {
                    return Err(format!("{}: not mutable; cannot update", name));
                },
            }
        }
        self.vars.insert(name.to_string(), (is_const, val));
        Ok(())
    }
    pub fn del(&mut self, name: &str) {
        self.vars.remove(name);
    }
    pub fn is_const(&mut self, name: &str) -> Result<bool, String> {
        if self.used.contains_key(name) {
            return Ok(true);
        }
        match self.vars.get(name) {
            None => Err(format!("{}: not a variable", name)),
            Some((k, _)) => Ok(*k),
        }
    }
    pub fn id(&self) -> &BRat { &self.id }
    pub fn get_used(&self) -> HashMap<String, (String, String)> {
        let mut used = self.used.clone();
        for (name, (is_const, bbb)) in &self.vars {
            if let (true, Bst::Func(path, func)) = (is_const, bbb) {
                used.insert(name.to_owned(), (path.to_owned(), func.to_owned()));
            }
        }
        used
    }
    pub fn get_vars(&self) -> HashMap<String, (bool, String)> {
        let mut vars = HashMap::new();
        for (name, (is_const, bbb)) in &self.vars {
            let variant = bbb.variant();
            if !is_const || "Func" != variant {
                vars.insert(name.to_owned(), (*is_const, variant.to_owned()));
            }
        }
        vars
    }
}

// Ast for rat
use std::fmt;

use crate::brat::BRat;
use crate::udisp::write_vec;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Ast {
    Break,
    Call(Box<Ast>, Vec<Ast>), // func, args
    Each(String, Box<Ast>, Vec<Ast>),
    File(Vec<Ast>),
    FnDecl(Option<String>, Vec<(String, bool)>, bool, bool), // func_name (arg_name, is_strict) has_dots strict_dots
    Let(bool, bool, bool, String, Box<Ast>), // create?, const?, strict?, var, val
    List(Vec<Ast>),
    Loop(Vec<Ast>),
    Name(String),
    RatLit(BRat),
    Return(Option<Box<Ast>>),
    Use(String, String, String), // path, func, var
    When(Vec<(Ast, Vec<Ast>)>, Vec<Ast>), // (cond, body), else
}

impl fmt::Display for Ast {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Ast::Break => write!(f, "break"),
            Ast::Call(name, args) => write_vec(f, &format!("{}(", name), args, ", ", ")"),
            Ast::Each(name, list, stmts) => {
                write!(f, "each {} in {} ", name, list)?;
                write_vec(f, "{", stmts, "; ", "}")
            },
            Ast::File(seq) => write_vec(f, "", seq, "; ", "\n"),
            Ast::FnDecl(oname, vargs, has_dots, strict_dots) => {
                write!(f, "function ")?;
                match oname {
                    None => {},
                    Some(name) => write!(f, "{}", name)?,
                };
                write!(f, "(")?;
                let mut first = true;
                for (arg, is_strict) in vargs {
                    if first {
                        first = false;
                    } else {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}{}", if *is_strict {""} else { "lazy " }, arg)?;
                }
                if *has_dots {
                    if !vargs.is_empty() {
                        write!(f, ", ")?;
                    }
                    if !*strict_dots {
                        write!(f, "lazy ")?;
                    }
                    write!(f, "...")?;
                }
                write!(f, ")")
            },
            Ast::Let(is_create, is_const, is_strict, name, value) => {
                let which = match (is_create, is_const) {
                    (true, true) => "let",
                    (true, false) => "mutable",
                    (false, _) => "update",
                };
                let how = if *is_strict {""} else { "lazy "};
                write!(f, "{} {} = {}{}", which, name, how, value)
            },
            Ast::List(list) => write_vec(f, "list[", list, ", ", "]"),
            Ast::Loop(seq) => write_vec(f, "loop {", seq, "; ", "}"),
            Ast::Name(n) => write!(f, "{}", n),
            Ast::RatLit(r) => write!(f, "{}", r),
            Ast::Return(r) => match r {
                None => write!(f, "return"),
                Some(e) => write!(f, "return {}", e),
            },
            Ast::Use(p, func, v) => write!(f, "use {} {} as {}", p, func, v),
            Ast::When(pairs, elze) => {
                write!(f, "when {{")?;
                for (cond, thyn) in pairs {
                    write!(f, " {} => ", cond)?;
                    write_vec(f, "{", thyn, "; ", "};")?;
                }
                write_vec(f, "} else => {", elze, "; ", "}")
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn da(a: Ast, x: &str) { assert_eq!(format!("{}", a), x); }

    #[test] fn d1() { da(Ast::Break, "break"); }
    #[test] fn d2() {
        let list = Box::new(Ast::Name("b".to_owned()));
        da(Ast::Each("a".to_owned(), list, vec![]), "each a in b {}");
    }
    #[test] fn d3() {
        let a1 = ("b".to_owned(), false);
        let a2 = ("c".to_owned(), true);
        let fd = Ast::FnDecl(Some("a".to_owned()), vec![a1, a2], true, true);
        da(fd, "function a(lazy b, c, ...)");
    }
    #[test] fn d5() { da(Ast::Loop(vec![]), "loop {}"); }
    #[test] fn d6() { da(Ast::Return(None), "return"); }
    #[test] fn d7() {
        let val = Box::new(Ast::List(vec![]));
        da(Ast::Return(Some(val)), "return list[]");
    }
}

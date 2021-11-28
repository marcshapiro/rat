use crate::ast::Ast;
use crate::brat::BRat;
use self::parser::{Parser, Token};

use logos::{Lexer, Logos};
use pomelo::pomelo;

fn within_quotes(lex: &mut Lexer<Lexi>) -> String {
    let s = lex.slice();
    let w = &s[1..(s.len()-1)];
    w.to_string()
}

fn to_brat(lex: &mut Lexer<Lexi>) -> BRat {
    match BRat::from_str(lex.slice()) {
        Some(r) => r,
        None => BRat::zero(),
    }
}

#[derive(Logos, Debug, PartialEq)]
enum Lexi {
    #[regex(r"'[^\n']*'", within_quotes)]
    TextLit(String),

    #[regex(r"[+-]?[0-9]+(\.[0-9]*)?([eE][+-]?[0-9]+)?(/[0-9]+(\.[0-9]*)?([eE][+-]?[0-9]+)?)?", to_brat)]
    RatLit(BRat),

    #[regex(r"[\t ]+", logos::skip)]
    White,

    #[regex(r"#[^\r\n]*", logos::skip)]
    Comment,

    #[regex("\r\n|\n\r|\n|\r")]
    Line,

    // keywords before Word
    #[token("-inf")] Minf,
    #[token("as")] As,
    #[token("break")] Break,
    #[token("case")] Case,
    #[token("each")] Each,
    #[token("else")] Else,
    #[token("function")] Function,
    #[token("if")] If,
    #[token("in")] In,
    #[token("inf")] Inf,
    #[token("lazy")] Lazy,
    #[token("let")] Let,
    #[token("list")] List,
    #[token("loop")] Loop,
    #[token("mutable")] Mutable,
    #[token("my")] My,
    #[token("nan")] Nan,
    #[token("return")] Return,
    #[token("std")] Std,
    #[token("switch")] Switch,
    #[token("sys")] Sys,
    #[token("update")] Update,
    #[token("use")] Use,
    #[token("usr")] Usr,
    #[token("when")] When,

    #[regex(r"[A-Za-z][A-Za-z0-9_?]*", |lex| lex.slice().to_string())]
    Word(String),

    // long char punct
    #[token("...")] Dots3,

    // two char punct before one
    #[token("==")] Eqq,
    #[token("<=")] Le,
    #[token(">=")] Ge,
    #[token("<>")] Ne,
    #[token("&&")] Andd,
    #[token("||")] Orr,
    #[token("=>")] Arrow,

    // one char punct
    #[token("[")] LSq,
    #[token("]")] RSq,
    #[token("(")] LPar,
    #[token(")")] RPar,
    #[token("{")] LCurl,
    #[token("}")] RCurl,
    #[token("=")] Eq,
    #[token(";")] Semi,
    #[token(",")] Comma,
    #[token("<")] Lt,
    #[token(">")] Gt,
    #[token("+")] Add,
    #[token("-")] Sub,
    #[token("*")] Mul,
    #[token("/")] Div,
    #[token("^")] Pow,
    #[token("!")] Not,

    #[error] Err,
}

fn if_to_when(cond: Ast, then_: Vec<Ast>, else_: Vec<Ast>) -> Ast {
    Ast::When(vec![(cond, then_)], else_)
}

fn case_to_when(xpr: Ast, tail: (Vec<(Ast, Vec<Ast>)>, Vec<Ast>)) -> Ast {
    let (pairs, else_) = tail;
    let mut bodies = vec![];
    for (maybe, then_) in pairs {
        let cond = named_call("op_eq", vec![maybe, xpr.clone()]);
        bodies.push((cond, then_));
    }
    Ast::When(bodies, else_)
}

fn switch_to_when(cmp: Ast, xpr: Ast, tail: (Vec<(Ast, Vec<Ast>)>, Vec<Ast>)) -> Ast {
    let (pairs, else_) = tail;
    let mut bodies = vec![];
    for (maybe, then_) in pairs {
        let cond = Ast::Call(Box::new(cmp.clone()), vec![maybe, xpr.clone()]);
        bodies.push((cond, then_));
    }
    Ast::When(bodies, else_)
}

fn named_call(name: &str, args: Vec<Ast>) -> Ast {
    Ast::Call(Box::new(Ast::Name(name.to_string())), args)
}

fn string_to_list(s: String) -> Ast {
    Ast::List(s.chars().map(
            |c| Ast::RatLit(BRat::from_char(c))
        ).collect())
}

pomelo! {
    %include {
        use crate::ast::Ast;
        use crate::brat::BRat;
        use crate::parse::{case_to_when, if_to_when, named_call, string_to_list, switch_to_when};
    }
    %type Word String;
    %type RatLit BRat;
    %type TextLit String;
    %token #[derive(Clone,Debug)]
        pub enum Token {};

    %type file Ast;
    file ::= stmt_list(A) { Ast::File(A) };

    %type stmt_list_1 Vec<Ast>;
    stmt_list_1 ::= stmt_list_1(A) Semi stmt(B) { let mut a = A; a.push(B); a };
    stmt_list_1 ::= stmt(A) { vec![A] };

    %type stmt_list Vec<Ast>;
    stmt_list ::= stmt_list_1(A) Semi { A };
    stmt_list ::= stmt_list_1(A) { A };
    stmt_list ::= { vec![] };

    %type stmt_list_curl Vec<Ast>;
    stmt_list_curl ::= LCurl stmt_list(A) RCurl { A };

    %type arg (String, bool);
    arg ::= Lazy Word(A) { (A, false) }; // strict?
    arg ::= Word(A) { (A, true) };

    %type elli bool;
    elli ::= Lazy Dots3 { false }; // strict?
    elli ::= Dots3 { true };

    %type arg_list_1 Vec<(String, bool)>; // strict?
    arg_list_1 ::= arg_list_1(A) Comma arg(B) { let mut a = A; a.push(B); a };
    arg_list_1 ::= arg(A) { vec![A] };

    %type arg_list (Vec<(String, bool)>, bool, bool); // [arg strict] hasdots strictdots
    arg_list ::= arg_list_1(A) Comma elli(B) { (A, true, B) };
    arg_list ::= arg_list_1(A) { (A, false, false) };
    arg_list ::= elli(A) { (vec![], true, A) };
    arg_list ::= { (vec![], false, false) };

    %type path &'static str;
    path ::= My { "my" };
    path ::= Std { "std" };
    path ::= Sys { "sys" };
    path ::= Usr { "usr" };

    %type stmt Ast;
    stmt ::= expr(A) { A };
    stmt ::= Break { Ast::Break };
    stmt ::= Function Word(A) LPar arg_list(B) RPar { let (x, y, z) = B; Ast::FnDecl(Some(A), x, y, z) };
    stmt ::= Function LPar arg_list(A) RPar { let (x, y, z) = A; Ast::FnDecl(None, x, y, z) };
    stmt ::= Return expr(A) { Ast::Return(Some(Box::new(A))) };
    stmt ::= Return { Ast::Return(None) };
    stmt ::= Let Word(A) Eq Lazy expr(B) { Ast::Let(true, true, false, A, Box::new(B)) }; // create, const, strict
    stmt ::= Let Word(A) Eq expr(B) { Ast::Let(true, true, true, A, Box::new(B)) };
    stmt ::= Mutable Word(A) Eq Lazy expr(B) { Ast::Let(true, false, false, A, Box::new(B)) };
    stmt ::= Mutable Word(A) Eq expr(B) { Ast::Let(true, false, true, A, Box::new(B)) };
    stmt ::= Update Word(A) Eq Lazy expr(B) { Ast::Let(false, false, false, A, Box::new(B)) };
    stmt ::= Update Word(A) Eq expr(B) { Ast::Let(false, false, true, A, Box::new(B)) };
    stmt ::= Loop stmt_list_curl(A) { Ast::Loop(A) };
    stmt ::= Each Word(A) In expr(B) stmt_list_curl(C) { Ast::Each(A, Box::new(B), C) };
    stmt ::= Use path(A) Word(B) As Word(C) { Ast::Use(A.to_string(), B, C) };
    stmt ::= Use path(A) Word(B) { Ast::Use(A.to_string(), B.clone(), B) };

    %type expr_pair (Ast, Vec<Ast>);
    expr_pair ::= expr(A) Arrow stmt_list_curl(B) { (A, B) };
    expr_pair ::= expr(A) Arrow stmt(B) { (A, vec![B]) };

    %type expr_pair_list_1 Vec<(Ast, Vec<Ast>)>;
    expr_pair_list_1 ::= expr_pair_list_1(A) Semi expr_pair(B) { let mut a = A; a.push(B); a };
    expr_pair_list_1 ::= expr_pair(A) { vec![A] };

    %type expr_pair_list Vec<(Ast, Vec<Ast>)>;
    expr_pair_list ::= expr_pair_list_1(A) Semi { A };
    expr_pair_list ::= expr_pair_list_1(A) { A };
    expr_pair_list ::= { vec![] };

    %type switch_head Ast;
    switch_head ::= Switch Eqq { Ast::Name("op_eq".to_string()) };
    switch_head ::= Switch Ne { Ast::Name("op_ne".to_string()) };
    switch_head ::= Switch Le { Ast::Name("op_le".to_string()) };
    switch_head ::= Switch Lt { Ast::Name("op_lt".to_string()) };
    switch_head ::= Switch Ge { Ast::Name("op_ge".to_string()) };
    switch_head ::= Switch Gt { Ast::Name("op_gt".to_string()) };
    switch_head ::= Switch LPar expr(A) RPar { A }; // eval to binary function

    %type when_tail (Vec<(Ast, Vec<Ast>)>, Vec<Ast>);
    when_tail ::= LCurl expr_pair_list(A) RCurl Else stmt_list_curl(B) { (A, B) };
    when_tail ::= LCurl expr_pair_list(A) RCurl { (A, vec![]) };

    %type expr_list_1 Vec<Ast>;
    expr_list_1 ::= expr_list_1(A) Comma expr(B) { let mut a = A; a.push(B); a };
    expr_list_1 ::= expr(A) { vec![A] };

    %type expr_list Vec<Ast>;
    expr_list ::= expr_list_1(A) Comma { A };
    expr_list ::= expr_list_1(A) { A };
    expr_list ::= { vec![] };

    %left Orr;
    %left Andd;
    %nonassoc Eqq Ne Gt Ge Lt Le;
    %left Add Sub;
    %left Mul Div;
    %right Not;
    %right Pow;
    %nonassoc RPar;

    %type expr Ast;
    expr ::= LPar expr(A) RPar { A };
    expr ::= RatLit(A) [RPar] { Ast::RatLit(A) };
    expr ::= TextLit(A) [RPar] { string_to_list(A) };
    expr ::= List LSq expr_list(A) RSq [RPar] { Ast::List(A) };
    expr ::= Word(A) LPar expr_list(B) RPar [RPar] { Ast::Call(Box::new(Ast::Name(A)), B) };
    expr ::= LPar expr(A) RPar LPar expr_list(B) RPar { Ast::Call(Box::new(A), B) };
    expr ::= If expr(A) stmt_list_curl(B) Else stmt_list_curl(C) [RPar] { if_to_when(A, B, C) };
    expr ::= If expr(A) stmt_list_curl(B) [RPar] { if_to_when(A, B, vec![]) };
    expr ::= When when_tail(A) [RPar] { let (x, y) = A; Ast::When(x, y) };
    expr ::= Case expr(A) when_tail(B) [RPar] { case_to_when(A, B) };
    expr ::= switch_head(A) expr(B) when_tail(C) [RPar] { switch_to_when(A, B, C) };
    expr ::= Word(A) [RPar] { Ast::Name(A) };
    expr ::= expr(A) Pow expr(B) { named_call("op_pow", vec![A, B]) };
    expr ::= Add expr(A) [Not] { A };
    expr ::= Sub expr(A) [Not] { named_call("op_neg", vec![A]) };
    expr ::= Not expr(A) { named_call("op_not", vec![A]) };
    expr ::= expr(A) Mul expr(B) { named_call("op_mul", vec![A, B]) };
    expr ::= expr(A) Div expr(B) { named_call("op_div", vec![A, B]) };
    expr ::= expr(A) Add expr(B) { named_call("op_add", vec![A, B]) };
    expr ::= expr(A) Sub expr(B) { named_call("op_sub", vec![A, B]) };
    expr ::= expr(A) Eqq expr(B) { named_call("op_eq", vec![A, B]) };
    expr ::= expr(A) Ne expr(B) { named_call("op_ne", vec![A, B]) };
    expr ::= expr(A) Le expr(B) { named_call("op_le", vec![A, B]) };
    expr ::= expr(A) Lt expr(B) { named_call("op_lt", vec![A, B]) };
    expr ::= expr(A) Ge expr(B) { named_call("op_ge", vec![A, B]) };
    expr ::= expr(A) Gt expr(B) { named_call("op_gt", vec![A, B]) };
    expr ::= expr(A) Andd expr(B) { named_call("op_and", vec![A, B]) };
    expr ::= expr(A) Orr expr(B) { named_call("op_or", vec![A, B]) };
}

fn retok(t: &Lexi) -> Result<Token, String> {
    Ok(match t {
        Lexi::TextLit(s) => Token::TextLit(s.to_string()),
        Lexi::RatLit(r) => Token::RatLit(r.clone()),
        Lexi::White => return Err("Should skip White".to_owned()),
        Lexi::Comment => return Err("Should skip Comment".to_owned()),
        Lexi::Line => return Err("Should filter Line".to_owned()),
        Lexi::Minf => Token::RatLit(BRat::minf()),
        Lexi::As => Token::As,
        Lexi::Break => Token::Break,
        Lexi::Case => Token::Case,
        Lexi::Each => Token::Each,
        Lexi::Else => Token::Else,
        Lexi::Function => Token::Function,
        Lexi::If => Token::If,
        Lexi::In => Token::In,
        Lexi::Inf => Token::RatLit(BRat::inf()),
        Lexi::Lazy => Token::Lazy,
        Lexi::Let => Token::Let,
        Lexi::List => Token::List,
        Lexi::Loop => Token::Loop,
        Lexi::Mutable => Token::Mutable,
        Lexi::My => Token::My,
        Lexi::Nan => Token::RatLit(BRat::nan()),
        Lexi::Return => Token::Return,
        Lexi::Std => Token::Std,
        Lexi::Switch => Token::Switch,
        Lexi::Sys => Token::Sys,
        Lexi::Update => Token::Update,
        Lexi::Use => Token::Use,
        Lexi::Usr => Token::Usr,
        Lexi::When => Token::When,
        Lexi::Word(s) => Token::Word(s.to_string()),
        Lexi::Dots3 => Token::Dots3,
        Lexi::Eqq => Token::Eqq,
        Lexi::Le => Token::Le,
        Lexi::Ge => Token::Ge,
        Lexi::Ne => Token::Ne,
        Lexi::Andd => Token::Andd,
        Lexi::Orr => Token::Orr,
        Lexi::Arrow => Token::Arrow,
        Lexi::LSq => Token::LSq,
        Lexi::RSq => Token::RSq,
        Lexi::LPar => Token::LPar,
        Lexi::RPar => Token::RPar,
        Lexi::LCurl => Token::LCurl,
        Lexi::RCurl => Token::RCurl,
        Lexi::Eq => Token::Eq,
        Lexi::Semi => Token::Semi,
        Lexi::Comma => Token::Comma,
        Lexi::Lt => Token::Lt,
        Lexi::Gt => Token::Gt,
        Lexi::Add => Token::Add,
        Lexi::Sub => Token::Sub,
        Lexi::Mul => Token::Mul,
        Lexi::Div => Token::Div,
        Lexi::Pow => Token::Pow,
        Lexi::Not => Token::Not,
        Lexi::Err => return Err("Should filter Err".to_owned()),
    })
}

pub fn parse_str_to_ast_file(s: &str, filepath: &str) -> Result<Ast, String> {
    let mut lex = Lexi::lexer(s);
    let mut lno = 1;
    let mut lch = 0usize;
    let mut psr = Parser::new();
    loop {
        match lex.next() {
            None => break,
            Some(Lexi::Line) => {
                lno += 1;
                lch = lex.span().end;
            },
            Some(Lexi::Err) => {
                let sp = lex.span();
                let cno = sp.end - lch;
                return Err(format!("{}:{}:..{}: error in '{}'", filepath, lno, cno, lex.slice()));
            },
            Some(t) => {
                let sp = lex.span();
                let cno = sp.end - lch;
                let tt = retok(&t);
                if let Err(e) = psr.parse(retok(&t)?) {
                    return Err(format!("{}:{}:..{} '{:?}' {:?} => {:?}: error: {:?}",
                        filepath, lno, cno, lex.slice(), &t, tt, e));
                }
            },
        }
    }
    match psr.end_of_input() {
        Err(e) => Err(format!("{}: End of input error: {:?}", filepath, e)),
        Ok(d) => Ok(d),
    }
}

pub fn parse_str_to_ast_stmt(s: &str, filepath: &str) -> Result<Ast, String> {
    match parse_str_to_ast_file(s, filepath)? {
        Ast::File(stmts) => {
            let n = stmts.len();
            if 1 != n {
                return Err(format!("Expected one statement, found {}", n));
            }
            Ok(stmts[0].clone())
        },
        _ => Err("Not a File".to_owned()),
    }
}

pub fn parse_str_to_ast_function_decl(s: &str, filepath: &str) -> Result<Ast, String> {
    let a = parse_str_to_ast_stmt(s, filepath)?;
    match a {
        Ast::FnDecl(..) => Ok(a),
        _ => Err("Not an FnDecl".to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(at: Result<Ast, String>, x: &str) {
        assert_eq!(format!("{}", at.unwrap()), x);
    }
    fn ps(s: &str, x: &str) { parsed(parse_str_to_ast_stmt(s, "_t"), x); }
    fn pf(s: &str, x: &str) { parsed(parse_str_to_ast_file(s, "_t"), x); }
    fn psx(s: &str) { ps(s, s); }

    #[test] fn rat1() { ps("1e11", "100000000000"); }
    #[test] fn add1() { ps("1+ 2", "op_add(1, 2)") }
    #[test] fn case1() { ps("case 1 { 1 => 2; 3 => { 4 }; }",
        "when { op_eq(1, 1) => {2}; op_eq(3, 1) => {4};} else => {}"); }
    #[test] fn sw1() { ps("switch < 10 { 5 => { 4 } }",
        "when { op_lt(5, 10) => {4};} else => {}"); }
    #[test] fn sw2() { ps("switch > 10 { 5 => { 4 } }",
        "when { op_gt(5, 10) => {4};} else => {}"); }
    #[test] fn sw3() { ps("switch <= 10 { 5 => { 4 } }",
        "when { op_le(5, 10) => {4};} else => {}"); }
    #[test] fn sw4() { ps("switch >= 10 { 5 => { 4 } }",
        "when { op_ge(5, 10) => {4};} else => {}"); }
    #[test] fn sw5() { ps("switch == 10 { 5 => { 4 } }",
        "when { op_eq(5, 10) => {4};} else => {}"); }
    #[test] fn sw6() { ps("switch <> 10 { 5 => { 4 } }",
        "when { op_ne(5, 10) => {4};} else => {}"); }
    #[test] fn sw7() { ps("switch (op_ne) 10 { 5 => { 4 } }",
        "when { op_ne(5, 10) => {4};} else => {}"); }
    #[test] fn pr1() { ps("1+ 2*3", "op_add(1, op_mul(2, 3))"); }
    #[test] fn pr2() { ps("++1- 2 /3", "op_sub(1, op_div(2, 3))"); }
    #[test] fn pr3() { ps("- 3^2", "op_neg(op_pow(3, 2))"); }
    #[test] fn pr4() { ps("1 >= 2 || !(3 <> 4) || 5 > 6",
        "op_or(op_or(op_ge(1, 2), op_not(op_ne(3, 4))), op_gt(5, 6))"); }
    #[test] fn pr5() { ps("1 == 2 && 3 <= 4 && 4 < 5",
        "op_and(op_and(op_eq(1, 2), op_le(3, 4)), op_lt(4, 5))"); }
    #[test] fn fdec1() { ps("function (lazy ...)", "function (lazy ...)"); }
    #[test] fn call1() { ps("(element(x, 2))(1)", "element(x, 2)(1)"); }
    #[test] fn file1() { pf("", "\n"); }
    #[test] fn file2() { pf("1;", "1\n"); }
    #[test] fn file3() { pf("a;b;c", "a; b; c\n"); }
    #[test] fn file4() { pf("1;add(2,3,)", "1; add(2, 3)\n"); }
    #[test] fn use1() { psx("use my home as yours"); }
    #[test] fn use2() { ps("use usr usb", "use usr usb as usb"); }
    #[test] fn let1() { psx("let x = lazy y"); }
    #[test] fn mut1() { psx("mutable x = lazy y"); }
    #[test] fn upd1() { psx("update x = lazy y"); }
}

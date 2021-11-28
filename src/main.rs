//! `rat` is an interpreter for a toy programming language which uses rational
//! numbers of unlimited precision.  See [README](README.html) for details.

/// parse tree (Ast)
mod ast;
/// built-in functions
mod bi;
/// big rationals, with nan and inf
mod brat;
/// executable tree
mod bst;
/// execution environment
mod cab;
/// lexer and parser: text to Ast
mod parse;
/// top-level
mod run;
/// display utilities
mod udisp;

use run::standard_main;

/// call a `rat` function from the command line
#[cfg(not(tarpaulin_include))] // input (command line), output (result)
fn main() -> Result<(), String> {
    standard_main()
}

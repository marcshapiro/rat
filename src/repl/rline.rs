// rat repl readline
use crate::cab::Cab;

use regex::Regex;

use rustyline::completion::Completer;
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{CompletionType, Config, Context, Editor, Helper};

use std::borrow::Cow::{self, Borrowed, Owned};
use std::collections::HashMap;

pub struct RLine {
    rl: rustyline::Editor<MyHelper>,
    hist_file: &'static str,
}

impl RLine {
    // set up and load history
    pub fn new() -> RLine {
        let config = Config::builder()
            .completion_type(CompletionType::List)
            .build();
        let mut rl = Editor::with_config(config);
        rl.set_helper(Some(MyHelper::new()));
        let hist_file = ".repl.hist.txt";
        if rl.load_history(hist_file).is_err() { }
        RLine { rl, hist_file, }
    }

    // update completion data and read line
    pub fn line(&mut self, cab: &Cab)
            -> Result<String, ()> { // Err() to exit loop
        if let Some(helper) = self.rl.helper_mut() {
            update_completion_data(helper, cab);
        }

        match self.rl.readline("rat> ") {
            Ok(line) => Ok(line),
            Err(ReadlineError::Interrupted) // ^C
            | Err(ReadlineError::Eof) => Err(()), // ^D
            Err(_) => Ok("".to_owned()),
        }
    }

    // save line to history
    pub fn mark(&mut self, line: &str) {
        self.rl.add_history_entry(line);
    }
}

fn update_completion_data(helper: &mut MyHelper, cab: &Cab) {
    let mut const_names = vec![];
    let mut mut_names = vec![];
    let mut usable = HashMap::new();
    if let Ok(vars) = cab.get_vars(&None) {
        for (name, (is_const, _)) in vars {
            if is_const {
                const_names.push(name);
            } else {
                mut_names.push(name);
            }
        }
    }
    if let Ok(used) = cab.get_used(&None) {
        for name in used.keys() {
            const_names.push(name.to_owned());
        }
    }
    if let Ok(cabable) = cab.get_usable(&None) {
        for (path, func) in cabable.keys() {
            if !usable.contains_key(path) {
                usable.insert(path.to_owned(), vec![]);
            }
            usable.get_mut(path).unwrap().push(func.to_owned());
        }
    };

    helper.const_names = const_names;
    helper.mut_names = mut_names;
    helper.usable = usable;
}

impl Completer for MyHelper {
    type Candidate = String;

    fn complete(
        &self,
        line_in: &str, // full line
        pos: usize, // cursor position
        _ctx: &Context<'_>, // history
    ) -> Result<(usize, // starting position of completion
            Vec<Self::Candidate>), // possible completions
            ReadlineError> {
        let line = &line_in[0..pos];

        let mut result = vec![];

        if let Some(cap) = self.re.cur_word.captures(line) {
            if let Some(mre) = cap.get(1) {
                let word = mre.as_str();
                push_static(&mut result, KEYWORDS, word);
                push_static(&mut result, PATHS, word);
                push_string(&mut result, &self.mut_names, word);
                push_string(&mut result, &self.const_names, word);
                push_values(&mut result, &self.usable, word);
            }
        }

        Ok((pos, result))
    }
}

fn push_values(res: &mut Vec<String>, map: &HashMap<String, Vec<String>>, word: &str) {
    let zw = word.len();
    for list in map.values() {
        for elt in list {
            if zw <= elt.len() && elt.starts_with(word) {
                res.push(elt[zw..].to_owned());
            }
        }
    }
}

fn push_string(res: &mut Vec<String>, list: &[String], word: &str) {
    let zw = word.len();
    for elt in list {
        if zw <= elt.len() && elt.starts_with(word) {
            res.push(elt[zw..].to_owned());
        }
    }
}

fn push_static(res: &mut Vec<String>, list: &[&str], word: &str) {
    let zw = word.len();
    for elt in list {
        if zw <= elt.len() && elt.starts_with(word) {
            res.push(elt[zw..].to_owned());
        }
    }
}

struct CompiledRegexes {
    cur_word: Regex,
}

impl CompiledRegexes {
    fn new() -> CompiledRegexes {
        let cur_word = Regex::new("(?:^|[^A-Za-z0-9_?])([A-Za-z0-9_?]+)$").unwrap();
        CompiledRegexes { cur_word }
    }
}

struct MyHelper {
    highlighter: MatchingBracketHighlighter,
    mut_names: Vec<String>, // TODO: context: after update
    const_names: Vec<String>, // let and use
    usable: HashMap<String, Vec<String>>, // path -> func // in 'use' context
    re: CompiledRegexes,
}
impl MyHelper {
    fn new() -> MyHelper {
        let highlighter = MatchingBracketHighlighter::new();
        let mut_names = vec![];
        let const_names = vec![];
        let usable = HashMap::new();
        let re = CompiledRegexes::new();
        MyHelper {
            highlighter,
            mut_names,
            const_names,
            usable,
            re,
        }
    }
}

// save history
impl Drop for RLine {
    fn drop(&mut self) {
        self.rl.save_history(self.hist_file).unwrap();
    }
}

const KEYWORDS: &[&str] = &[
    // "as", // TODO: context after 'use' 'path' 'func'
    "break", // TODO: context: after more '{' than '}'
    "case",
    "each",
    "else", // TODO: context: after 'if' ... '}'
    // "function" // not in repl
    "if",
    // "in" // TODO: context: after each 'name'
    "inf",
    // "lazy", // TODO: context: after 'let'|'mutable'|'update' '='
    "let",
    "list",
    "loop",
    "mutable",
    "nan",
    "return", // TODO: context: like break
    "switch",
    "update",
    "use", // TODO: context: at start or after ';'
    "when",
];

const PATHS: &[&str] = &[
    "my",
    "std",
    "sys",
    "usr",
];

impl Helper for MyHelper {}
impl Hinter for MyHelper { type Hint = String; }
impl Validator for MyHelper { }

impl Highlighter for MyHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Borrowed(prompt)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

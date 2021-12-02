// rat repl
mod rline;

use self::rline::RLine;
use crate::bst::Bst;
use crate::cab::Cab;

use std::time::Instant;

#[cfg(not(tarpaulin_include))] // input (and loads and output)
pub fn run_repl() {
    let mut rl = RLine::new();
    let mut cab = intro();
    while let Ok(line1) = rl.line(&cab) {
        let line2 = line1.trim();
        if line2.is_empty() { continue; }
        if "quit" == line2 { break };
        rl.mark(line2);
        eval_print(&mut cab, line2);
    }
    println!("Farewell.  The rat repl misses you already.");
}

const INTRO_SCRIPT: &str = "
use sys as_decimal;
use sys rat_version;
use sys show_usable;
use sys show_used;
use sys show_vars;
mutable repl_format = '';
mutable repl_digits = 20;
mutable repl_time = 0;
mutable ans = 0;
say('Welcome to the repl for', rat_version());
";

#[cfg(not(tarpaulin_include))] // output, loads
fn env_eval(cab: &mut Cab, t: &str, time: bool) -> Result<Bst, String> {
    // text to Bst + process any 'use', which may load files
    let t0 = Instant::now();
    let bf = Bst::from_str_file(t, "_repl")?;
    if bf.decl.is_some() {
        println!("'function' declaration ignored at repl");
    }
    for (path, func, name) in &bf.uses {
        if let Err(e) = cab.env_use(path, func, name) {
            println!("use {} {} as {}: error (ignored): {}",
                path, func, name, e);
        }
    }

    // eval Bst
    let t1 = Instant::now();
    let result = cab.vec_eval(&None, &bf.file);
    let t2 = Instant::now();

    // time even if error in eval
    if time {
        let rtime = def_eval(cab, "repl_time", false, |b|b.d_rat_bool("_r"));
        if rtime {
            let t_load = t1.duration_since(t0).as_secs_f64() * 1000.0;
            let t_run = t2.duration_since(t1).as_secs_f64() * 1000.0;
            println!("Time: {:>.3} msec load; {:>.3} msec run", t_load, t_run);
        }
    }

    // remove Break/Return
    let res = result?;
    Ok(match res {
        Bst::Break => Bst::zero(),
        Bst::Return(retv) => *retv,
        _ => res,
    })
}

#[cfg(not(tarpaulin_include))] // loads
fn intro() -> Cab {
    let mut cab = Cab::limo(true).unwrap();
    if let Err(e) = env_eval(&mut cab, INTRO_SCRIPT, false) {
        println!("Warning: normal repl startup failed: {}", e);
    }
    let start_fn = "my/start.repl";
    if let Ok(start_script) = std::fs::read_to_string(start_fn) {
        if let Err(e) = env_eval(&mut cab, &start_script, false) {
            println!("Warning: {} failed: {}", start_fn, e);
        }
    };

    cab
}

#[cfg(not(tarpaulin_include))] // output
fn eval_print(cab: &mut Cab, line: &str) {
    match env_eval(cab, line, true) {
        Err(e) => println!("Error: {}", e),
        Ok(res) => {
            update_ans(cab, &res);
            out_result(cab, &res);
        },
    }
}

fn def_eval<T>(cab: &mut Cab, text: &str, def: T, cvt: fn (Bst) -> Result<T, String>) -> T {
    match env_eval(cab, text, false) {
        Ok(b) => cvt(b).unwrap_or(def),
        Err(_) => def,
    }
}

fn update_ans(cab: &mut Cab, res: &Bst) {
    let upd = Bst::Let(false, false, true, "ans".to_owned(), Box::new(res.clone()));
    if let Err(e) = cab.eval(&None, &upd) {
        println!("Cannot update 'ans': {}", e);
    }
}

#[cfg(not(tarpaulin_include))] // output
fn out_result(cab: &mut Cab, res: &Bst) {
    let fmt = def_eval(cab, "repl_format", "".to_owned(), |b|b.d_list_text("_r"));
    match fmt.as_str() {
        "text" => match res.d_list_text("_out") {
            Ok(text) => { println!("'{}'", text); },
            Err(_) => { println!("{}",  res); },
        },
        "dec" => {
            if let Err(e) = env_eval(cab, "say(as_decimal(ans, repl_digits))", false) {
                println!("Cannot display with format 'dec': {}", e);
                println!("{}", res);
            }
        },
        "quiet" => {},
        _ => println!("{}", res),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn ans1() {
        let mut cab = Cab::taxi(false);
        let bdefine = Bst::from_str_stmt("mutable ans = 0", "_t").unwrap();
        cab.eval(&None, &bdefine).unwrap();
        let newval = Bst::from_str_stmt("5", "_t").unwrap();
        update_ans(&mut cab, &newval);
        let query = Bst::from_str_stmt("ans", "_t").unwrap();
        let res = cab.eval(&None, &query).unwrap();
        assert_eq!(format!("{}", res), "5");
    }
    #[test] fn qev1() {
        let mut cab = Cab::taxi(false);
        let def = 3usize;
        let res = def_eval(&mut cab, "3/2", def, |b| b.d_rat_usize("_t"));
        assert_eq!(res, def);
    }
    #[test] fn qev2() {
        let mut cab = Cab::taxi(false);
        let res = def_eval(&mut cab, "4", 3usize, |b| b.d_rat_usize("_t"));
        assert_eq!(res, 4usize);
    }
    #[test] fn qev3() {
        let mut cab = Cab::taxi(false);
        let def = 3usize;
        let res = def_eval(&mut cab, "@", def, |b| b.d_rat_usize("_t"));
        assert_eq!(res, def);
    }
}

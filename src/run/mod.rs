// run files for Rat
mod cmd;

use self::cmd::{process_arguments, Opt, OutFormat};
use crate::bi::register_bi;
use crate::brat::BRat;
use crate::bst::{Bst, BstFile};
use crate::cab::Cab;

use std::env;
use std::time::Instant;

#[cfg(not(tarpaulin_include))] // env, output
pub fn standard_main() -> Result<(), String> {
    let args = env::args().skip(1).collect();
    call_file_arg(args)
}

fn call_file_arg(args: Vec<String>) -> Result<(), String> {
    let opt = process_arguments(args)?;
    call_file(&opt, true, true)?;
    Ok(())
}

fn add_use_last_call(bf: &mut BstFile, path: &str) -> Result<(), String> {
    let n = bf.file.len();
    if 0 == n { return Err(format!("-{} requires non-empty script", path)); }
    match &bf.file[n-1] {
        Bst::Call(nm, _) => match &**nm {
            Bst::Name(name) => {
                bf.uses.push(build_use(path, name, name));
            },
            _ => return Err(format!("-{} requires script ending call to name", path)),
        },
        _ => return Err(format!("-{} requires script ending call", path)),
    }
    Ok(())
}

fn prep_call(bf: &BstFile, load_auto: bool, load_use: bool) -> Result<(Cab, Bst), String> {
    let mut cab = Cab::new();
    register_bi(&mut cab)?;
    if load_auto {
        cab.use_all_load("auto")?;
    }
    cab.load_file_rec("_main", "_main", bf, load_use)?;
    cab.postload()?;

    let ecall = build_call_func("_main", "_main", vec![]);
    Ok((cab, ecall))
}

fn build_use(path: &str, func: &str, name: &str) -> (String, String, String) {
    let nm = if name.is_empty() { name } else { func };
    (path.to_owned(), func.to_owned(), nm.to_owned())
}

fn build_call_func(path: &str, func: &str, args: Vec<Bst>) -> Bst {
    let efc = Bst::Func(path.to_owned(), func.to_owned());
    Bst::Call(Box::new(efc), args)
}

fn call_file(opt: &Opt, load_auto: bool, load_use: bool) -> Result<Bst, String> {
    let t0 = Instant::now();
    let mut bf = Bst::from_str_file(&opt.script, "_main")?;

    // process -sys -std -usr -my // convert to 'use'
    match &opt.path {
        None => {},
        Some(p) => add_use_last_call(&mut bf, p)?,
    }

    let (mut cab, ecall) = prep_call(&bf, load_auto, load_use)?;
    let t1 = Instant::now();

    // call wrapper
    let res = cab.eval(&None, &ecall)?;

    let t2 = Instant::now();
    if opt.time {
        let t_load = t1.duration_since(t0).as_secs_f64() * 1000.0;
        let t_run = t2.duration_since(t1).as_secs_f64() * 1000.0;
        println!("Time: {:>.3} msec load; {:>.3} msec run", t_load, t_run);
    }

    out_result(&res, &opt.fmt, load_auto, load_use)?;
    Ok(res)
}

fn out_result(res: &Bst, fmt: &OutFormat, load_auto: bool, load_use: bool) -> Result<(), String> {
    match fmt {
        OutFormat::General => { println!("{}", res); },
        OutFormat::None => {},
        OutFormat::Text => {
            match res.d_list_text("_out") {
                Ok(text) => { println!("'{}'", text); },
                Err(_) => { println!("{}",  res); },
            }
        },
        OutFormat::Decimal(sdigits) => match BRat::from_str(sdigits) {
            None => { println!("-dec: cannot parse '{}'; result is {}",  sdigits, res); },
            Some(ndigits) => {
                let deccall = build_call_func("sys", "as_decimal", vec![res.clone(), Bst::Rat(ndigits)]);
                let saycall = build_call_func("auto", "say", vec![deccall]);
                let uses = vec![build_use("sys", "as_decimal", "")];
                let outbf = BstFile { uses, decl: None, file: vec![saycall] };
                let (mut dcab, dcall) = prep_call(&outbf, load_auto, load_use)?;
                dcab.eval(&None, &dcall)?;
            },
        },
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // cfu() does a call_file without I/O, unless it is in the script
    // Only builtins are loaded.  'sys' builtins still need a 'use'.
    fn cfu(spath: &str, sscript: &str) -> Result<Bst, String> {
        let path = if spath.is_empty() {
            None
        } else {
            Some(spath.to_owned())
        };
        let fmt = OutFormat::None;
        let time = false;
        let script = sscript.to_owned();
        let opt = Opt { path, script, fmt, time };
        let load_auto = false;
        let load_use = false;
        call_file(&opt, load_auto, load_use)
    }
    fn cf(spath: &str, sscript: &str, xres: &str) {
        let r = cfu(spath, sscript).unwrap();
        assert_eq!(format!("{}", r), xres);
    }
    fn xf(spath: &str, sscript: &str, xerr: &str) {
        let e = cfu(spath, sscript).unwrap_err();
        assert_eq!(e, xerr);
    }

    #[test] fn cf1() { cf("", "1 + 2", "3"); }
    #[test] fn cf2() { cf("sys", "is_char(-1)", "0"); }
    #[test] fn cf3() { cf("", "", "0"); }
    #[test] fn xf1() { xf("sys", "let x = is_char(-1)",
        "-sys requires script ending call"); }
    #[test] fn xf2() { xf("usr", "", "-usr requires non-empty script"); }
}

#![crate_id(name = "iron",
            vers = "0.1",
            author = "Arcterus",
            license = "MPL v2.0")]

#![feature(macro_rules, globs, phase)]

#[phase(syntax, link)] extern crate log;
extern crate collections;
extern crate getopts;

use std::io;
use std::os;

mod interp;
mod ast;

static NAME: &'static str = "iron";
static VERSION: &'static str = "0.1";

fn main() {
	let args: Vec<StrBuf> = os::args().move_iter().map(|x| x.into_strbuf()).collect();
	let program = args.get(0).as_slice();

	let opts = [
		getopts::optflag("d", "debug", "debug mode"),
		getopts::optflag("", "ast", "print out the AST instead of interpreting the code"),
		getopts::optflag("V", "version", "print the version number"),
		getopts::optflag("h", "help", "print this help menu"),
	];

	let matches = match getopts::getopts(args.tail(), opts) {
		Ok(m) => m,
		Err(f) => {
			error!("{}", f.to_err_msg());
			os::set_exit_status(1);
			return
		}
	};

	if matches.opt_present("h") {
		help_menu(program, opts);
	} else if matches.opt_present("V") {
		version();
	} else if matches.free.len() == 0 {
		error!("REPL NYI");
		os::set_exit_status(1);
	} else {
		let mode =
			if matches.opt_present("d") {
				interp::Debug
			} else {
				interp::Release
			};
		let code = match io::File::open(&Path::new(matches.free.get(0).as_slice())) {
			Ok(mut file) => file.read_to_str().unwrap(),
			Err(f) => {
				error!("{}", f.to_str());
				os::set_exit_status(1);
				return
			}
		};
		let mut interp = interp::Interpreter::new();
		interp.set_mode(mode);
		//interp.load_code("(fn hi [param] (+ 1 param))".to_owned());
		//interp.load_code("(fn hi 1 \"hello world\" 1.05 '(1 2 3.0 4 3.4) [hi 2.354 0.1 \"hi\" (hi)])".to_owned());
		//interp.load_code("(println (add 2 3.4))".to_owned());
		interp.load_code(code);
		if matches.opt_present("ast") {
			interp.dump_ast();
		} else {
			println!("exit status: {}", interp.execute());
		}
	}
}

#[inline(always)]
fn version() {
	println!("{} v{}", NAME, VERSION);
}

#[inline(always)]
fn help_menu(program: &str, opts: &[getopts::OptGroup]) {
	version();
	println!("");
	println!("Usage:");
	println!("    {} [OPTIONS...] FILES...", program);
	println!("");
	print!("{}", getopts::usage("A simple, Lisp-based programming language written in Rust.", opts));
}

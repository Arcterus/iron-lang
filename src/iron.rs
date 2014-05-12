#![crate_id(name = "iron",
            vers = "0.1",
            author = "Arcterus",
            license = "MPL v2.0")]

#![feature(macro_rules, globs, phase)]

#[phase(syntax, link)] extern crate log;
extern crate getopts;

use std::os;

mod interp;
mod ast;

static NAME: &'static str = "iron";
static VERSION: &'static str = "0.1";

fn main() {
	let args = os::args();
	let program = args.get(0).clone();

	let opts = [
		getopts::optflag("d", "debug", "debug mode"),
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
	} else {
		let mode =
			if matches.opt_present("d") {
				interp::Debug
			} else {
				interp::Release
			};
		let mut interp = interp::Interpreter::new();
		interp.set_mode(mode);
		/*interp.load_code("(fn hi [param] (+ 1 param))".to_owned());*/
		interp.load_code("(fn hi 1)".to_owned());
		println!("exit status: {}", interp.execute());
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

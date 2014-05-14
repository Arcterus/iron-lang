use std::{i64, f64};

use collections;
use self::parser::Parser;
use ast::*;

mod parser;

#[deriving(Eq)]
pub enum InterpMode {
	Debug,
	Release
}

enum EnvValue {
	Code(fn(stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst),
	Value(ExprAst)
}

pub struct Interpreter<'a> {
	mode: InterpMode,
	parser: Parser,
	env: Environment<'a>,
	stack: Vec<ExprAst>
}

struct Environment<'a> {
	pub parent: Option<&'a Environment<'a>>,
	pub values: collections::HashMap<~str, EnvValue>
}

impl<'a> Interpreter<'a> {
	pub fn new() -> Interpreter<'a> {
		let mut env = Environment::new(None);
		env.populate_default();
		Interpreter {
			parser: Parser::new(),
			mode: Release,
			env: env,
			stack: vec!()
		}
	}

	pub fn set_mode(&mut self, mode: InterpMode) {
		self.mode = mode;
	}

	pub fn load_code(&mut self, code: ~str) {
		self.parser.load_code(code);
	}

	pub fn execute(&mut self) -> int {
		let mut root: Box<RootAst> = match self.parser.parse() { Root(ast) => ast, _ => unreachable!() };
		if self.mode != Debug {
			root = match root.optimize().unwrap() { Root(ast) => ast, _ => unreachable!() };
		}
		for ast in root.asts.iter() {
			self.execute_node(ast);
		}
		0 // exit status
	}

	fn execute_node(&mut self, node: &ExprAst) {
		match *node {
			Sexpr(ref ast) => {
				for subast in ast.operands.iter() {
					self.execute_node(subast);
				}
				match self.env.values.find(&ast.op.value) {
					Some(thing) => match *thing {
						Code(ref thunk) => {
							let val = (*thunk)(&mut self.stack, ast.operands.len());
							self.stack.push(val)
						}
						Value(_) => fail!("Not a thunk")  // XXX: fix
					},
					None => fail!("Could not find key")  // XXX: also fix
				}
			},
			ref other => self.stack.push(other.clone())  // XXX: probably can be fixed
		}

	}

	pub fn dump_ast(&mut self) {
		self.parser.parse().dump();
	}
}

impl<'a> Environment<'a> {
	pub fn new(parent: Option<&'a Environment<'a>>) -> Environment<'a> {
		Environment {
			parent: parent,
			values: collections::HashMap::new()
		}
	}

	pub fn populate_default(&mut self) {
		self.values.insert("add".to_owned(), Code(Environment::add));
		self.values.insert("print".to_owned(), Code(Environment::print));
		self.values.insert("println".to_owned(), Code(Environment::println));
	}

	fn add(stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		let mut ops = ops;
		let mut val = 0f64;
		let mut decimal = false;
		while ops > 0 {
			match unsafe { (*stack).pop() }.unwrap() {
				Integer(ref ast) => {
					val += ast.value as f64;
				}
				Float(ref ast) => {
					decimal = true;
					val += ast.value;
				}
				ref other => {
					fail!("NYI"); // XXX: implement obviously
				}
			}
			ops -= 1;
		}
		if decimal { Float(box FloatAst::new(val)) } else { Integer(box IntegerAst::new(val as i64)) }
	}

	fn print(stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		let mut ops = ops;
		while ops > 0 {
			match unsafe { (*stack).pop() }.unwrap() {
				Integer(ref ast) => i64::to_str_bytes(ast.value, 10, |v| print!("{}", v)),
				Float(ref ast) => print!("{}", f64::to_str(ast.value)),
				String(ref ast) => print!("{}", ast.string),
				ref other => fail!()  // XXX: more of the same
			}
			ops -= 1;
		}
		Integer(box IntegerAst::new(0))  // TODO: this should probably be result of output
	}

	fn println(stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		let val = Environment::print(stack, ops);
		println!("");
		val
	}
}

use std::{i64, f64};
use std::rc::Rc;
use std::vec::FromVec;

use collections;
use self::parser::Parser;
use ast::*;

mod parser;

#[deriving(Eq)]
pub enum InterpMode {
	Debug,
	Release
}

#[deriving(Clone)]
enum EnvValue {
	Code(fn(env: *mut Environment, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst),
	Value(ExprAst)
}

pub struct Interpreter {
	mode: InterpMode,
	parser: Parser,
	env: Environment,
	stack: Vec<ExprAst>
}

pub struct Environment {
	pub parent: Option<Rc<Environment>>,
	pub values: collections::HashMap<~str, EnvValue>
}

impl Interpreter {
	pub fn new() -> Interpreter {
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
			Interpreter::execute_node(&mut self.env, &mut self.stack, ast);
			self.stack.clear();
		}
		0 // exit status
	}

	pub fn execute_node(env: &mut Environment, stack: &mut Vec<ExprAst>, node: &ExprAst) {
		match *node {
			Sexpr(ref ast) => {
				let val: &str = ast.op.value;
				if val == "fn" {  // XXX: maybe refactor so this doesn't need to be treated specially?
					for subast in ast.operands.iter() {
						stack.push(subast.clone());
					}
				} else {
					for subast in ast.operands.iter() {
						Interpreter::execute_node(env, stack, subast);
					}
				}
				let thing = match env.values.find(&ast.op.value) {
					Some(thing) => (*thing).clone(),
					None => fail!("Could not find key")  // XXX: also fix
				};
				match thing {
					Code(thunk) => {
						let val = thunk(env as *mut Environment, stack as *mut Vec<ExprAst>, ast.operands.len());
						stack.push(val);
					}
					Value(ast) => match ast {
						super::ast::Code(ast) => {
							let mut count = 0;
							let mut params = vec!();
							for param in ast.params.items.iter() {
								let len = stack.len();
								match *param {
									Ident(ref idast) => {params.push(idast.value.clone());env.values.insert(idast.value.clone(), Value(stack.remove(len - ast.params.items.len() + count).unwrap()))},
									_ => fail!() // XXX: fix
								};
								count += 1;
							}
							for subast in ast.code.iter() {
								Interpreter::execute_node(env, stack, subast);
							}
							for param in params.iter() {  // TODO: remove in exchange for multiple Environments
								env.values.remove(param);
							}
						}
						_ => fail!("Not executable")  // XXX: fix
					}
				};
			}
			ref other => stack.push(other.clone())  // XXX: probably can be fixed
		}

	}

	pub fn dump_ast(&mut self) {
		self.parser.parse().dump();
	}
}

impl Environment {
	pub fn new(parent: Option<Rc<Environment>>) -> Environment {
		Environment {
			parent: parent,
			values: collections::HashMap::new()
		}
	}

	pub fn populate_default(&mut self) {
		self.values.insert("+".to_owned(), Code(Environment::add));
		self.values.insert("print".to_owned(), Code(Environment::print));
		self.values.insert("define".to_owned(), Code(Environment::define));
		self.values.insert("fn".to_owned(), Code(Environment::function));
	}

	fn add(env: *mut Environment, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
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
				Ident(ref ast) => {
					match unsafe { (*env).values.find(&ast.value) } {
						Some(thing) => match *thing {
							Value(ref ast) => {
								unsafe { (*stack).push(ast.clone()); }
								ops += 1;
							}
							_ => fail!("cannot add to a thunk")  // XXX: fix
						},
						None => fail!("could not find ident")  // XXX: fix
					}
				}
				ref other => {
					fail!("NYI"); // XXX: implement obviously
				}
			}
			ops -= 1;
		}
		if decimal { Float(box FloatAst::new(val)) } else { Integer(box IntegerAst::new(val as i64)) }
	}

	fn print(env: *mut Environment, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		let mut ops = ops;
		while ops > 0 {
			match unsafe { (*stack).remove((*stack).len() - ops) }.unwrap() {
				Integer(ref ast) => print!("{}", ast.value.to_str()),
				Float(ref ast) => print!("{}", f64::to_str(ast.value)),
				Ident(ref ast) => {
					match unsafe { (*env).values.find(&ast.value) } {
						Some(value) => match *value {
							Value(ref value) => {
								unsafe { (*stack).insert((*stack).len() + 1 - ops, value.clone()); }
								ops += 1;
							}
							Code(_) => fail!() // XXX: fix
						},
						None => fail!() // XXX: fix
					}
				}
				String(ref ast) => {
					let mut output = StrBuf::new();
					let mut escape = false;
					for ch in ast.string.chars() {
						if ch == '\\' {
							if escape {
								escape = false;
								output.push_char('\\');
							} else {
								escape = true;
							}
						} else if escape {
							match ch {
								'n' => println!("{}", output.to_owned()),
								't' => print!("{}\t", output.to_owned()),
								other => fail!("\\\\{} not a valid escape sequence", ch)  // XXX: fix
							}
							escape = false;
							output.truncate(0);
						} else {
							output.push_char(ch);
						}
					}
					if escape {
						fail!("unterminated escape sequence");  // XXX: fix
					}
					print!("{}", output.into_owned());
				},
				ref other => fail!()  // XXX: more of the same
			}
			ops -= 1;
		}
		Integer(box IntegerAst::new(0))  // TODO: this should probably be result of output
	}

	// should be able to take stuff like (define var value)
	fn define(env: *mut Environment, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		let ops = ops;
		if ops != 2 {
			fail!("define can only take two arguments");  // XXX: fix
		}
		let val = match unsafe { (*stack).pop() }.unwrap() {
			Sexpr(ast) => {
				Interpreter::execute_node(unsafe { ::std::mem::transmute(env) }, unsafe { ::std::mem::transmute(stack) }, &Sexpr(ast));
				Value(unsafe { (*stack).pop() }.unwrap())
			}
			other => Value(other)
		};
		let name = match unsafe { (*stack).pop() }.unwrap() {
			Ident(ref ast) => ast.value.clone(),
			_ => fail!("define must take ident for first argument")  // XXX: fix
		};
		// TODO: add checking in env to see if conflicting names
		unsafe { (*env).values.insert(name.clone(), val); }
		Ident(box IdentAst::new(name))
	}

	fn function(env: *mut Environment, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		let mut ops = ops;
		let mut code = vec!();
		if ops == 0 {
			fail!("fn need at least one argument");  // XXX: fix
		}
		let params = match unsafe { (*stack).remove((*stack).len() - ops) }.unwrap() {
			Array(ast) => *ast,
			_ => fail!() // XXX: fix
		};
		ops -= 1;
		while ops > 0 {
			unsafe { code.push((*stack).remove((*stack).len() - ops).unwrap()); }
			ops -= 1;
		}
		super::ast::Code(box CodeAst::new(params, FromVec::from_vec(code)))
	}
}

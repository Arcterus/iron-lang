use std::f64;
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
	pub parent: Option<*mut Environment>,
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
		debug!("execute");
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
		debug!("execute_node");
		match *node {
			Sexpr(ref sast) => {
				let val: &str = sast.op.value;
				if val == "fn" {  // XXX: maybe refactor so this doesn't need to be treated specially?
					for subast in sast.operands.iter() {
						stack.push(subast.clone());
					}
				} else if val == "if" {
					if sast.operands.len() > 0 {
						Interpreter::execute_node(env, stack, sast.operands.get(0).unwrap());
					}
					for subast in sast.operands.slice_from(1).iter() {
						stack.push(subast.clone());
					}
				} else {
					for subast in sast.operands.iter() {
						Interpreter::execute_node(env, stack, subast);
					}
				}
				let thing = match env.find(&sast.op.value) {
					Some(thing) => (*thing).clone(),
					None => fail!("Could not find key")  // XXX: also fix
				};
				match thing {
					Code(thunk) => {
						debug!("executing thunk...");
						let val = thunk(env as *mut Environment, stack as *mut Vec<ExprAst>, sast.operands.len());
						stack.push(val);
					}
					Value(ast) => match ast {
						super::ast::Code(ast) => {
							debug!("evaluating code...");
							let mut count = 0;
							let mut subenv = Environment::new(Some(env as *mut Environment));
							let len = sast.operands.len();
							let idx = stack.len() - len;
							for param in ast.params.items.iter() {
								match *param {
									Ident(ref idast) => {
										if idast.value.ends_with("...") {
											debug!("variadic param");
											let vec = Vec::from_fn(len - count, |_| stack.remove(idx).unwrap());
											subenv.values.insert(idast.value.slice_to(idast.value.len() - 3).to_owned(),
											                     Value(Array(box ArrayAst::new(FromVec::from_vec(vec)))));
										} else {
											debug!("normal param");
											subenv.values.insert(idast.value.clone(), Value(match stack.remove(idx).unwrap() {
												Ident(ast) => match env.find(&ast.value) {
													Some(val) => match val {
														&Value(ref val) => val.clone(),
														&Code(_) => fail!() // XXX: fix
													},
													None => fail!() // XXX: fix
												},
												other => other
											}));
										}
									}
									_ => fail!() // XXX: fix
								};
								count += 1;
							}
							for subast in ast.code.iter() {
								Interpreter::execute_node(&mut subenv, stack, subast);
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
	pub fn new(parent: Option<*mut Environment>) -> Environment {
		Environment {
			parent: parent,
			values: collections::HashMap::new()
		}
	}

	pub fn find<'a>(&'a self, key: &~str) -> Option<&'a EnvValue> {
		match self.values.find(key) {
			Some(m) => Some(m),
			None => match self.parent {
				Some(env) => unsafe { (*env).find(key) },
				None => None
			}
		}
	}

	pub fn populate_default(&mut self) {
		self.values.insert("+".to_owned(), Code(Environment::add));
		self.values.insert("=".to_owned(), Code(Environment::equal));
		self.values.insert("print".to_owned(), Code(Environment::print));
		self.values.insert("if".to_owned(), Code(Environment::ifexpr));
		self.values.insert("define".to_owned(), Code(Environment::define));
		self.values.insert("fn".to_owned(), Code(Environment::function));
		self.values.insert("get".to_owned(), Code(Environment::get));
		self.values.insert("len".to_owned(), Code(Environment::len));
	}

	fn add(env: *mut Environment, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		debug!("add");
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
					match unsafe { (*env).find(&ast.value) } {
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
				_ => {
					fail!("NYI"); // XXX: implement obviously
				}
			}
			ops -= 1;
		}
		if decimal { Float(box FloatAst::new(val)) } else { Integer(box IntegerAst::new(val as i64)) }
	}

	fn print(env: *mut Environment, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		debug!("print");
		let mut ops = ops;
		while ops > 0 {
			match unsafe { (*stack).remove((*stack).len() - ops) }.unwrap() {
				Integer(ref ast) => print!("{}", ast.value.to_str()),
				Float(ref ast) => print!("{}", f64::to_str(ast.value)),
				Ident(ref ast) => {
					match unsafe { (*env).find(&ast.value) } {
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
								other => fail!("\\\\{} not a valid escape sequence", other)  // XXX: fix
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
				Boolean(ast) => print!("{}", ast.value),
				_ => fail!()  // XXX: more of the same
			}
			ops -= 1;
		}
		Integer(box IntegerAst::new(0))  // TODO: this should probably be result of output
	}

	// should be able to take stuff like (define var value)
	fn define(env: *mut Environment, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		debug!("define");
		let ops = ops;
		if ops != 2 {
			fail!("define can only take two arguments");  // XXX: fix
		}
		let val = match unsafe { (*stack).pop() }.unwrap() {
			Sexpr(ast) => {
				Interpreter::execute_node(unsafe { ::std::mem::transmute(env) }, unsafe { ::std::mem::transmute(stack) }, &Sexpr(ast));
				Value(unsafe { (*stack).pop() }.unwrap())
			}
			Ident(ast) => match unsafe { (*env).find(&ast.value) } {
				Some(val) => match val {
					&Value(ref val) => Value(val.clone()),
					&Code(_) => fail!()  // XXX: fix
				},
				None => fail!()  // XXX: fix
			},
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

	fn function(_: *mut Environment, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		debug!("function");
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

	fn get(env: *mut Environment, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		debug!("get");
		if ops != 2 {
			fail!("get only takes two values (list/array and index)");  // XXX: fix
		}
		let arr = match unsafe { (*stack).remove((*stack).len() - 2) }.unwrap() {
			Array(ast) => *ast,
			Ident(ast) => match unsafe { (*env).find(&ast.value) } {
				Some(val) => match val {
					&Value(ref ast) => match ast {
						&Array(ref ast) => (**ast).clone(),
						_ => fail!()  // XXX: fix
					},
					&Code(_) => fail!()  // XXX: fix
				},
				None => fail!()  // XXX: fix
			},
			_ => fail!()  // XXX: fix
		};
		let idx = match unsafe { (*stack).pop() }.unwrap() {
			Integer(ast) => ast,
			Ident(ast) => match unsafe { (*env).find(&ast.value) } {
				Some(val) => match val {
					&Value(ref val) => match val {
						&Integer(ref ast) => (*ast).clone(),
						_ => fail!()  // XXX: fix
					},
					&Code(_) => fail!()  // XXX: fix
				},
				None => fail!()  // XXX: fix
			},
			_ => fail!()  // XXX: fix
		};
		let idx =
			if idx.value < 0 {
				fail!("cannot have negative index")  // XXX: fix
			} else {
				idx.value as uint
			};
		// TODO: check bounds
		arr.items.get(idx).unwrap().clone()
	}

	fn len(env: *mut Environment, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		debug!("len");
		if ops != 1 {
			fail!("get only takes one value (list/array)");  // XXX: fix
		}
		let arr = match unsafe { (*stack).pop() }.unwrap() {
			Array(ast) => *ast,
			Ident(ast) => match unsafe { (*env).find(&ast.value) } {
				Some(val) => match val {
					&Value(ref ast) => match ast {
						&Array(ref ast) => (**ast).clone(),
						_ => fail!()  // XXX: fix
					},
					&Code(_) => fail!()  // XXX: fix
				},
				None => fail!()  // XXX: fix
			},
			_ => fail!()  // XXX: fix
		};
		Integer(box IntegerAst::new(arr.items.len() as i64))
	}

	fn equal(_: *mut Environment, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		debug!("equal");
		let mut ops = ops;
		if ops < 2 {
			fail!("= needs at least two operands"); // XXX: fix
		}
		let cmpast = unsafe { (*stack).pop() }.unwrap();
		ops -= 1;
		while ops > 0 {
			if unsafe { (*stack).pop() }.unwrap() != cmpast {
				return Boolean(box BooleanAst::new(false));
			}
			ops -= 1;
		}
		Boolean(box BooleanAst::new(true))
	}

	fn ifexpr(env: *mut Environment, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		debug!("if");
		if ops < 2 || ops > 3 {
			fail!("if needs >= 2 && <= 4 operands");  // XXX: fix
		}
		let cond = match unsafe { (*stack).remove((*stack).len() - ops) }.unwrap() {
			Boolean(ast) => ast.value,
			_ => fail!() // XXX: fix
		};
		let ontrue = unsafe { (*stack).remove((*stack).len() - ops + 1) }.unwrap();
		if cond {
			Interpreter::execute_node(unsafe { ::std::mem::transmute(env) }, unsafe { ::std::mem::transmute(stack) }, &ontrue);
		}
		if ops - 2 > 0 {
			let onfalse = unsafe { (*stack).pop() }.unwrap();
			if !cond {
				Interpreter::execute_node(unsafe { ::std::mem::transmute(env) }, unsafe { ::std::mem::transmute(stack) }, &onfalse);
			}
		}
		unsafe { (*stack).pop() }.unwrap()
	}
}

#![allow(raw_pointer_deriving)]

use std::cell::RefCell;
use std::f64;
use std::io;
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

#[deriving(Clone, Eq)]
enum EnvValue {
	Code(fn(env: Rc<RefCell<Environment>>, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst),
	Value(ExprAst)
}

impl Eq for fn(env: Rc<RefCell<Environment>>, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
	fn eq(&self, other: &fn(env: Rc<RefCell<Environment>>, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst) -> bool {
		let other: *() = unsafe { ::std::mem::transmute(other) };
		let this: *() = unsafe { ::std::mem::transmute(self) };
		this == other
	}

	fn ne(&self, other: &fn(env: Rc<RefCell<Environment>>, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst) -> bool {
		!self.eq(other)
	}
}

pub struct Interpreter {
	mode: InterpMode,
	parser: Parser,
	pub env: Rc<RefCell<Environment>>,
	stack: Vec<ExprAst>
}

#[deriving(Clone, Eq)]
pub struct Environment {
	pub parent: Option<Rc<RefCell<Environment>>>,
	pub values: collections::HashMap<~str, EnvValue>
}

impl Interpreter {
	pub fn new() -> Interpreter {
		let mut env = Environment::new(None);
		env.populate_default();
		Interpreter {
			parser: Parser::new(),
			mode: Release,
			env: Rc::new(RefCell::new(env)),
			stack: vec!()
		}
	}

	pub fn set_mode(&mut self, mode: InterpMode) {
		self.mode = mode;
	}

	pub fn set_file(&mut self, file: ~str) {
		self.env.clone().borrow_mut().values.insert("FILE".to_owned(), Value(String(box StringAst::new(file))));
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
			Interpreter::execute_node(self.env.clone(), &mut self.stack, ast);
			self.stack.clear();
		}
		0 // exit status
	}

	pub fn execute_node(env: Rc<RefCell<Environment>>, stack: &mut Vec<ExprAst>, node: &ExprAst) {
		debug!("execute_node");
		let stacklen = stack.len();
		match *node {
			Sexpr(ref sast) => {
				let val: &str = sast.op.value;
				match val {
					"fn" => {
						for subast in sast.operands.iter() {
							stack.push(subast.clone());
						}
					}
					"if" => {
						if sast.operands.len() > 0 {
							Interpreter::execute_node(env.clone(), stack, sast.operands.get(0).unwrap());
						}
						for subast in sast.operands.slice_from(1).iter() {
							stack.push(subast.clone());
						}
					}
					"define" | "set" => {
						if sast.operands.len() > 0 {
							stack.push(sast.operands.get(0).unwrap().clone());
							for subast in sast.operands.slice_from(1).iter() {
								Interpreter::execute_node(env.clone(), stack, subast);
							}
						}
					}
					_ => {
						for subast in sast.operands.iter() {
							Interpreter::execute_node(env.clone(), stack, subast);
						}
					}
				};
				let thing = match env.borrow().find(&sast.op.value) {
					Some(thing) => thing,
					None => fail!("Could not find key")  // XXX: also fix
				};
				match thing {
					Code(thunk) => {
						debug!("executing thunk...");
						let val = thunk(env, stack as *mut Vec<ExprAst>, sast.operands.len());
						stack.push(val);
					}
					Value(ast) => match ast {
						super::ast::Code(ast) => {
							debug!("evaluating code...");
							let mut count = 0;
							let mut subenv = Environment::new(Some(ast.env.clone()));
							let mut len = sast.operands.len();
							if len > ast.params.items.len() {
								for _ in range(0, len - ast.params.items.len()) {
									stack.pop();
								}
								len = ast.params.items.len();
							}
							let idx = stack.len() - len;
							debug!("begin params");
							for param in ast.params.items.iter() {
								match *param {
									Ident(ref idast) => {
										debug!("\t{}", idast.value);
										if idast.value.ends_with("...") {
											let vec = Vec::from_fn(len - count, |_| stack.remove(idx).unwrap());
											subenv.values.insert(idast.value.slice_to(idast.value.len() - 3).to_owned(),
											                     Value(Array(box ArrayAst::new(FromVec::from_vec(vec)))));
										} else {
											subenv.values.insert(idast.value.clone(), Value(stack.remove(idx).unwrap()));
										}
									}
									_ => fail!() // XXX: fix
								};
								count += 1;
							}
							debug!("end params");
							let subenv = Rc::new(RefCell::new(subenv));
							for subast in ast.code.iter() {
								Interpreter::execute_node(subenv.clone(), stack, subast);
							}
						}
						_ => fail!("Not executable")  // XXX: fix
					}
				};
			}
			Ident(ref ast) => match env.borrow().find(&ast.value) {
				Some(val) => match val {
					Value(ref val) => stack.push(val.clone()),
					Code(_) => fail!()  // TODO: this should not actually fail
				},
				None => fail!("ident {} not declared", ast.value)
			},
			ref other => stack.push(other.clone())  // XXX: probably can be fixed
		}
		for _ in range(stacklen + 1, stack.len()) {
			let len = stack.len();
			stack.remove(len - 1);
		}
	}

	pub fn dump_ast(&mut self) {
		self.parser.parse().dump();
	}
}

impl Environment {
	pub fn new(parent: Option<Rc<RefCell<Environment>>>) -> Environment {
		Environment {
			parent: parent,
			values: collections::HashMap::new()
		}
	}

	pub fn find(&self, key: &~str) -> Option<EnvValue> {
		match self.values.find(key) {
			Some(m) => Some(m.clone()),
			None => match self.parent.clone() {
				Some(env) => (*env).clone().unwrap().find(key),
				None => None
			}
		}
	}

	pub fn replace(&mut self, key: ~str, value: EnvValue) -> bool {
		if self.values.contains_key(&key) {
			self.values.insert(key, value);
			true
		} else {
			match self.parent {
				Some(ref env) => env.borrow_mut().replace(key, value),
				None => false
			}
		}
	}

	pub fn populate_default(&mut self) {
		self.values.insert("FILE".to_owned(), Value(String(box StringAst::new("".to_owned()))));
		self.values.insert("+".to_owned(), Code(Environment::add));
		self.values.insert("=".to_owned(), Code(Environment::equal));
		self.values.insert("print".to_owned(), Code(Environment::print));
		self.values.insert("if".to_owned(), Code(Environment::ifexpr));
		self.values.insert("define".to_owned(), Code(Environment::define));
		self.values.insert("fn".to_owned(), Code(Environment::function));
		self.values.insert("get".to_owned(), Code(Environment::get));
		self.values.insert("set".to_owned(), Code(Environment::set));
		self.values.insert("len".to_owned(), Code(Environment::len));
		self.values.insert("import".to_owned(), Code(Environment::importexpr));
		self.values.insert("type".to_owned(), Code(Environment::type_obj));
	}

	fn add(_: Rc<RefCell<Environment>>, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
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
				_ => {
					fail!("NYI"); // XXX: implement obviously
				}
			}
			ops -= 1;
		}
		if decimal { Float(box FloatAst::new(val)) } else { Integer(box IntegerAst::new(val as i64)) }
	}

	fn print(_: Rc<RefCell<Environment>>, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		debug!("print");
		let mut ops = ops;
		while ops > 0 {
			match unsafe { (*stack).remove((*stack).len() - ops) }.unwrap() {
				Integer(ref ast) => print!("{}", ast.value.to_str()),
				Float(ref ast) => print!("{}", f64::to_str_digits(ast.value, 15)),
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
				Symbol(ast) => print!("'{}", ast.value),
				Boolean(ast) => print!("{}", ast.value),
				_ => fail!()  // XXX: more of the same
			}
			ops -= 1;
		}
		Integer(box IntegerAst::new(0))  // TODO: this should probably be result of output
	}

	// should be able to take stuff like (define var value)
	fn define(env: Rc<RefCell<Environment>>, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		debug!("define");
		let ops = ops;
		if ops != 2 {
			fail!("define can only take two arguments");  // XXX: fix
		}
		let valast = match unsafe { (*stack).pop() }.unwrap() {
			Sexpr(ast) => {
				Interpreter::execute_node(env.clone(), unsafe { ::std::mem::transmute(stack) }, &Sexpr(ast));
				unsafe { (*stack).pop() }.unwrap()
			}
			other => other
		};
		let name = match unsafe { (*stack).pop() }.unwrap() {
			Ident(ref ast) => ast.value.clone(),
			_ => fail!("define must take ident for first argument")  // XXX: fix
		};
		// TODO: add checking in env to see if conflicting names
		env.clone().borrow_mut().values.insert(name.clone(), Value(valast.clone()));
		valast
	}

	fn function(env: Rc<RefCell<Environment>>, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
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
		super::ast::Code(box CodeAst::new(params, FromVec::from_vec(code), env.clone()))
	}

	fn get(_: Rc<RefCell<Environment>>, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		debug!("get");
		if ops != 2 {
			fail!("get only takes two values (list/array and index)");  // XXX: fix
		}
		let arr = match unsafe { (*stack).remove((*stack).len() - 2) }.unwrap() {
			Array(ast) => *ast,
			_ => fail!()  // XXX: fix
		};
		let idx = match unsafe { (*stack).pop() }.unwrap() {
			Integer(ast) => ast,
			_ => fail!()  // XXX: fix
		};
		let idx =
			if idx.value < 0 {
				let arrlen = arr.items.len();
				if arrlen < -idx.value as uint {
					fail!("absolute value of {} is too large for the array/list", idx.value); // XXX: fix
				} else {
					arrlen + idx.value as uint
				}
			} else {
				idx.value as uint
			};
		// TODO: check bounds
		arr.items.get(idx).unwrap().clone()
	}

	fn set(env: Rc<RefCell<Environment>>, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		debug!("set");
		if ops != 3 {
			fail!("set only takes three values (list/array, index, value)");  // XXX: fix
		}
		let (idast, mut arrast) = match unsafe { (*stack).remove((*stack).len() - 3) }.unwrap() {
			Array(_) => return Nil(box NilAst::new()),
			Ident(ast) => match env.clone().borrow().find(&ast.value) {
				Some(val) => match val {
					Value(ref val) => match val {
						&Array(ref arrast) => (ast, arrast.clone()),
						_ => fail!() // XXX: fix
					},
					Code(_) => fail!() // XXX: fix
				},
				None => fail!() // XXX: fix
			},
			_ => fail!()  // XXX: fix
		};
		let idx = match unsafe { (*stack).remove((*stack).len() - 2) }.unwrap() {
			Integer(ast) => ast,
			_ => fail!()  // XXX: fix
		};
		let value = unsafe { (*stack).pop() }.unwrap();
		let idx =
			if idx.value < 0 {
				let arrlen = arrast.items.len();
				if arrlen < -idx.value as uint {
					fail!("absolute value of {} is too large for the array/list", idx.value); // XXX: fix
				} else {
					arrlen + idx.value as uint
				}
			} else {
				idx.value as uint
			};
		// TODO: fix this horrifically inefficient mess
		let mut vec: Vec<ExprAst> = arrast.items.clone().move_iter().collect();
		vec.grow_set(idx, &Nil(box NilAst::new()), value);
		arrast.items = FromVec::from_vec(vec);
		env.clone().borrow_mut().replace(idast.value, Value(Array(arrast)));
		Nil(box NilAst::new())
	}

	fn len(_: Rc<RefCell<Environment>>, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		debug!("len");
		if ops != 1 {
			fail!("get only takes one value (list/array)");  // XXX: fix
		}
		let arr = match unsafe { (*stack).pop() }.unwrap() {
			Array(ast) => *ast,
			_ => fail!()  // XXX: fix
		};
		Integer(box IntegerAst::new(arr.items.len() as i64))
	}

	fn equal(_: Rc<RefCell<Environment>>, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
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

	fn ifexpr(env: Rc<RefCell<Environment>>, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		debug!("if");
		if ops < 2 || ops > 3 {
			fail!("if needs >= 2 && <= 4 operands");  // XXX: fix
		}
		let cond = match unsafe { (*stack).remove((*stack).len() - ops) }.unwrap() {
			Boolean(ast) => ast.value,
			_ => fail!() // XXX: fix
		};
		let ontrue = unsafe { (*stack).remove((*stack).len() - ops + 1) }.unwrap();
		if ops - 2 > 0 {
			let onfalse = unsafe { (*stack).pop() }.unwrap();
			if !cond {
				Interpreter::execute_node(env.clone(), unsafe { ::std::mem::transmute(stack) }, &onfalse);
			}
		}
		if cond {
			Interpreter::execute_node(env.clone(), unsafe { ::std::mem::transmute(stack) }, &ontrue);
		}
		unsafe { (*stack).pop() }.unwrap()
	}

	fn importexpr(env: Rc<RefCell<Environment>>, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		let mut ops = ops;
		if ops == 0 {
			fail!("import requires at least one operand"); // XXX: fix
		}
		while ops > 0 {
			match unsafe { (*stack).remove((*stack).len() - ops) }.unwrap() {
				String(ast) => {
					let mut path = if ast.string.starts_with("./") || ast.string.starts_with("../") {
						Path::new(match env.clone().borrow().find(&"FILE".to_owned()).unwrap() {
							Value(val) => match val {
								String(ast) => ast.string,
								_ => fail!() // XXX: fix
							},
							Code(_) => fail!() // XXX: fix
						}).dir_path()
					} else {
						fail!();
						Path::new("MODULE DIRECTORY GOES HERE") // TODO: ...
					}.join(Path::new(ast.string.clone()));
					if !ast.string.ends_with(".irl") {
						path.set_extension("irl");
					}
					let code = match io::File::open(&path) {
						Ok(m) => m,
						Err(_) => fail!() // XXX: fix
					}.read_to_str().unwrap();
					let mut interp = Interpreter::new();
					interp.load_code(code);
					interp.set_file(path.as_str().unwrap().to_owned());
					interp.execute();
					env.borrow_mut().values.extend((*interp.env).clone().unwrap().values.move_iter());
				}
				_ => fail!() // XXX: fix
			}
			ops -= 1;
		}
		Nil(box NilAst::new())
	}

	fn type_obj(_: Rc<RefCell<Environment>>, stack: *mut Vec<ExprAst>, ops: uint) -> ExprAst {
		if ops != 1 {
			fail!("type only takes one object"); // XXX: fix
		}
		Symbol(box SymbolAst::new(match unsafe { (*stack).pop() }.unwrap() {
			Integer(_) => "integer",
			Float(_) => "float",
			Array(_) => "array",
			List(_) => "list",
			String(_) => "string",
			Symbol(_) => "symbol",
			super::ast::Code(_) => "code",
			Boolean(_) => "boolean",
			Nil(_) => "nil",
			_ => fail!() // XXX: fix
		}.to_owned()))
	}
}

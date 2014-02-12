use std::cast::transmute;
use std::vec;

pub enum AstKind {
	Root,
	Sexpr,
	String,
	List,
	Array,
	Pointer,
	Integer,
	Float
}

pub trait Ast {
	fn kind(&self) -> AstKind;
	fn optimize(~self) -> Option<~Ast>;
	//fn eval(&self) -> Option<~Any>;
	fn compile(&self) -> ~[u8];
}

pub struct RootAst {
	asts: ~[~Ast]
}

pub struct SexprAst {
	op: ~str,
	operands: ~[~Ast]
}

pub struct StringAst {
	string: ~str
}

pub struct ListAst {
	items: ~[~Ast]
}

pub struct ArrayAst {
	items: ~[~Ast]
}

pub struct PointerAst {
	pointee: ~Ast
}

pub struct IntegerAst {
	value: i64
}

pub struct FloatAst {
	value: f64
}

impl RootAst {
	pub fn new() -> RootAst {
		RootAst {
			asts: ~[]
		}
	}

	pub fn push(&mut self, ast: ~Ast) {
		self.asts.push(ast);
	}
}

impl Ast for RootAst {
	#[inline(always)]
	fn kind(&self) -> AstKind {
		Root
	}

	fn optimize(~self) -> Option<~Ast> {
		let mut result = RootAst::new();
		result.asts = self.asts.move_iter().filter_map(|ast| ast.optimize()).to_owned_vec();
		Some(~result as ~Ast)
	}

	fn compile(&self) -> ~[u8] {
		let mut result = ~[];
		for bc in self.asts.map(|ast| ast.compile()).iter() {
			result = vec::append(result, *bc);
		}
		result
	}
}

impl SexprAst {
	fn is_math_op(&self) -> bool {
		let op: &str = self.op;
		match op {
			"+" | "-" | "*" | "/" => true,
			_ => false
		}
	}
}

impl Ast for SexprAst {
	#[inline(always)]
	fn kind(&self) -> AstKind {
		Sexpr
	}

	fn optimize(~self) -> Option<~Ast> {
		if self.is_math_op() {
			// TODO: check if ops can be eliminated
		}
		Some(self as ~Ast)
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}
}

impl Ast for StringAst {
	#[inline(always)]
	fn kind(&self) -> AstKind {
		String
	}

	fn optimize(~self) -> Option<~Ast> {
		// TODO: perhaps this should deal with a string table?
		Some(self as ~Ast)
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}
}

impl Ast for ListAst {
	#[inline(always)]
	fn kind(&self) -> AstKind {
		List
	}

	fn optimize(~self) -> Option<~Ast> {
		Some(self as ~Ast)
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}
}

impl Ast for ArrayAst {
	#[inline(always)]
	fn kind(&self) -> AstKind {
		Array
	}

	fn optimize(~self) -> Option<~Ast> {
		Some(self as ~Ast)
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}
}

impl Ast for PointerAst {
	#[inline(always)]
	fn kind(&self) -> AstKind {
		Pointer
	}

	fn optimize(~self) -> Option<~Ast> {
		Some(self as ~Ast)
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}
}

impl IntegerAst {
	pub fn new(num: i64) -> IntegerAst {
		IntegerAst {
			value: num
		}
	}
}

impl Ast for IntegerAst {
	#[inline(always)]
	fn kind(&self) -> AstKind {
		Integer
	}

	fn optimize(~self) -> Option<~Ast> {
		Some(self as ~Ast)
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}
}

impl Ast for FloatAst {
	#[inline(always)]
	fn kind(&self) -> AstKind {
		Float
	}

	fn optimize(~self) -> Option<~Ast> {
		Some(self as ~Ast)
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}
}


use std::mem::transmute;
use std::vec;
use std::vec::FromVec;

pub enum AstKind {
	Root,
	Sexpr,
	String,
	List,
	Array,
	Pointer,
	Ident,
	Integer,
	Float
}

pub trait Ast {
	fn kind(&self) -> AstKind;
	fn optimize(~self) -> Option<Box<Ast>>;
	//fn eval(&self) -> Option<~Any>;
	fn compile(&self) -> ~[u8];
}

pub struct RootAst {
	asts: Vec<Box<Ast>>
}

pub struct SexprAst {
	op: IdentAst,
	operands: ~[Box<Ast>]
}

pub struct StringAst {
	string: ~str
}

pub struct ListAst {
	items: ~[Box<Ast>]
}

pub struct ArrayAst {
	items: ~[Box<Ast>]
}

pub struct PointerAst {
	pointee: Box<Ast>
}

pub struct IdentAst {
	value: ~str
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
			asts: vec!()
		}
	}

	pub fn push(&mut self, ast: Box<Ast>) {
		self.asts.push(ast);
	}
}

impl Ast for RootAst {
	#[inline(always)]
	fn kind(&self) -> AstKind {
		Root
	}

	fn optimize(~self) -> Option<Box<Ast>> {
		let mut result = RootAst::new();
		result.asts = self.asts.move_iter().filter_map(|ast| ast.optimize()).collect();
		Some(box result as Box<Ast>)
	}

	fn compile(&self) -> ~[u8] {
		let mut result = vec!();
		for ast in self.asts.iter() {
			result.push_all(ast.compile());
		}
		FromVec::from_vec(result)
	}
}

impl SexprAst {
	pub fn new(op: IdentAst, operands: ~[Box<Ast>]) -> SexprAst {
		SexprAst {
			op: op,
			operands: operands
		}
	}

	fn is_math_op(&self) -> bool {
		let op: &str = self.op.value;
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

	fn optimize(~self) -> Option<Box<Ast>> {
		if self.is_math_op() {
			// TODO: check if ops can be eliminated
		}
		Some(self as Box<Ast>)
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

	fn optimize(~self) -> Option<Box<Ast>> {
		// TODO: perhaps this should deal with a string table?
		Some(self as Box<Ast>)
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

	fn optimize(~self) -> Option<Box<Ast>> {
		Some(self as Box<Ast>)
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

	fn optimize(~self) -> Option<Box<Ast>> {
		Some(self as Box<Ast>)
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

	fn optimize(~self) -> Option<Box<Ast>> {
		Some(self as Box<Ast>)
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

	fn optimize(~self) -> Option<Box<Ast>> {
		Some(self as Box<Ast>)
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}
}

impl IdentAst {
	pub fn new(ident: ~str) -> IdentAst {
		IdentAst {
			value: ident
		}
	}
}

impl Ast for IdentAst {
	#[inline(always)]
	fn kind(&self) -> AstKind {
		Ident
	}

	fn optimize(~self) -> Option<Box<Ast>> {
		Some(self as Box<Ast>)
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

	fn optimize(~self) -> Option<Box<Ast>> {
		Some(self as Box<Ast>)
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}
}


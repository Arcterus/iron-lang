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
	fn optimize(&mut self) -> Option<~Ast>;
	//fn eval(&self) -> Option<~Any>;
	fn compile(&self) -> ~[u8];
}

pub struct RootAst {
	asts: ~[~Ast]
}

pub struct SexprAst<'a> {
	op: &'a str,
	operands: ~[~Ast]
}

pub struct StringAst<'a> {
	string: &'a str
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

	fn optimize(&mut self) -> Option<~Ast> {
		self.asts = self.asts.mut_iter().filter_map(|ast| ast.optimize()).to_owned_vec();
		Some(unsafe { transmute(self) })
	}

	fn compile(&self) -> ~[u8] {
		let mut result = ~[];
		for bc in self.asts.map(|ast| ast.compile()).iter() {
			result = vec::append(result, *bc);
		}
		result
	}
}

impl<'a> SexprAst<'a> {
	fn is_math_op(&self) -> bool {
		match self.op {
			"+" | "-" | "*" | "/" => true,
			_ => false
		}
	}
}

impl<'a> Ast for SexprAst<'a> {
	#[inline(always)]
	fn kind(&self) -> AstKind {
		Sexpr
	}

	fn optimize(&mut self) -> Option<~Ast> {
		if self.is_math_op() {
			// TODO: check if ops can be eliminated
		}
		Some(unsafe { transmute(self) })
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}
}

impl<'a> Ast for StringAst<'a> {
	#[inline(always)]
	fn kind(&self) -> AstKind {
		String
	}

	fn optimize(&mut self) -> Option<~Ast> {
		// TODO: perhaps this should deal with a string table?
		Some(unsafe { transmute(self) })
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

	fn optimize(&mut self) -> Option<~Ast> {
		Some(unsafe { transmute(self) })
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

	fn optimize(&mut self) -> Option<~Ast> {
		Some(unsafe { transmute(self) })
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

	fn optimize(&mut self) -> Option<~Ast> {
		Some(unsafe { transmute(self) })
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}
}

impl Ast for IntegerAst {
	#[inline(always)]
	fn kind(&self) -> AstKind {
		Integer
	}

	fn optimize(&mut self) -> Option<~Ast> {
		Some(unsafe { transmute(self) })
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

	fn optimize(&mut self) -> Option<~Ast> {
		Some(unsafe { transmute(self) })
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}
}


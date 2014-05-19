#![allow(dead_code)]  // the code it warns about is not actually dead, so...

use std::cell::RefCell;
use std::rc::Rc;
use std::vec::FromVec;

static INDENTATION: uint = 2;

#[deriving(Clone, Eq)]
pub enum ExprAst {
	Root(Box<RootAst>),
	Sexpr(Box<SexprAst>),
	String(Box<StringAst>),
	List(Box<ListAst>),
	Array(Box<ArrayAst>),
	Pointer(Box<PointerAst>),
	Ident(Box<IdentAst>),
	Symbol(Box<SymbolAst>),
	Integer(Box<IntegerAst>),
	Float(Box<FloatAst>),
	Boolean(Box<BooleanAst>),
	Nil(Box<NilAst>),
	Comment(Box<CommentAst>),
	Code(Box<CodeAst>)
}

pub trait Ast {
	fn optimize(&self) -> Option<ExprAst>;
	fn optimize_owned(~self) -> Option<ExprAst>;
	//fn eval(&self) -> Option<~Any>;
	fn compile(&self) -> ~[u8];

	fn dump(&self) { self.dump_level(0) }

	// XXX: this should in actuality be private...
	fn dump_level(&self, level: uint);
}

#[deriving(Clone, Eq)]
pub struct RootAst {
	pub asts: Vec<ExprAst>
}

#[deriving(Clone, Eq)]
pub struct SexprAst {
	pub op: IdentAst,
	pub operands: ~[ExprAst]
}

#[deriving(Clone, Eq)]
pub struct StringAst {
	pub string: ~str
}

#[deriving(Clone, Eq)]
pub struct ListAst {
	pub items: ~[ExprAst]
}

#[deriving(Clone, Eq)]
pub struct ArrayAst {
	pub items: ~[ExprAst]
}

#[deriving(Clone, Eq)]
pub struct PointerAst {
	pub pointee: ExprAst
}

#[deriving(Clone, Eq)]
pub struct IdentAst {
	pub value: ~str
}

#[deriving(Clone, Eq)]
pub struct SymbolAst {
	pub value: ~str
}

#[deriving(Clone, Eq)]
pub struct IntegerAst {
	pub value: i64
}

#[deriving(Clone, Eq)]
pub struct FloatAst {
	pub value: f64
}

#[deriving(Clone, Eq)]
pub struct BooleanAst {
	pub value: bool
}

#[deriving(Clone, Eq)]
pub struct NilAst;

#[deriving(Clone, Eq)]
pub struct CommentAst {
	pub value: ~str
}

#[deriving(Clone, Eq)]
pub struct CodeAst {
	pub params: ArrayAst,
	pub code: ~[ExprAst],
	pub env: Rc<RefCell<::interp::Environment>>
}

impl Ast for ExprAst {
	fn optimize(&self) -> Option<ExprAst> {
		let val = (*self).clone();
		(box val).optimize_owned()
	}

	fn optimize_owned(~self) -> Option<ExprAst> {
		match *self {
			Root(ast) => ast.optimize_owned(),
			Sexpr(ast) => ast.optimize_owned(),
			String(ast) => ast.optimize_owned(),
			List(ast) => ast.optimize_owned(),
			Array(ast) => ast.optimize_owned(),
			Pointer(ast) => ast.optimize_owned(),
			Ident(ast) => ast.optimize_owned(),
			Symbol(ast) => ast.optimize_owned(),
			Integer(ast) => ast.optimize_owned(),
			Float(ast) => ast.optimize_owned(),
			Boolean(ast) => ast.optimize_owned(),
			Nil(ast) => ast.optimize_owned(),
			Comment(ast) => ast.optimize_owned(),
			Code(ast) => ast.optimize_owned()
		}
	}

	fn compile(&self) -> ~[u8] {
		match *self {
			Root(ref ast) => ast.compile(),
			Sexpr(ref ast) => ast.compile(),
			String(ref ast) => ast.compile(),
			List(ref ast) => ast.compile(),
			Array(ref ast) => ast.compile(),
			Pointer(ref ast) => ast.compile(),
			Ident(ref ast) => ast.compile(),
			Symbol(ref ast) => ast.compile(),
			Integer(ref ast) => ast.compile(),
			Float(ref ast) => ast.compile(),
			Boolean(ref ast) => ast.compile(),
			Nil(ref ast) => ast.compile(),
			Comment(ref ast) => ast.compile(),
			Code(ref ast) => ast.compile()
		}
	}

	fn dump_level(&self, level: uint) {
		match *self {
			Root(ref ast) => ast.dump_level(level),
			Sexpr(ref ast) => ast.dump_level(level),
			String(ref ast) => ast.dump_level(level),
			List(ref ast) => ast.dump_level(level),
			Array(ref ast) => ast.dump_level(level),
			Pointer(ref ast) => ast.dump_level(level),
			Ident(ref ast) => ast.dump_level(level),
			Symbol(ref ast) => ast.dump_level(level),
			Integer(ref ast) => ast.dump_level(level),
			Float(ref ast) => ast.dump_level(level),
			Boolean(ref ast) => ast.dump_level(level),
			Nil(ref ast) => ast.dump_level(level),
			Comment(ref ast) => ast.dump_level(level),
			Code(ref ast) => ast.dump_level(level)
		}
	}
}

impl RootAst {
	pub fn new() -> RootAst {
		RootAst {
			asts: vec!()
		}
	}

	pub fn push(&mut self, ast: ExprAst) {
		self.asts.push(ast);
	}
}

impl Ast for RootAst {
	fn optimize(&self) -> Option<ExprAst> {
		let mut result = RootAst::new();
		result.asts = self.asts.iter().filter_map(|ast| ast.optimize()).collect();
		Some(Root(box result))
	}

	fn optimize_owned(~self) -> Option<ExprAst> {
		self.optimize()
	}

	fn compile(&self) -> ~[u8] {
		let mut result = vec!();
		for ast in self.asts.iter() {
			result.push_all(ast.compile());
		}
		FromVec::from_vec(result)
	}

	fn dump_level(&self, level: uint) {
		let mut spaces = StrBuf::new();
		for _ in range(0, level * INDENTATION) {
			spaces.push_char(' ');
		}
		let spaces = spaces.into_owned();
		println!("{}RootAst {}", spaces, "{");
		for ast in self.asts.iter() {
			ast.dump_level(level + 1);
		}
		println!("{}{}", spaces, "}");
	}
}

impl SexprAst {
	pub fn new(op: IdentAst, operands: ~[ExprAst]) -> SexprAst {
		SexprAst {
			op: op,
			operands: operands
		}
	}

	fn is_math_op(&self) -> bool {
		let op: &str = self.op.value;
		match op {
			"add" | "sub" | "mul" | "div" => true,
			_ => false
		}
	}
}

impl Ast for SexprAst {
	fn optimize(&self) -> Option<ExprAst> {
		let val = (*self).clone();
		(box val).optimize_owned()
	}

	fn optimize_owned(~self) -> Option<ExprAst> {
		if self.is_math_op() {
			// TODO: check if ops can be eliminated
		}
		Some(Sexpr(self))
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}

	fn dump_level(&self, level: uint) {
		let mut spaces = StrBuf::new();
		for _ in range(0, level * INDENTATION) {
			spaces.push_char(' ');
		}
		let spaces = spaces.into_owned();
		println!("{}SexprAst {}", spaces, "{");
		self.op.dump_level(level + 1);
		for ast in self.operands.iter() {
			ast.dump_level(level + 1);
		}
		println!("{}{}", spaces, "}");
	}
}

impl StringAst {
	pub fn new(value: ~str) -> StringAst {
		StringAst {
			string: value
		}
	}
}

impl Ast for StringAst {
	fn optimize(&self) -> Option<ExprAst> {
		let val = (*self).clone();
		(box val).optimize_owned()
	}

	fn optimize_owned(~self) -> Option<ExprAst> {
		// TODO: perhaps this should deal with a string table?
		Some(String(self))
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}

	fn dump_level(&self, level: uint) {
		let mut buf = StrBuf::new();
		for _ in range(0, INDENTATION) {
			buf.push_char(' ');
		}
		let indent = buf.to_owned();
		let spaces =
			if level == 0 {
				"".to_owned()
			} else {
				for _ in range(0, (level - 1) * INDENTATION) {
					buf.push_char(' ');
				}
				buf.into_owned()
			};
		println!("{}StringAst {}", spaces, "{");
		println!("{}{}\"{}\"", spaces, indent, self.string);
		println!("{}{}", spaces, "}");
	}
}

impl ListAst {
	pub fn new(items: ~[ExprAst]) -> ListAst {
		ListAst {
			items: items
		}
	}
}

impl Ast for ListAst {
	fn optimize(&self) -> Option<ExprAst> {
		let val = (*self).clone();
		(box val).optimize_owned()
	}

	fn optimize_owned(~self) -> Option<ExprAst> {
		Some(List(self))
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}

	fn dump_level(&self, level: uint) {
		let mut spaces = StrBuf::new();
		for _ in range(0, level * INDENTATION) {
			spaces.push_char(' ');
		}
		let spaces = spaces.into_owned();
		println!("{}ListAst {}", spaces, "{");
		for item in self.items.iter() {
			item.dump_level(level + 1);
		}
		println!("{}{}", spaces, "}");
	}
}

impl ArrayAst {
	pub fn new(items: ~[ExprAst]) -> ArrayAst {
		ArrayAst {
			items: items
		}
	}
}

impl Ast for ArrayAst {
	fn optimize(&self) -> Option<ExprAst> {
		let val = (*self).clone();
		(box val).optimize_owned()
	}

	fn optimize_owned(~self) -> Option<ExprAst> {
		Some(Array(self))
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}

	fn dump_level(&self, level: uint) {
		let mut spaces = StrBuf::new();
		for _ in range(0, level * INDENTATION) {
			spaces.push_char(' ');
		}
		let spaces = spaces.into_owned();
		println!("{}ArrayAst {}", spaces, "{");
		for item in self.items.iter() {
			item.dump_level(level + 1);
		}
		println!("{}{}", spaces, "}");
	}
}

impl Ast for PointerAst {
	fn optimize(&self) -> Option<ExprAst> {
		let val = (*self).clone();
		(box val).optimize_owned()
	}

	fn optimize_owned(~self) -> Option<ExprAst> {
		Some(Pointer(self))
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}

	fn dump_level(&self, _: uint) { }
}

impl IntegerAst {
	pub fn new(num: i64) -> IntegerAst {
		IntegerAst {
			value: num
		}
	}
}

impl Ast for IntegerAst {
	fn optimize(&self) -> Option<ExprAst> {
		let val = (*self).clone();
		(box val).optimize_owned()
	}

	fn optimize_owned(~self) -> Option<ExprAst> {
		Some(Integer(self))
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}

	fn dump_level(&self, level: uint) {
		let mut buf = StrBuf::new();
		for _ in range(0, INDENTATION) {
			buf.push_char(' ');
		}
		let indent = buf.to_owned();
		let spaces =
			if level == 0 {
				"".to_owned()
			} else {
				for _ in range(0, (level - 1) * INDENTATION) {
					buf.push_char(' ');
				}
				buf.into_owned()
			};
		println!("{}IntegerAst {}", spaces, "{");
		println!("{}{}{}", spaces, indent, self.value);
		println!("{}{}", spaces, "}");
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
	fn optimize(&self) -> Option<ExprAst> {
		let val = (*self).clone();
		(box val).optimize_owned()
	}

	fn optimize_owned(~self) -> Option<ExprAst> {
		Some(Ident(self))
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}

	fn dump_level(&self, level: uint) {
		let mut buf = StrBuf::new();
		for _ in range(0, INDENTATION) {
			buf.push_char(' ');
		}
		let indent = buf.to_owned();
		let spaces =
			if level == 0 {
				"".to_owned()
			} else {
				for _ in range(0, (level - 1) * INDENTATION) {
					buf.push_char(' ');
				}
				buf.into_owned()
			};
		println!("{}IdentAst {}", spaces, "{");
		println!("{}{}{}", spaces, indent, self.value);
		println!("{}{}", spaces, "}");
	}
}

impl SymbolAst {
	pub fn new(value: ~str) -> SymbolAst {
		SymbolAst {
			value: value
		}
	}
}

impl Ast for SymbolAst {
	fn optimize(&self) -> Option<ExprAst> {
		let val = (*self).clone();
		(box val).optimize_owned()
	}

	fn optimize_owned(~self) -> Option<ExprAst> {
		Some(Symbol(self))
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}

	fn dump_level(&self, level: uint) {
		let mut buf = StrBuf::new();
		for _ in range(0, INDENTATION) {
			buf.push_char(' ');
		}
		let indent = buf.to_owned();
		let spaces =
			if level == 0 {
				"".to_owned()
			} else {
				for _ in range(0, (level - 1) * INDENTATION) {
					buf.push_char(' ');
				}
				buf.into_owned()
			};
		println!("{}SymbolAst {}", spaces, "{");
		println!("{}{}{}", spaces, indent, self.value);
		println!("{}{}", spaces, "}");
	}
}

impl FloatAst {
	pub fn new(value: f64) -> FloatAst {
		FloatAst {
			value: value
		}
	}
}

impl Ast for FloatAst {
	fn optimize(&self) -> Option<ExprAst> {
		let val = (*self).clone();
		(box val).optimize_owned()
	}

	fn optimize_owned(~self) -> Option<ExprAst> {
		Some(Float(self))
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}

	fn dump_level(&self, level: uint) {
		let mut buf = StrBuf::new();
		for _ in range(0, INDENTATION) {
			buf.push_char(' ');
		}
		let indent = buf.to_owned();
		let spaces =
			if level == 0 {
				"".to_owned()
			} else {
				for _ in range(0, (level - 1) * INDENTATION) {
					buf.push_char(' ');
				}
				buf.into_owned()
			};
		println!("{}FloatAst {}", spaces, "{");
		println!("{}{}{}", spaces, indent, self.value);
		println!("{}{}", spaces, "}");
	}
}

impl BooleanAst {
	pub fn new(value: bool) -> BooleanAst {
		BooleanAst {
			value: value
		}
	}
}

impl Ast for BooleanAst {
	fn optimize(&self) -> Option<ExprAst> {
		let val = (*self).clone();
		(box val).optimize_owned()
	}

	fn optimize_owned(~self) -> Option<ExprAst> {
		Some(Boolean(self))
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}

	fn dump_level(&self, level: uint) {
		let mut buf = StrBuf::new();
		for _ in range(0, INDENTATION) {
			buf.push_char(' ');
		}
		let indent = buf.to_owned();
		let spaces =
			if level == 0 {
				"".to_owned()
			} else {
				for _ in range(0, (level - 1) * INDENTATION) {
					buf.push_char(' ');
				}
				buf.into_owned()
			};
		println!("{}BooleanAst {}", spaces, "{");
		println!("{}{}{}", spaces, indent, self.value);
		println!("{}{}", spaces, "}");
	}
}

impl NilAst {
	pub fn new() -> NilAst {
		NilAst
	}
}

impl Ast for NilAst {
	fn optimize(&self) -> Option<ExprAst> {
		let val = (*self).clone();
		(box val).optimize_owned()
	}

	fn optimize_owned(~self) -> Option<ExprAst> {
		Some(Nil(self))
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}

	fn dump_level(&self, level: uint) {
		let mut buf = StrBuf::new();
		for _ in range(0, level * INDENTATION) {
			buf.push_char(' ');
		}
		println!("{}NilAst", buf.into_owned());
	}
}

impl CommentAst {
	pub fn new(value: ~str) -> CommentAst {
		CommentAst {
			value: value
		}
	}
}

impl Ast for CommentAst {
	fn optimize(&self) -> Option<ExprAst> {
		let val = (*self).clone();
		(box val).optimize_owned()
	}

	fn optimize_owned(~self) -> Option<ExprAst> {
		Some(Comment(self))
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}

	fn dump_level(&self, level: uint) {
		let mut buf = StrBuf::new();
		for _ in range(0, INDENTATION) {
			buf.push_char(' ');
		}
		let indent = buf.to_owned();
		let spaces =
			if level == 0 {
				"".to_owned()
			} else {
				for _ in range(0, (level - 1) * INDENTATION) {
					buf.push_char(' ');
				}
				buf.into_owned()
			};
		println!("{}CommentAst {}", spaces, "{");
		println!("{}{}{}", spaces, indent, self.value);
		println!("{}{}", spaces, "}");
	}
}

impl CodeAst {
	pub fn new(params: ArrayAst, code: ~[ExprAst], env: Rc<RefCell<::interp::Environment>>) -> CodeAst {
		CodeAst {
			params: params,
			code: code,
			env: env
		}
	}
}

impl Ast for CodeAst {
	fn optimize(&self) -> Option<ExprAst> {
		let val = (*self).clone();
		(box val).optimize_owned()
	}

	fn optimize_owned(~self) -> Option<ExprAst> {
		Some(Code(self))
	}

	fn compile(&self) -> ~[u8] {
		~[]
	}

	fn dump_level(&self, _: uint) { }
}


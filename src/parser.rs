use ast::*;

mod ast;

macro_rules! parse_subexprs (
	($expfn:ident, $($others:ident),+) => (
		match self.$expfn() {
			Ok(m) => m,
			Err(f) => parse_subexprs!($($others)+)
		}
	);
	($expfn:ident) => (
		if_ok!(self.$expfn())
	)
)

pub struct Parser {
	code: ~str,
	pos: uint,
	line: uint,
	column: uint
}

pub struct ParseError<'a> {
	line: uint,
	column: uint,
	desc: &'a str
}

pub type ParseResult<'a, T> = Result<T, ParseError<'a>>;

impl<'a> ParseError<'a> {
	pub fn new(line: uint, col: uint, desc: &'a str) -> ParseError<'a> {
		ParseError {
			line: line,
			column: col,
			desc: desc
		}
	}
}

impl Parser {
	pub fn new(code: ~str) -> Parser {
		Parser {
			code: code,
			pos: 0,
			line: 1,
			column: 1
		}
	}

	pub fn parse(&mut self) -> ~Ast {
		let mut root = RootAst::new();
		while {
			let expr = match self.parse_expr() {
				Ok(m) => m,
				Err(f) => {
					error!("error at line {}, column {}: {}", f.line, f.column, f.desc);
					fail!(); // fix fail! later
				}
			};
			root.push(expr);
			if self.code.len() == 0 {
				false
			} else {
				true
			}
		} {}
		~root as ~Ast
	}

	fn parse_expr(&mut self) -> ParseResult<~Ast> {
		let expr = parse_subexprs!(parse_sexpr, parse_integer);
		Ok(expr)
	}

	fn parse_sexpr(&mut self) -> ParseResult<~Ast> {
		Err(ParseError::new(self.line, self.column, "not implemented"))
	}

	fn parse_integer(&mut self) -> ParseResult<~Ast> {
		Err(ParseError::new(self.line, self.column, "not implemented"))
	}
}

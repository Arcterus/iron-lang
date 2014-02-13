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

pub struct ParseError {
	line: uint,
	column: uint,
	desc: ~str
}

pub type ParseResult<T> = Result<T, ParseError>;

impl ParseError {
	pub fn new(line: uint, col: uint, desc: ~str) -> ParseError {
		ParseError {
			line: line,
			column: col,
			desc: desc
		}
	}
}

impl Parser {
	pub fn new() -> Parser {
		Parser {
			code: ~"",
			pos: 0,
			line: 1,
			column: 1
		}
	}

	pub fn load_code(&mut self, code: ~str) {
		self.code = code;
		self.pos = 0;
		self.line = 1;
		self.column = 1;
	}

	pub fn parse_code(&mut self, code: ~str) -> ~Ast {
		self.load_code(code);
		self.parse()
	}

	pub fn parse(&mut self) -> ~Ast {
		let mut root = RootAst::new();
		while self.code.len() > 0 {
			let expr = match self.parse_expr() {
				Ok(m) => m,
				Err(f) => {
					error!("error at line {}, column {}: {}", f.line, f.column, f.desc);
					fail!(); // fix fail! later
				}
			};
			root.push(expr);
		}
		~root as ~Ast
	}

	fn parse_expr(&mut self) -> ParseResult<~Ast> {
		let expr = parse_subexprs!(parse_sexpr, parse_integer);
		Ok(expr)
	}

	fn parse_sexpr(&mut self) -> ParseResult<~Ast> {
		Err(ParseError::new(self.line, self.column, ~"not implemented"))
	}

	fn parse_integer(&mut self) -> ParseResult<~Ast> {
		if self.pos == self.code.len() {
			return Err(ParseError::new(self.line, self.column, ~"end of file"));
		}
		let mut number = 0;
		let mut neg = false;
		let mut digits = 0;
		while {
			match self.code.char_at(self.pos) {
				num @ '0'..'9' => {
					digits += 1;
					number = number * 10 + num.to_digit(10).unwrap() as i64;
					true
				}
				'-' => {
					if digits == 0 {
						neg = true;
					} else {
						return Err(ParseError::new(self.line, self.column, ~"expected integer but found '-'"));
					}
					true
				}
				other => {
					if digits == 0 {
						return Err(ParseError::new(self.line, self.column,
							format!("expected integer but found '{:c}'", other)));
					}
					false
				}
			}
		} { self.pos += 1; if self.pos == self.code.len() { break } }
		Ok(~IntegerAst::new(if neg { -number } else { number }) as ~Ast)
	}

	#[inline(always)]
	fn add_line(&mut self) {
		self.line += 1;
		self.column = 1;
	}
}

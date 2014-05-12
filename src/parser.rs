use std::vec::FromVec;
use ast::*;

mod ast;

macro_rules! parse_subexprs (
	($expfn:ident, $($others:ident),+) => ({
		let oldpos = self.pos;
		let oldcol = self.column;
		let oldline = self.line;
		match self.$expfn() {
			Ok(m) => m,
			Err(f) => {
				self.pos = oldpos;
				self.column = oldcol;
				self.line = oldline;
				parse_subexprs!($($others),+)
			}
		}
	});
	($expfn:ident) => (
		try!(self.$expfn())
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
			code: "".to_owned(),
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

	pub fn parse_code(&mut self, code: ~str) -> Box<Ast> {
		self.load_code(code);
		self.parse()
	}

	pub fn parse(&mut self) -> Box<Ast> {
		let mut root = RootAst::new();
		self.skip_whitespace();
		while self.pos < self.code.len() {
			let expr = match self.parse_expr() {
				Ok(m) => m,
				Err(f) => {
					error!("error at line {}, column {}: {}", f.line, f.column, f.desc);
					fail!(); // fix fail! later
				}
			};
			root.push(expr);
			self.skip_whitespace();
		}
		box root as Box<Ast>
	}

	fn parse_expr(&mut self) -> ParseResult<Box<Ast>> {
		let expr = parse_subexprs!(parse_sexpr, parse_integer, parse_ident);
		Ok(expr)
	}

	fn parse_sexpr(&mut self) -> ParseResult<Box<Ast>> {
		self.skip_whitespace();
		if self.pos == self.code.len() {
			Err(ParseError::new(self.line, self.column, "end of file".to_owned()))
		} else if self.code.char_at(self.pos) == '(' {
			self.inc_pos_col();
			let op = try!(self.parse_ident_stack());
			let mut operands = vec!();
			loop {
				self.skip_whitespace();
				if self.pos == self.code.len() {
					return Err(ParseError::new(self.line, self.column, "end of file".to_owned()));
				}
				if self.code.char_at(self.pos) == ')' {
					self.inc_pos_col();
					break;
				}
				operands.push(try!(self.parse_expr()));
			}
			Ok(box SexprAst::new(op, FromVec::from_vec(operands)) as Box<Ast>)
		} else {
			Err(ParseError::new(self.line, self.column, format!("expected '(' but found {}", self.code.char_at(self.pos))))
		}
	}

	fn parse_integer(&mut self) -> ParseResult<Box<Ast>> {
		self.skip_whitespace();
		if self.pos == self.code.len() {
			return Err(ParseError::new(self.line, self.column, "end of file".to_owned()));
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
						return Err(ParseError::new(self.line, self.column, "expected integer but found '-'".to_owned()));
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
		} { self.inc_pos_col(); if self.pos == self.code.len() { break } }
		Ok(box IntegerAst::new(if neg { -number } else { number }) as Box<Ast>)
	}

	fn parse_ident(&mut self) -> ParseResult<Box<Ast>> {
		let val = try!(self.parse_ident_stack());
		Ok(box val as Box<Ast>)
	}

	fn parse_ident_stack(&mut self) -> ParseResult<IdentAst> {
		self.skip_whitespace();
		if self.pos == self.code.len() {
			Err(ParseError::new(self.line, self.column, "end of file".to_owned()))
		} else {
			let mut ident = StrBuf::new();
			loop {
				match self.code.char_at(self.pos) {
					ch @ 'a' .. 'z' | ch @ 'A' .. 'Z' | ch @ '_' => {
						ident.push_char(ch);
					}
					num @ '0' .. '9' => {
						if ident.len() > 0 {
							ident.push_char(num);
						} else {
							return Err(ParseError::new(self.line, self.column, format!("expected ident but found '{}'", num)));
						}
					}
					other => {
						if ident.len() > 0 {
							break
						} else {
							return Err(ParseError::new(self.line, self.column, format!("expected ident but found '{}'", other)));
						}
					}
				};
				self.inc_pos_col();
				if self.pos == self.code.len() {
					break;
				}
			}
			Ok(IdentAst::new(ident.into_owned()))
		}
	}

	#[inline(always)]
	fn skip_whitespace(&mut self) {
		while self.pos < self.code.len() && self.code.char_at(self.pos).is_whitespace() {
			if self.code.char_at(self.pos) == '\n' {
				self.add_line();
			} else {
				self.column += 1;
			}
			self.pos += 1;
		}
	}

	#[inline(always)]
	fn add_line(&mut self) {
		self.line += 1;
		self.column = 1;
	}

	#[inline(always)]
	fn inc_pos_col(&mut self) {
		self.column += 1;
		self.pos += 1;
	}
}

use std::num;
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
			Err(_) => {
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

	pub fn parse_code(&mut self, code: ~str) -> ExprAst {
		self.load_code(code);
		self.parse()
	}

	pub fn parse(&mut self) -> ExprAst {
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
		Root(box root)
	}

	fn parse_expr(&mut self) -> ParseResult<ExprAst> {
		let expr = parse_subexprs!(parse_sexpr, parse_float, parse_integer, parse_boolean, parse_ident, parse_string, parse_list, parse_array);
		Ok(expr)
	}

	fn parse_sexpr(&mut self) -> ParseResult<ExprAst> {
		self.skip_whitespace();
		if self.pos == self.code.len() {
			Err(self.eof_error())
		} else if self.code.char_at(self.pos) == '(' {
			self.inc_pos_col();
			let op = try!(self.parse_ident_stack());
			let mut operands = vec!();
			loop {
				self.skip_whitespace();
				if self.pos == self.code.len() {
					return Err(self.eof_error());
				}
				if self.code.char_at(self.pos) == ')' {
					self.inc_pos_col();
					break;
				}
				operands.push(try!(self.parse_expr()));
			}
			Ok(Sexpr(box SexprAst::new(op, FromVec::from_vec(operands))))
		} else {
			Err(self.unexpected_error("'('", format!("'{}'", self.code.char_at(self.pos))))
		}
	}

	fn parse_integer(&mut self) -> ParseResult<ExprAst> {
		Ok(Integer(box IntegerAst::new(try!(self.parse_integer_val()).val0())))
	}

	fn parse_integer_val(&mut self) -> ParseResult<(i64, uint)> {
		self.skip_whitespace();
		if self.pos == self.code.len() {
			return Err(self.eof_error());
		}
		let neg =
			if self.code.char_at(self.pos) == '-' {
				self.inc_pos_col();
				true
			} else {
				false
			};
		let mut number = 0;
		let mut digits = 0;
		while self.pos < self.code.len() && self.code.char_at(self.pos).is_digit() {
			digits += 1;
			number = number * 10 + self.code.char_at(self.pos).to_digit(10).unwrap() as i64;
			self.inc_pos_col();
		}
		if digits == 0 {
			Err(self.unexpected_error("integer", format!("'{}'", self.code.char_at(self.pos))))
		} else {
			Ok((if neg { -number } else { number }, digits))
		}
	}

	fn parse_float(&mut self) -> ParseResult<ExprAst> {
		let front = try!(self.parse_integer_val()).val0();
		if self.pos + 1 >= self.code.len() {
			Err(self.eof_error())
		} else if self.code.char_at(self.pos) != '.' {
			Err(self.unexpected_error("'.'", format!("'{}'", self.code.char_at(self.pos))))
		} else {
			self.inc_pos_col();
			if !self.code.char_at(self.pos).is_digit() {
				Err(self.unexpected_error("float", format!("'{}'", self.code.char_at(self.pos))))
			} else {
				let back = try!(self.parse_integer_val());
				Ok(Float(box FloatAst::new(front as f64 + back.val0() as f64 / num::pow(10, back.val1()) as f64)))
			}
		}
	}

	fn parse_array(&mut self) -> ParseResult<ExprAst> {
		self.skip_whitespace();
		if self.pos + 1 >= self.code.len() {
			Err(self.eof_error())
		} else if self.code.char_at(self.pos) == '[' {
			self.inc_pos_col();
			let mut items = vec!();
			loop {
				self.skip_whitespace();
				if self.pos == self.code.len() {
					return Err(self.eof_error());
				}
				if self.code.char_at(self.pos) == ']' {
					self.inc_pos_col();
					break;
				}
				items.push(try!(self.parse_expr()));
			}
			Ok(Array(box ArrayAst::new(FromVec::from_vec(items))))
		} else {
			Err(self.unexpected_error("'['", format!("'{}'", self.code.char_at(self.pos))))
		}
	}

	fn parse_list(&mut self) -> ParseResult<ExprAst> {
		self.skip_whitespace();
		if self.pos + 2 >= self.code.len() {
			Err(self.eof_error())
		} else if self.code.char_at(self.pos) == '\'' {
			self.inc_pos_col();
			if self.code.char_at(self.pos) == '(' {
				self.inc_pos_col();
				let mut items = vec!();
				loop {
					self.skip_whitespace();
					if self.pos == self.code.len() {
						return Err(self.eof_error());
					}
					if self.code.char_at(self.pos) == ')' {
						self.inc_pos_col();
						break;
					}
					items.push(try!(self.parse_expr()));
				}
				Ok(List(box ListAst::new(FromVec::from_vec(items))))
			} else {
				Err(self.unexpected_error("'('", format!("'{}'", self.code.char_at(self.pos))))
			}
		} else {
			Err(self.unexpected_error("'''", format!("'{}'", self.code.char_at(self.pos))))
		}
	}

	fn parse_ident(&mut self) -> ParseResult<ExprAst> {
		let val = try!(self.parse_ident_stack());
		Ok(Ident(box val))
	}

	fn parse_ident_stack(&mut self) -> ParseResult<IdentAst> {
		self.skip_whitespace();
		if self.pos == self.code.len() {
			Err(self.eof_error())
		} else {
			let mut ident = StrBuf::new();
			loop {
				let ch = self.code.char_at(self.pos);
				if ch.is_digit() || ch.is_whitespace() || ch == '(' || ch == ')' || ch == '[' || ch == ']' || ch == '\'' || ch == '"' {
					break;
				}
				ident.push_char(ch);
				self.inc_pos_col();
				if self.pos == self.code.len() {
					break;
				}
			}
			if ident.len() == 0 {
				if self.pos == self.code.len() {
					Err(self.eof_error())
				} else {
					Err(self.unexpected_error("ident", format!("'{}'", self.code.char_at(self.pos))))
				}
			} else {
				Ok(IdentAst::new(ident.into_owned()))
			}
		}
	}

	fn parse_string(&mut self) -> ParseResult<ExprAst> {
		self.skip_whitespace();
		if self.pos == self.code.len() {
			Err(self.eof_error())
		} else if self.code.char_at(self.pos) == '"' {
			self.inc_pos_col();
			let mut buf = StrBuf::new();
			while self.pos < self.code.len() && (self.code.char_at(self.pos) != '"' || self.code.char_at(self.pos - 1) == '\\') {
				buf.push_char(self.code.char_at(self.pos));
				if self.code.char_at(self.pos) == '\n' {
					self.add_line();
				} else {
					self.column += 1;
				}
				self.pos += 1;
			}
			if self.pos == self.code.len() {
				Err(self.eof_error())
			} else {
				self.inc_pos_col();
				Ok(String(box StringAst::new(buf.into_owned())))
			}
		} else {
			Err(self.unexpected_error("\"", format!("'{}'", self.code.char_at(self.pos))))
		}
	}

	fn parse_boolean(&mut self) -> ParseResult<ExprAst> {
		self.skip_whitespace();
		if self.pos == self.code.len() {
			Err(self.eof_error())
		} else {
			let mut buf = StrBuf::new();
			while self.pos < self.code.len() && self.code.char_at(self.pos).is_alphabetic() {
				buf.push_char(self.code.char_at(self.pos));
				self.inc_pos_col();
			}
			let string: &str = buf.into_owned();
			match string {
				"true" => Ok(Boolean(box BooleanAst::new(true))),
				"false" => Ok(Boolean(box BooleanAst::new(false))),
				other => Err(self.unexpected_error("\"true\" or \"false\"", format!("\"{}\"", other)))
			}
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

	#[inline(always)]
	fn eof_error(&self) -> ParseError {
		ParseError::new(self.line, self.column, "end of file".to_owned())
	}

	#[inline(always)]
	fn nyi_error(&self, item: &str) -> ParseError {
		ParseError::new(self.line, self.column, format!("{} not yet implemented", item))
	}

	#[inline(always)]
	fn unexpected_error(&self, expect: &str, found: &str) -> ParseError {
		ParseError::new(self.line, self.column, format!("expected {} but found {}", expect, found))
	}
}

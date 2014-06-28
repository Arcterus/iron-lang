use std::num;
use ast::*;

macro_rules! parse_subexprs (
	($expfn:ident, $($others:ident),+) => ({
		let oldpos = self.pos;
		let oldcol = self.column;
		let oldline = self.line;
		match self.$expfn() {
			Ok(m) => m,
			Err(f) => {
				let mut largeval = self.pos - oldpos;
				let mut largest = f;
				parse_subexprs!(S largest, largeval, oldpos, oldcol, oldline, $($others),+)
			}
		}
	});
	(S $largest:ident, $largeval:ident, $oldpos:ident, $oldcol:ident, $oldline:ident, $expfn:ident, $($others:ident),+) => ({
		self.pos = $oldpos;
		self.column = $oldcol;
		self.line = $oldline;
		match self.$expfn() {
			Ok(m) => m,
			Err(f) => {
				let ldiff = self.pos - $oldpos;
				if ldiff > $largeval {
					$largeval = ldiff;
					$largest = f;
				}
				parse_subexprs!(S $largest, $largeval, $oldpos, $oldcol, $oldline, $($others),+)
			}
		}
	});
	(S $largest:ident, $largeval:ident, $oldpos:ident, $oldcol:ident, $oldline:ident, $expfn:ident) => ({
		match self.$expfn() {
			Ok(m) => m,
			Err(f) =>
				return Err(if self.pos - $oldpos > $largeval {
					f
				} else {
					$largest
				})
		}
	})
)

pub struct Parser {
	code: String,
	pos: uint,
	line: uint,
	column: uint
}

pub struct ParseError {
	line: uint,
	column: uint,
	desc: String
}

pub type ParseResult<T> = Result<T, ParseError>;

impl ParseError {
	pub fn new(line: uint, col: uint, desc: String) -> ParseError {
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
			code: "".to_string(),
			pos: 0,
			line: 1,
			column: 1
		}
	}

	pub fn load_code(&mut self, code: String) {
		self.code = code;
		self.pos = 0;
		self.line = 1;
		self.column = 1;
	}

	pub fn parse_code(&mut self, code: String) -> ExprAst {
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
		let expr = parse_subexprs!(parse_sexpr, parse_float, parse_integer, parse_boolean, parse_nil, parse_ident, parse_string, parse_symbol, parse_list, parse_array, parse_comment);
		Ok(expr)
	}

	fn parse_sexpr(&mut self) -> ParseResult<ExprAst> {
		let code: &mut str = unsafe { ::std::mem::transmute(self.code.as_slice()) };
		self.skip_whitespace();
		if self.pos == code.len() {
			Err(self.eof_error())
		} else if code.char_at(self.pos) == '(' {
			self.inc_pos_col();
			let op = try!(self.parse_ident_stack());
			let mut operands = vec!();
			loop {
				self.skip_whitespace();
				if self.pos == code.len() {
					return Err(self.eof_error());
				}
				if code.char_at(self.pos) == ')' {
					self.inc_pos_col();
					break;
				}
				operands.push(try!(self.parse_expr()));
			}
			Ok(Sexpr(box SexprAst::new(op, operands)))
		} else {
			Err(self.unexpected_error("'('", format!("'{}'", code.char_at(self.pos))))
		}
	}

	fn parse_integer(&mut self) -> ParseResult<ExprAst> {
		Ok(Integer(box IntegerAst::new(try!(self.parse_integer_val()).val0())))
	}

	fn parse_integer_val(&mut self) -> ParseResult<(i64, uint)> {
		let code: &mut str = unsafe { ::std::mem::transmute(self.code.as_slice()) };
		self.skip_whitespace();
		if self.pos == code.len() {
			return Err(self.eof_error());
		}
		let neg =
			if code.char_at(self.pos) == '-' {
				self.inc_pos_col();
				true
			} else {
				false
			};
		let mut number = 0;
		let mut digits = 0;
		while self.pos < code.len() && code.char_at(self.pos).is_digit() {
			digits += 1;
			number = number * 10 + code.char_at(self.pos).to_digit(10).unwrap() as i64;
			self.inc_pos_col();
		}
		if digits == 0 {
			Err(self.unexpected_error("integer", format!("'{}'", code.char_at(self.pos))))
		} else {
			Ok((if neg { -number } else { number }, digits))
		}
	}

	fn parse_float(&mut self) -> ParseResult<ExprAst> {
		let code: &mut str = unsafe { ::std::mem::transmute(self.code.as_slice()) };
		let front = try!(self.parse_integer_val()).val0();
		if self.pos + 1 >= code.len() {
			Err(self.eof_error())
		} else if code.char_at(self.pos) != '.' {
			Err(self.unexpected_error("'.'", format!("'{}'", code.char_at(self.pos))))
		} else {
			self.inc_pos_col();
			if !code.char_at(self.pos).is_digit() {
				Err(self.unexpected_error("float", format!("'{}'", code.char_at(self.pos))))
			} else {
				let back = try!(self.parse_integer_val());
				Ok(Float(box FloatAst::new(front as f64 + back.val0() as f64 / num::pow(10u, back.val1()) as f64)))
			}
		}
	}

	fn parse_array(&mut self) -> ParseResult<ExprAst> {
		let code: &mut str = unsafe { ::std::mem::transmute(self.code.as_slice()) };
		self.skip_whitespace();
		if self.pos + 1 >= code.len() {
			Err(self.eof_error())
		} else if code.char_at(self.pos) == '[' {
			self.inc_pos_col();
			let mut items = vec!();
			loop {
				self.skip_whitespace();
				if self.pos == code.len() {
					return Err(self.eof_error());
				}
				if code.char_at(self.pos) == ']' {
					self.inc_pos_col();
					break;
				}
				items.push(try!(self.parse_expr()));
			}
			Ok(Array(box ArrayAst::new(items)))
		} else {
			Err(self.unexpected_error("'['", format!("'{}'", code.char_at(self.pos))))
		}
	}

	fn parse_list(&mut self) -> ParseResult<ExprAst> {
		let code: &mut str = unsafe { ::std::mem::transmute(self.code.as_slice()) };
		self.skip_whitespace();
		if self.pos + 2 >= code.len() {
			Err(self.eof_error())
		} else if code.char_at(self.pos) == '\'' {
			self.inc_pos_col();
			if code.char_at(self.pos) == '(' {
				self.inc_pos_col();
				let mut items = vec!();
				loop {
					self.skip_whitespace();
					if self.pos == code.len() {
						return Err(self.eof_error());
					}
					if code.char_at(self.pos) == ')' {
						self.inc_pos_col();
						break;
					}
					items.push(try!(self.parse_expr()));
				}
				Ok(List(box ListAst::new(items)))
			} else {
				Err(self.unexpected_error("'('", format!("'{}'", code.char_at(self.pos))))
			}
		} else {
			Err(self.unexpected_error("'''", format!("'{}'", code.char_at(self.pos))))
		}
	}

	fn parse_ident(&mut self) -> ParseResult<ExprAst> {
		let val = try!(self.parse_ident_stack());
		Ok(Ident(box val))
	}

	fn parse_ident_stack(&mut self) -> ParseResult<IdentAst> {
		let code: &mut str = unsafe { ::std::mem::transmute(self.code.as_slice()) };
		self.skip_whitespace();
		if self.pos == code.len() {
			Err(self.eof_error())
		} else {
			let mut ident = String::new();
			loop {
				let ch = code.char_at(self.pos);
				if !self.is_ident_char(ch) {
					break;
				}
				ident.push_char(ch);
				self.inc_pos_col();
				if self.pos == code.len() {
					break;
				}
			}
			if ident.len() == 0 {
				if self.pos == code.len() {
					Err(self.eof_error())
				} else {
					Err(self.unexpected_error("ident", format!("'{}'", code.char_at(self.pos))))
				}
			} else {
				Ok(IdentAst::new(ident))
			}
		}
	}

	fn parse_string(&mut self) -> ParseResult<ExprAst> {
		let code: &mut str = unsafe { ::std::mem::transmute(self.code.as_slice()) };
		self.skip_whitespace();
		if self.pos == code.len() {
			Err(self.eof_error())
		} else if code.char_at(self.pos) == '"' {
			self.inc_pos_col();
			let mut buf = String::new();
			while self.pos < code.len() && (code.char_at(self.pos) != '"' || code.char_at(self.pos - 1) == '\\') {
				buf.push_char(code.char_at(self.pos));
				if code.char_at(self.pos) == '\n' {
					self.add_line();
				} else {
					self.column += 1;
				}
				self.pos += 1;
			}
			if self.pos == code.len() {
				Err(self.eof_error())
			} else {
				self.inc_pos_col();
				Ok(String(box StringAst::new(buf)))
			}
		} else {
			Err(self.unexpected_error("\"", format!("'{}'", code.char_at(self.pos))))
		}
	}

	fn parse_boolean(&mut self) -> ParseResult<ExprAst> {
		let code: &mut str = unsafe { ::std::mem::transmute(self.code.as_slice()) };
		self.skip_whitespace();
		if self.pos == code.len() {
			Err(self.eof_error())
		} else {
			let mut buf = String::new();
			while self.pos < code.len() && code.char_at(self.pos).is_alphabetic() {
				buf.push_char(code.char_at(self.pos));
				self.inc_pos_col();
			}
			let string: &str = buf.as_slice();
			match string {
				"true" => Ok(Boolean(box BooleanAst::new(true))),
				"false" => Ok(Boolean(box BooleanAst::new(false))),
				other => Err(self.unexpected_error("\"true\" or \"false\"", format!("\"{}\"", other)))
			}
		}
	}

	fn parse_nil(&mut self) -> ParseResult<ExprAst> {
		let code: &mut str = unsafe { ::std::mem::transmute(self.code.as_slice()) };
		self.skip_whitespace();
		if self.pos == code.len() {
			Err(self.eof_error())
		} else {
			let mut buf = String::new();
			while self.pos < code.len() && code.char_at(self.pos).is_alphabetic() {
				buf.push_char(code.char_at(self.pos));
				self.inc_pos_col();
			}
			let string: &str = buf.as_slice();
			if string == "nil" {
				Ok(Nil(box NilAst::new()))
			} else {
				Err(self.unexpected_error("\"nil\"", format!("\"{}\"", string)))
			}
		}
	}

	fn parse_symbol(&mut self) -> ParseResult<ExprAst> {
		let code: &mut str = unsafe { ::std::mem::transmute(self.code.as_slice()) };
		self.skip_whitespace();
		if self.pos + 1 >= code.len() {
			Err(self.eof_error())
		} else if !self.is_ident_char(code.char_at(self.pos + 1)) {
			self.column += 1;
			Err(self.unexpected_error("alphabetic character", format!("'{}'", code.char_at(self.pos + 1))))
		} else if code.char_at(self.pos) == '\'' {
			self.inc_pos_col();
			let ident = try!(self.parse_ident_stack());
			Ok(Symbol(box SymbolAst::new(ident.value)))
		} else {
			Err(self.unexpected_error("\"'\"", format!("'{}'", code.char_at(self.pos))))
		}
	}

	fn parse_comment(&mut self) -> ParseResult<ExprAst> {
		let code: &mut str = unsafe { ::std::mem::transmute(self.code.as_slice()) };
		self.skip_whitespace();
		if self.pos == code.len() {
			Err(self.eof_error())
		} else if code.char_at(self.pos) == ';' {
			self.inc_pos_col();
			let mut buf = String::new();
			while self.pos < code.len() && code.char_at(self.pos) != '\n' {
				buf.push_char(code.char_at(self.pos));
				self.inc_pos_col();
			}
			Ok(Comment(box CommentAst::new(buf)))
		} else {
			Err(self.unexpected_error("';'", format!("'{}'", code.char_at(self.pos))))
		}
	}

	#[inline(always)]
	fn is_ident_char(&self, ch: char) -> bool {
		if ch.is_digit() || ch.is_whitespace() || ch == '(' || ch == ')' || ch == '[' || ch == ']' || ch == '\'' || ch == '"' || ch == ';' {
			false
		} else {
			true
		}
	}

	#[inline(always)]
	fn skip_whitespace(&mut self) {
		let code: &mut str = unsafe { ::std::mem::transmute(self.code.as_slice()) };
		while self.pos < code.len() && code.char_at(self.pos).is_whitespace() {
			if code.char_at(self.pos) == '\n' {
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
		ParseError::new(self.line, self.column, "end of file".to_string())
	}

	#[inline(always)]
	fn nyi_error<T: Str>(&self, item: T) -> ParseError {
		ParseError::new(self.line, self.column, format!("{} not yet implemented", item.as_slice()))
	}

	#[inline(always)]
	fn unexpected_error<T: Str, U: Str>(&self, expect: T, found: U) -> ParseError {
		ParseError::new(self.line, self.column, format!("expected {} but found {}", expect.as_slice(), found.as_slice()))
	}
}

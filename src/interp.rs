use self::parser::Parser;

mod parser;

#[deriving(Eq)]
pub enum InterpMode {
	Debug,
	Release
}

pub struct Interpreter {
	mode: InterpMode,
	parser: Parser
}

impl Interpreter {
	pub fn new() -> Interpreter {
		Interpreter {
			parser: Parser::new(),
			mode: Release
		}
	}

	pub fn set_mode(&mut self, mode: InterpMode) {
		self.mode = mode;
	}

	pub fn load_code(&mut self, code: ~str) {
		self.parser.load_code(code);
	}

	pub fn execute(&mut self) -> int {
		let mut ast = self.parser.parse();
		if self.mode != Debug {
			ast = ast.optimize().unwrap();
		}
		0 // exit status
	}
}

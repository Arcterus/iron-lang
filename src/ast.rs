#![allow(dead_code)]  // the code it warns about is not actually dead, so...

use std::cell::RefCell;
use std::rc::Rc;

static INDENTATION: uint = 2;

#[deriving(Clone, PartialEq)]
pub enum ExprAst {
   Root(RootAst),
   Sexpr(SexprAst),
   String(StringAst),
   List(ListAst),
   Array(ArrayAst),
   Pointer(PointerAst),
   Ident(IdentAst),
   Symbol(SymbolAst),
   Integer(IntegerAst),
   Float(FloatAst),
   Boolean(BooleanAst),
   Nil(NilAst),
   Comment(CommentAst),
   Code(CodeAst)
}

pub trait Ast {
   fn optimize(self) -> Option<ExprAst>;
   //fn eval(&self) -> Option<Box<Any>>;
   fn compile(&self) -> Vec<u8>;

   fn dump(&self) { self.dump_level(0) }

   // XXX: this should in actuality be private...
   fn dump_level(&self, level: uint);
}

#[deriving(Clone, PartialEq)]
pub struct RootAst {
   pub asts: Vec<ExprAst>
}

#[deriving(Clone, PartialEq)]
pub struct SexprAst {
   pub op: IdentAst,
   pub operands: Vec<ExprAst>
}

#[deriving(Clone, PartialEq)]
pub struct StringAst {
   pub string: String
}

#[deriving(Clone, PartialEq)]
pub struct ListAst {
   pub items: Vec<ExprAst>
}

#[deriving(Clone, PartialEq)]
pub struct ArrayAst {
   pub items: Vec<ExprAst>
}

#[deriving(Clone, PartialEq)]
pub struct PointerAst {
   pub pointee: Box<ExprAst>
}

#[deriving(Clone, PartialEq)]
pub struct IdentAst {
   pub value: String
}

#[deriving(Clone, PartialEq)]
pub struct SymbolAst {
   pub value: String
}

#[deriving(Clone, PartialEq)]
pub struct IntegerAst {
   pub value: i64
}

#[deriving(Clone, PartialEq)]
pub struct FloatAst {
   pub value: f64
}

#[deriving(Clone, PartialEq)]
pub struct BooleanAst {
   pub value: bool
}

#[deriving(Clone, PartialEq)]
pub struct NilAst;

#[deriving(Clone, PartialEq)]
pub struct CommentAst {
   pub value: String
}

#[deriving(Clone, PartialEq)]
pub struct CodeAst {
   pub params: ArrayAst,
   pub code: Vec<ExprAst>,
   pub env: Rc<RefCell<::interp::Environment>>
}

impl Ast for ExprAst {
   fn optimize(self) -> Option<ExprAst> {
      match self {
         Root(ast) => ast.optimize(),
         Sexpr(ast) => ast.optimize(),
         String(ast) => ast.optimize(),
         List(ast) => ast.optimize(),
         Array(ast) => ast.optimize(),
         Pointer(ast) => ast.optimize(),
         Ident(ast) => ast.optimize(),
         Symbol(ast) => ast.optimize(),
         Integer(ast) => ast.optimize(),
         Float(ast) => ast.optimize(),
         Boolean(ast) => ast.optimize(),
         Nil(ast) => ast.optimize(),
         Comment(ast) => ast.optimize(),
         Code(ast) => ast.optimize()
      }
   }

   fn compile(&self) -> Vec<u8> {
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
   fn optimize(self) -> Option<ExprAst> {
      let mut result = RootAst::new();
      result.asts = self.asts.move_iter().filter_map(|ast| ast.optimize()).collect();
      Some(Root(result))
   }

   fn compile(&self) -> Vec<u8> {
      let mut result = vec!();
      for ast in self.asts.iter() {
         result.push_all_move(ast.compile());
      }
      result
   }

   fn dump_level(&self, level: uint) {
      let mut spaces = String::new();
      for _ in range(0, level * INDENTATION) {
         spaces.push_char(' ');
      }
      println!("{}RootAst {}", spaces, "{");
      for ast in self.asts.iter() {
         ast.dump_level(level + 1);
      }
      println!("{}{}", spaces, "}");
   }
}

impl SexprAst {
   pub fn new(op: IdentAst, operands: Vec<ExprAst>) -> SexprAst {
      SexprAst {
         op: op,
         operands: operands
      }
   }

   fn is_math_op(&self) -> bool {
      match self.op.value.as_slice() {
         "add" | "sub" | "mul" | "div" => true,
         _ => false
      }
   }
}

impl Ast for SexprAst {
   fn optimize(self) -> Option<ExprAst> {
      if self.is_math_op() {
         // TODO: check if ops can be eliminated
      }
      Some(Sexpr(self))
   }

   fn compile(&self) -> Vec<u8> {
      vec!()
   }

   fn dump_level(&self, level: uint) {
      let mut spaces = String::new();
      for _ in range(0, level * INDENTATION) {
         spaces.push_char(' ');
      }
      println!("{}SexprAst {}", spaces, "{");
      self.op.dump_level(level + 1);
      for ast in self.operands.iter() {
         ast.dump_level(level + 1);
      }
      println!("{}{}", spaces, "}");
   }
}

impl StringAst {
   pub fn new(value: String) -> StringAst {
      StringAst {
         string: value
      }
   }
}

impl Ast for StringAst {
   fn optimize(self) -> Option<ExprAst> {
      // TODO: perhaps this should deal with a string table?
      Some(String(self))
   }

   fn compile(&self) -> Vec<u8> {
      vec!()
   }

   fn dump_level(&self, level: uint) {
      let mut buf = String::new();
      for _ in range(0, INDENTATION) {
         buf.push_char(' ');
      }
      let indent = buf.clone();
      let spaces =
         if level == 0 {
            "".to_string()
         } else {
            for _ in range(0, (level - 1) * INDENTATION) {
               buf.push_char(' ');
            }
            buf
         };
      println!("{}StringAst {}", spaces, "{");
      println!("{}{}\"{}\"", spaces, indent, self.string);
      println!("{}{}", spaces, "}");
   }
}

impl ListAst {
   pub fn new(items: Vec<ExprAst>) -> ListAst {
      ListAst {
         items: items
      }
   }
}

impl Ast for ListAst {
   fn optimize(self) -> Option<ExprAst> {
      Some(List(self))
   }

   fn compile(&self) -> Vec<u8> {
      vec!()
   }

   fn dump_level(&self, level: uint) {
      let mut spaces = String::new();
      for _ in range(0, level * INDENTATION) {
         spaces.push_char(' ');
      }
      println!("{}ListAst {}", spaces, "{");
      for item in self.items.iter() {
         item.dump_level(level + 1);
      }
      println!("{}{}", spaces, "}");
   }
}

impl ArrayAst {
   pub fn new(items: Vec<ExprAst>) -> ArrayAst {
      ArrayAst {
         items: items
      }
   }
}

impl Ast for ArrayAst {
   fn optimize(self) -> Option<ExprAst> {
      Some(Array(self))
   }

   fn compile(&self) -> Vec<u8> {
      vec!()
   }

   fn dump_level(&self, level: uint) {
      let mut spaces = String::new();
      for _ in range(0, level * INDENTATION) {
         spaces.push_char(' ');
      }
      println!("{}ArrayAst {}", spaces, "{");
      for item in self.items.iter() {
         item.dump_level(level + 1);
      }
      println!("{}{}", spaces, "}");
   }
}

impl Ast for PointerAst {
   fn optimize(self) -> Option<ExprAst> {
      Some(Pointer(self))
   }

   fn compile(&self) -> Vec<u8> {
      vec!()
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
   fn optimize(self) -> Option<ExprAst> {
      Some(Integer(self))
   }

   fn compile(&self) -> Vec<u8> {
      vec!()
   }

   fn dump_level(&self, level: uint) {
      let mut buf = String::new();
      for _ in range(0, INDENTATION) {
         buf.push_char(' ');
      }
      let indent = buf.clone();
      let spaces =
         if level == 0 {
            "".to_string()
         } else {
            for _ in range(0, (level - 1) * INDENTATION) {
               buf.push_char(' ');
            }
            buf
         };
      println!("{}IntegerAst {}", spaces, "{");
      println!("{}{}{}", spaces, indent, self.value);
      println!("{}{}", spaces, "}");
   }
}

impl IdentAst {
   pub fn new(ident: String) -> IdentAst {
      IdentAst {
         value: ident
      }
   }
}

impl Ast for IdentAst {
   fn optimize(self) -> Option<ExprAst> {
      Some(Ident(self))
   }

   fn compile(&self) -> Vec<u8> {
      vec!()
   }

   fn dump_level(&self, level: uint) {
      let mut buf = String::new();
      for _ in range(0, INDENTATION) {
         buf.push_char(' ');
      }
      let indent = buf.clone();
      let spaces =
         if level == 0 {
            "".to_string()
         } else {
            for _ in range(0, (level - 1) * INDENTATION) {
               buf.push_char(' ');
            }
            buf
         };
      println!("{}IdentAst {}", spaces, "{");
      println!("{}{}{}", spaces, indent, self.value);
      println!("{}{}", spaces, "}");
   }
}

impl SymbolAst {
   pub fn new(value: String) -> SymbolAst {
      SymbolAst {
         value: value
      }
   }
}

impl Ast for SymbolAst {
   fn optimize(self) -> Option<ExprAst> {
      Some(Symbol(self))
   }

   fn compile(&self) -> Vec<u8> {
      vec!()
   }

   fn dump_level(&self, level: uint) {
      let mut buf = String::new();
      for _ in range(0, INDENTATION) {
         buf.push_char(' ');
      }
      let indent = buf.clone();
      let spaces =
         if level == 0 {
            "".to_string()
         } else {
            for _ in range(0, (level - 1) * INDENTATION) {
               buf.push_char(' ');
            }
            buf
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
   fn optimize(self) -> Option<ExprAst> {
      Some(Float(self))
   }

   fn compile(&self) -> Vec<u8> {
      vec!()
   }

   fn dump_level(&self, level: uint) {
      let mut buf = String::new();
      for _ in range(0, INDENTATION) {
         buf.push_char(' ');
      }
      let indent = buf.clone();
      let spaces =
         if level == 0 {
            "".to_string()
         } else {
            for _ in range(0, (level - 1) * INDENTATION) {
               buf.push_char(' ');
            }
            buf
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
   fn optimize(self) -> Option<ExprAst> {
      Some(Boolean(self))
   }

   fn compile(&self) -> Vec<u8> {
      vec!()
   }

   fn dump_level(&self, level: uint) {
      let mut buf = String::new();
      for _ in range(0, INDENTATION) {
         buf.push_char(' ');
      }
      let indent = buf.clone();
      let spaces =
         if level == 0 {
            "".to_string()
         } else {
            for _ in range(0, (level - 1) * INDENTATION) {
               buf.push_char(' ');
            }
            buf
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
   fn optimize(self) -> Option<ExprAst> {
      Some(Nil(self))
   }

   fn compile(&self) -> Vec<u8> {
      vec!()
   }

   fn dump_level(&self, level: uint) {
      let mut buf = String::new();
      for _ in range(0, level * INDENTATION) {
         buf.push_char(' ');
      }
      println!("{}NilAst", buf);
   }
}

impl CommentAst {
   pub fn new(value: String) -> CommentAst {
      CommentAst {
         value: value
      }
   }
}

impl Ast for CommentAst {
   fn optimize(self) -> Option<ExprAst> {
      Some(Comment(self))
   }

   fn compile(&self) -> Vec<u8> {
      vec!()
   }

   fn dump_level(&self, level: uint) {
      let mut buf = String::new();
      for _ in range(0, INDENTATION) {
         buf.push_char(' ');
      }
      let indent = buf.clone();
      let spaces =
         if level == 0 {
            "".to_string()
         } else {
            for _ in range(0, (level - 1) * INDENTATION) {
               buf.push_char(' ');
            }
            buf
         };
      println!("{}CommentAst {}", spaces, "{");
      println!("{}{}{}", spaces, indent, self.value);
      println!("{}{}", spaces, "}");
   }
}

impl CodeAst {
   pub fn new(params: ArrayAst, code: Vec<ExprAst>, env: Rc<RefCell<::interp::Environment>>) -> CodeAst {
      CodeAst {
         params: params,
         code: code,
         env: env
      }
   }
}

impl Ast for CodeAst {
   fn optimize(self) -> Option<ExprAst> {
      Some(Code(self))
   }

   fn compile(&self) -> Vec<u8> {
      vec!()
   }

   fn dump_level(&self, _: uint) { }
}

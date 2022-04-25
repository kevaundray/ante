//! This module defines the High-level Intermediate Representation's AST.
//!
//! The goal of this Ast is to function as a simpler Ast for the backends
//! to consume. In comparison to the main Ast, this one:
//! - Has no reliance on the ModuleCache
//! - Has all generic types removed either through monomorphisation or boxing
//! - All trait function calls are replaced with references to the exact
//!   function to call statically (monomorphisation) or are passed in as
//!   arguments to calling functions (boxing).
mod types;
mod monomorphisation;
mod decision_tree_monomorphisation;
mod printer;

use std::rc::Rc;

pub use monomorphisation::monomorphise;

use types::{ Type, IntegerKind, FunctionType };

use self::printer::FmtAst;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DefinitionId(usize);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Literal {
    Integer(u64, IntegerKind),
    Float(u64),
    CString(String),
    Char(char),
    Bool(bool),
    Unit,
}

#[derive(Debug, Clone)]
pub struct DefinitionInfo {
    /// The Ast for the Ast::Definition which defines this Variable.
    /// This may be None if this variable was defined from a function
    /// parameter or a match pattern.
    ///
    /// This Ast is expected to contain a hir::Definition in the form
    /// `id = expr` where id == self.definition_id. Most definitions will
    /// be exactly this, but others may be a sequence of several definitions
    /// in the case of e.g. tuple unpacking.
    pub definition: Option<Rc<Ast>>,

    pub definition_id: DefinitionId,
}

pub type Variable = DefinitionInfo;

impl From<Variable> for Ast {
    fn from(v: Variable) -> Ast {
        Ast::Variable(v)
    }
}

impl From<DefinitionId> for Variable {
    fn from(definition_id: DefinitionId) -> Variable {
        Variable { definition_id, definition: None }
    }
}

/// \a b. expr
/// Function definitions are also desugared to a ast::Definition with a ast::Lambda as its body
#[derive(Debug, Clone)]
pub struct Lambda {
    pub args: Vec<Ast>,
    pub body: Box<Ast>,
    pub typ: FunctionType,
}

/// foo a b c
#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub function: Box<Ast>,
    pub args: Vec<Ast>,
}

/// Unlike ast::Definition, hir::Definition
/// is desugared of any patterns, its lhs must
/// be a single variable to simplify backends.
#[derive(Debug, Clone)]
pub struct Definition {
    pub variable: DefinitionId,
    pub expr: Box<Ast>,
    pub mutable: bool,
}

impl From<Definition> for DefinitionInfo {
    fn from(def: Definition) -> Self {
        DefinitionInfo {
            definition_id: def.variable,
            definition: Some(Rc::new(Ast::Definition(def))),
        }
    }
}

/// if condition then expression else expression
#[derive(Debug, Clone)]
pub struct If {
    pub condition: Box<Ast>,
    pub then: Box<Ast>,
    pub otherwise: Option<Box<Ast>>,
}

#[derive(Debug, Clone)]
pub struct Match {
    // Unlike ast::Match this only contains the parts of the
    // branch after the ->.
    pub branches: Vec<Ast>,

    pub decision_tree: DecisionTree,
}

// This cannot be desugared into Ast::If due to the sharing
// of Leafs across separate branches. E.g. a match on:
// ```
// match foo
// | None, None -> ...
// | _ -> ...
// ```
// Compiles to the tree:
// ```
// Switch value1 {
//     Some -> Leaf(1)
//     None -> {
//         switch value2 {
//             Some -> Leaf(1)
//             None -> Leaf(0)
//         }
//     }
// }
// ```
// Where two different paths need to share the same leaf branch.
#[derive(Debug, Clone)]
pub enum DecisionTree {
    Leaf(usize),
    Definition(Definition, Box<DecisionTree>),
    Switch {
        int_to_switch_on: Box<Ast>,
        cases: Vec<(u32, DecisionTree)>,
        else_case: Option<Box<DecisionTree>>,
    },
}

/// return expression
#[derive(Debug, Clone)]
pub struct Return {
    pub expression: Box<Ast>,
}

/// statement1
/// statement2
/// ...
/// statementN
#[derive(Debug, Clone)]
pub struct Sequence {
    pub statements: Vec<Ast>,
}

/// extern declaration
/// // or
/// extern
///     declaration1
///     declaration2
///     ...
///     declarationN
#[derive(Debug, Clone)]
pub struct Extern {
    pub name: String,
    pub typ: Type,
}

/// lhs := rhs
#[derive(Debug, Clone)]
pub struct Assignment {
    pub lhs: Box<Ast>,
    pub rhs: Box<Ast>,
}

#[derive(Debug, Clone)]
pub struct MemberAccess{
    pub lhs: Box<Ast>,
    pub member_index: u32,
}

#[derive(Debug, Clone)]
pub struct Tuple {
    pub fields: Vec<Ast>,
}

/// Essentially the same as Builtin::Transmute.
/// Enum variants are padded with extra bytes
/// then lowered to this. lhs's type should be the same
/// size as the target type, though there may be
/// padding differences currently.
#[derive(Debug, Clone)]
pub struct ReinterpretCast {
    pub lhs: Box<Ast>,
    pub target_type: Type,
}

#[derive(Debug, Copy, Clone)]
pub enum Builtin {
    AddInt,
    AddFloat,

    SubInt,
    SubFloat,

    MulInt,
    MulFloat,

    DivInt,
    DivFloat,

    ModInt,
    ModFloat,

    LessInt,
    LessFloat,

    GreaterInt,
    GreaterFloat,

    EqInt,
    EqFloat,
    EqChar,
    EqBool,

    SignExtend,
    ZeroExtend,
    Truncate,
    Deref,
    Offset,
    Transmute,
}

#[derive(Debug, Clone)]
pub enum Ast {
    Literal(Literal),
    Variable(Variable),
    Lambda(Lambda),
    FunctionCall(FunctionCall),
    Definition(Definition),
    If(If),
    Match(Match),
    Return(Return),
    Sequence(Sequence),
    Extern(Extern),
    Assignment(Assignment),
    MemberAccess(MemberAccess),
    Tuple(Tuple),
    ReinterpretCast(ReinterpretCast),
    Builtin(Builtin),
}

impl std::fmt::Display for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut printer = printer::AstPrinter::default();
        self.fmt_ast(&mut printer, f)?;

        while let Some(ast) = printer.queue.pop_front() {
            write!(f, "\n\n")?;
            ast.fmt_ast(&mut printer, f)?;
        }

        Ok(())
    }
}

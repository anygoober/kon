use std::panic;

use bumpalo::{Bump, boxed::Box};
use peg::{error::ParseError, parser, str::LineCol};

#[derive(Debug)]
pub enum StringPart<'bump, 'input> {
    Literal(&'input str),
    Interp {
        expr: Box<'bump, Expr<'bump, 'input>>,
        format: Option<&'input str>,
    },
}

impl<'bump, 'input> StringPart<'bump, 'input> {
    pub fn clone_in(&self, bump: &'bump Bump) -> Self {
        match self {
            Self::Literal(lit) => Self::Literal(lit),
            Self::Interp { expr, format } => Self::Interp {
                expr: Box::new_in(expr.clone_in(bump), bump),
                format: *format,
            },
        }
    }
}

#[derive(Debug)]
pub enum Expr<'bump, 'input> {
    Number(f64),
    Bool(bool),
    Not(Box<'bump, Expr<'bump, 'input>>),
    If {
        cond: Box<'bump, Expr<'bump, 'input>>,
        then_branch: Vec<Stmt<'bump, 'input>>,
        else_branch: Option<Vec<Stmt<'bump, 'input>>>,
    },
    While {
        cond: Box<'bump, Expr<'bump, 'input>>,
        body: Vec<Stmt<'bump, 'input>>,
    },
    Ident(&'input str),
    Str(Vec<StringPart<'bump, 'input>>),
    EnumVariant(&'input str, &'input str), // Type.Variant
    DotVariant(&'input str),               // .Variant
    Neg(Box<'bump, Expr<'bump, 'input>>),
    Binary(
        Box<'bump, Expr<'bump, 'input>>,
        BinOp,
        Box<'bump, Expr<'bump, 'input>>,
    ),
    Call(Box<'bump, Expr<'bump, 'input>>, Vec<Expr<'bump, 'input>>),
    MethodCall(
        Box<'bump, Expr<'bump, 'input>>,
        &'input str,
        Vec<Expr<'bump, 'input>>,
    ),
    Field(Box<'bump, Expr<'bump, 'input>>, &'input str),
    Index(
        Box<'bump, Expr<'bump, 'input>>,
        Box<'bump, Expr<'bump, 'input>>,
    ),
    StructLit(&'input str, Vec<StructInitializer<'bump, 'input>>),
    Switch(
        Box<'bump, Expr<'bump, 'input>>,
        Vec<SwitchArm<'bump, 'input>>,
    ),
    For {
        binders: Vec<&'input str>,
        iter: Box<'bump, Expr<'bump, 'input>>,
        body: Vec<Stmt<'bump, 'input>>,
    },
    Block(Vec<Stmt<'bump, 'input>>),
}

impl<'bump, 'input> Expr<'bump, 'input> {
    pub fn clone_in(&self, bump: &'bump Bump) -> Self {
        match self {
            Self::Number(n) => Self::Number(*n),
            Self::Bool(b) => Self::Bool(*b),
            Self::Not(expr) => Self::Not(Box::new_in(expr.clone_in(bump), bump)),
            Self::If {
                cond,
                then_branch,
                else_branch,
            } => Self::If {
                cond: Box::new_in(cond.clone_in(bump), bump),
                then_branch: then_branch.iter().map(|e| e.clone_in(bump)).collect(),
                else_branch: else_branch
                    .as_ref()
                    .map(|inner| inner.iter().map(|s| s.clone_in(bump)).collect()),
            },

            Self::While { cond, body } => Self::While {
                cond: Box::new_in(cond.clone_in(bump), bump),
                body: body.iter().map(|e| e.clone_in(bump)).collect(),
            },

            Self::Ident(s) => Self::Ident(s),

            Self::Str(parts) => Self::Str(parts.iter().map(|p| p.clone_in(bump)).collect()),

            Self::EnumVariant(ty, var) => Self::EnumVariant(ty, var),

            Self::DotVariant(var) => Self::DotVariant(var),

            Self::Neg(expr) => Self::Neg(Box::new_in(expr.clone_in(bump), bump)),

            Self::Binary(lhs, op, rhs) => Self::Binary(
                Box::new_in(lhs.clone_in(bump), bump),
                *op,
                Box::new_in(rhs.clone_in(bump), bump),
            ),

            Self::Call(func, args) => Self::Call(
                Box::new_in(func.clone_in(bump), bump),
                args.iter().map(|e| e.clone_in(bump)).collect(),
            ),

            Self::MethodCall(receiver, method, args) => Self::MethodCall(
                Box::new_in(receiver.clone_in(bump), bump),
                method,
                args.iter().map(|e| e.clone_in(bump)).collect(),
            ),

            Self::Field(expr, field) => Self::Field(Box::new_in(expr.clone_in(bump), bump), field),

            Self::Index(lhs, rhs) => Self::Index(
                Box::new_in(lhs.clone_in(bump), bump),
                Box::new_in(rhs.clone_in(bump), bump),
            ),

            Self::StructLit(name, fields) => {
                Self::StructLit(name, fields.iter().map(|f| f.clone_in(bump)).collect())
            }

            Self::Switch(expr, arms) => Self::Switch(
                Box::new_in(expr.clone_in(bump), bump),
                arms.iter().map(|a| a.clone_in(bump)).collect(),
            ),

            Self::For {
                binders,
                iter,
                body,
            } => Self::For {
                binders: binders.clone(),
                iter: Box::new_in(iter.clone_in(bump), bump),
                body: body.iter().map(|s| s.clone_in(bump)).collect(),
            },

            Self::Block(stmts) => Self::Block(stmts.iter().map(|s| s.clone_in(bump)).collect()),
        }
    }
}

#[derive(Debug)]
pub struct StructInitializer<'bump, 'input> {
    pub label: &'input str,
    pub value: Expr<'bump, 'input>,
}

impl<'bump, 'input> StructInitializer<'bump, 'input> {
    pub fn clone_in(&self, bump: &'bump Bump) -> Self {
        Self {
            label: self.label,
            value: self.value.clone_in(bump),
        }
    }
}

#[derive(Debug)]
pub struct SwitchArm<'bump, 'input> {
    pub patterns: Vec<Expr<'bump, 'input>>,
    pub body: Expr<'bump, 'input>,
}

impl<'bump, 'input> SwitchArm<'bump, 'input> {
    pub fn clone_in(&self, bump: &'bump Bump) -> Self {
        Self {
            patterns: self
                .patterns
                .iter()
                .map(|item| item.clone_in(bump))
                .collect(),
            body: self.body.clone_in(bump),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug)]
pub enum Stmt<'bump, 'input> {
    Let(&'input str, Expr<'bump, 'input>),
    Expr(Expr<'bump, 'input>),
    Tail(Expr<'bump, 'input>),
    Return(Option<Expr<'bump, 'input>>),
    Break,
    Continue,
}

impl<'bump, 'input> Stmt<'bump, 'input> {
    pub fn clone_in(&self, bump: &'bump Bump) -> Self {
        match self {
            Self::Let(name, expr) => Self::Let(name, expr.clone_in(bump)),
            Self::Expr(expr) => Self::Expr(expr.clone_in(bump)),
            Self::Tail(expr) => Self::Tail(expr.clone_in(bump)),
            Self::Return(expr) => Self::Return(expr.as_ref().map(|e| e.clone_in(bump))),
            Self::Break => Self::Break,
            Self::Continue => Self::Continue,
        }
    }
}

#[derive(Debug)]
pub struct Param<'input> {
    pub name: &'input str,
    pub ty: Option<Type<'input>>,
}

#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    Public,
    Unspecified,
}

#[derive(Debug)]
pub struct TopLevelItem<'bump, 'input> {
    pub visibility: Visibility,
    pub item: Item<'bump, 'input>,
}

#[derive(Debug)]
pub enum Item<'bump, 'input> {
    Let(Box<'bump, LetItem<'bump, 'input>>),
    Const(Box<'bump, ConstItem<'bump, 'input>>),
    Enum(Box<'bump, EnumItem<'input>>),
    Struct(Box<'bump, StructItem<'input>>),
    Fn(Box<'bump, FnItem<'bump, 'input>>),
    Interface(Box<'bump, InterfaceItem<'bump, 'input>>),
    Import(Box<'bump, ImportItem<'input>>),

    Extern(Box<'bump, ExternItem<'bump, 'input>>),
    ExternFnItem(Box<'bump, ExternFnItem<'input>>),

    // c macros support
    CMacroInclude(&'input str),
}

#[derive(Debug)]
pub enum ImportItem<'input> {
    Module(Path<'input>),
    Specific {
        path: Path<'input>,
        idents: Vec<&'input str>,
        relative: bool,
    },
}

#[derive(Debug)]
pub struct LetItem<'bump, 'input> {
    pub ident: &'input str,
    pub expr: Expr<'bump, 'input>,
    pub ty: Option<Type<'input>>,
}

#[derive(Debug)]
pub struct ConstItem<'bump, 'input> {
    pub ident: &'input str,
    pub expr: Expr<'bump, 'input>,
    pub ty: Option<Type<'input>>,
}

#[derive(Debug)]
pub struct EnumItem<'input> {
    pub name: &'input str,
    pub variants: Vec<&'input str>,
    pub traits: Vec<Type<'input>>,
}

#[derive(Debug)]
pub struct StructItem<'input> {
    pub name: &'input str,
    pub fields: Vec<Param<'input>>,
    pub traits: Vec<Type<'input>>,
}

#[derive(Debug)]
pub struct FnItem<'bump, 'input> {
    pub receiver: Option<Type<'input>>,
    pub allocator_receiver: Option<&'input str>,
    pub name: &'input str,
    pub params: Vec<Param<'input>>,
    pub body: Vec<Stmt<'bump, 'input>>,
    pub return_type: Option<Type<'input>>,
}

#[derive(Debug)]
pub struct ExternFnItem<'input> {
    pub name: ExternFnName<'input>,
    pub params: Vec<Param<'input>>,
    pub return_type: Option<Type<'input>>,
    pub variadic_args: bool, // the '...' va_arg-ish c code
}

#[derive(Debug)]
pub enum ExternFnName<'input> {
    Name(&'input str),
    Rename {
        external: &'input str,
        rename: &'input str,
    },
}

#[derive(Debug)]
pub struct InterfaceItem<'bump, 'input> {
    pub name: &'input str,
    pub methods: Vec<InterfaceMethod<'bump, 'input>>,
    pub traits: Vec<Type<'input>>,
}

#[derive(Debug)]
pub struct InterfaceMethod<'bump, 'input> {
    pub name: &'input str,
    pub allocator_receiver: Option<&'input str>,
    pub params: Vec<Param<'input>>,
    pub return_type: Option<Type<'input>>,
    pub body: Vec<Stmt<'bump, 'input>>,
}

#[derive(Debug)]
pub struct ExternItem<'bump, 'input> {
    pub lang: &'input str,
    pub items: Vec<Stmt<'bump, 'input>>,
}

#[derive(Debug)]
pub struct Path<'input>(pub Vec<&'input str>);

#[derive(Debug)]
pub enum TypeModifier<'input> {
    None,
    Optional,
    Undefined,
    ErrorUnion { error: Option<Path<'input>> },
}

#[derive(Debug)]
pub struct Type<'input> {
    pub ident: Path<'input>,
    pub params: Vec<&'input str>,
    pub pointer: Option<PointerKind>,
    pub modifier: TypeModifier<'input>,
}

#[derive(Debug)]
pub enum PointerKind {
    Const,
    Mut,
}

enum LaterPostfixOp<'bump, 'input> {
    MethodCall(&'input str, Vec<Expr<'bump, 'input>>),
    Field(&'input str),
    Call(Vec<Expr<'bump, 'input>>),
    Index(Expr<'bump, 'input>),
}

parser! {
    pub grammar kon_parser<'bump>(bump: &'bump Bump) for str {

        rule _() = quiet!{ ([' ' | '\t' | '\n' | '\r'] / comment())* }
        rule comment()
            = "///" (!['\n'][_])*
            / "//" (!['\n'][_])*

        rule __() = quiet!{ [' ' | '\t' | '\n' | '\r']+ }

        rule keyword() -> ()
            = (
                "let" / "fn" / "enum" / "struct" / "switch" / "for" / "interface"
                / "return" / "extern" / "pub" / "if" / "else" / "while" / "true"
                / "false" / "break" / "continue"
            )
                ![ 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' ]

        rule ident() -> &'input str
            = quiet!{ !keyword() s:$(['a'..='z' | 'A'..='Z' | '_'] ['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*) { s } }
            / expected!("identifier")

        rule path() -> Path<'input>
            = parts:(ident() ++ (".")) { Path(parts) }

        rule pointer_kind() -> PointerKind
            = "*" _ "const" __ { PointerKind::Const }
            / "*" _ "mut" __ { PointerKind::Mut }

        rule type_noresult() -> Type<'input>
            = undefined_type()
            / optional_type()
            / pointer:pointer_kind()? p:path() _ params:type_params()? {
                Type { ident: p, params: params.unwrap_or_default(), pointer, modifier: TypeModifier::None }
            }

        rule typ() -> Type<'input>
            = type_noresult() / result_type()

        rule rt_typ() -> Type<'input>
            = type_noresult() / result_type_noerror()

        rule undefined_type() -> Type<'input>
            = "??" _ u:undefined_type_inner() { u }
            / "??" _ "(" _ u:undefined_type_inner() _ ")" { u }

        rule undefined_type_inner() -> Type<'input>
            = p:path() _ params:type_params()? {
                Type { ident: p, params: params.unwrap_or_default(), pointer: None, modifier: TypeModifier::Undefined }
            }

        rule optional_type() -> Type<'input>
            = "?" _ o:optional_type_inner() { o }
            / "?" _ "(" _ o:optional_type_inner() _ ")" { o }

        rule optional_type_inner() -> Type<'input>
            = pointer:pointer_kind()? p:path() _ params:type_params()? {
                Type { ident: p, params: params.unwrap_or_default(), pointer, modifier: TypeModifier::Optional }
            }

        rule result_type() -> Type<'input>
            = error:result_type_err() _ "!" _ ok:result_type_ok() {
                let (ident, params, pointer) = ok;
                Type { ident, params, pointer, modifier: TypeModifier::ErrorUnion { error: Some(error) } }
            }

        rule result_type_noerror() -> Type<'input>
            = "!" _ ok:result_type_ok() {
                let (ident, params, pointer) = ok;
                Type { ident, params, pointer, modifier: TypeModifier::ErrorUnion { error: None } }
            }

        rule result_type_err() -> Path<'input>
            = path()
            / "(" _ err:path() _ ")" { err }

        rule result_type_ok() -> (Path<'input>, Vec<&'input str>, Option<PointerKind>)
            = o:result_type_ok_inner() { o }
            / "(" _ o:result_type_ok_inner() _ ")" { o }

        rule result_type_ok_inner() -> (Path<'input>, Vec<&'input str>, Option<PointerKind>)
            = pointer:pointer_kind()? p:path() _ params:type_params()? {
                (p, params.unwrap_or_default(), pointer)
            }

        rule type_params() -> Vec<&'input str>
            = "<" _ params:(ident() ** (_ "," _))  _ ">" { params }

        rule number() -> f64
            = n:$(['0'..='9']+ ("." ['0'..='9']+)?) { n.parse().unwrap() }

        rule bool_lit() -> Expr<'bump, 'input>
            = "true" ![ 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' ] { Expr::Bool(true) }
            / "false" ![ 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' ] { Expr::Bool(false) }

        pub rule program() -> Vec<TopLevelItem<'bump, 'input>>
            = _ items:(toplevel_item() ** _) _ { items }

        rule nonimport_item() -> Item<'bump, 'input>
            = let_item()
            / const_item()
            / extern_item()
            / enum_item()
            / struct_item()
            / fn_item()
            / interface_item()

        rule visibility() -> Visibility
            = "pub" __ { Visibility::Public }
            / _ { Visibility::Unspecified }

        rule item() -> Item<'bump, 'input>
            = nonimport_item() / import_item()

        rule toplevel_item() -> TopLevelItem<'bump, 'input>
            = vis:visibility() itm:nonimport_item() { TopLevelItem { visibility: vis, item: itm } }
            / import:import_item() { TopLevelItem { visibility: Visibility::Unspecified, item: import } }

        rule import_item() -> Item<'bump, 'input>
            = "import" __ module:path()
              { Item::Import(Box::new_in(ImportItem::Module(module), bump)) }
            / "from" __ relative:(".")? module:path() __ "import" __ idents:import_identifiers()
              { Item::Import(Box::new_in(ImportItem::Specific { path: module, idents, relative: relative.is_some() }, bump))  }

        rule import_identifiers() -> Vec<&'input str>
            = "(" _ idents:(ident() ** (_ "," _)) ")" { idents }
            / name:ident() { vec![name] }

        rule let_item() -> Item<'bump, 'input>
            = "let" __ name:ident()
              _ ty:(":" _ ty:typ() { ty })?
              _ "=" _ e:expr() _ ";" {
                  Item::Let(Box::new_in(LetItem { ident: name, expr: e, ty }, bump))
              }

        rule const_item() -> Item<'bump, 'input>
            = "const" __ name:ident()
            _ ty:(":" _ ty:typ() { ty })?
            _ "=" _ e:expr() _ ";" {
                Item::Const(Box::new_in(ConstItem { ident: name, expr: e, ty }, bump))
            }

        rule extern_item() -> Item<'bump, 'input>
            = "extern" __ lang:string_lit() _ "{" extern_body() "}"
            { Item::Extern(Box::new_in(ExternItem { lang, items: vec![] }, bump)) }

        rule extern_body() -> Vec<Item<'bump, 'input>>
            = _ items:(extern_item_body() ** _) _ { items }

        rule extern_item_body() -> Item<'bump, 'input>
            = struct_item()
            / enum_item()
            / extern_fn_item()
            / extern_include_item()

        rule extern_include_item() -> Item<'bump, 'input>
            = "#include" __ name:extern_include_name() { Item::CMacroInclude(name) }

        rule extern_include_name() -> &'input str
            = string_lit()
            / "<" _ name:$((!['>'] [_])+) _ ">" { name }

        rule extern_fn_item() -> Item<'bump, 'input>
            = "fn" __ name:(extern_fn_name()) _
                "(" _ params:(param() ** (_ "," _)) _ variadic:("," _ "...")? _ ")" _
              rt:fn_rt()? ";" {
                Item::ExternFnItem(Box::new_in(ExternFnItem {
                    name,
                    params,
                    return_type: rt,
                    variadic_args: variadic.is_some()
                }, bump))
              }

        rule extern_fn_name() -> ExternFnName<'input>
            = external:(string_lit() / ident()) _ rename:extern_fn_rename() { ExternFnName::Rename { external, rename } }
            / name:ident() { ExternFnName::Name(name) }

        rule extern_fn_rename() -> &'input str
            = "=>" _ name:ident() _ { name }

        rule enum_item() -> Item<'bump, 'input>
            = "enum" __ traits:satisfies_traits()? name:ident() _ "{" _
              variants:(enum_variant() ** (_ "," _)) _ ","? _
              "}" { Item::Enum(Box::new_in(EnumItem { name, variants, traits: traits.unwrap_or_default() }, bump)) }

        rule enum_variant() -> &'input str
            = "." v:ident() { v }

        rule struct_item() -> Item<'bump, 'input>
            = struct_item_with_fields()
            / "struct" __ traits:satisfies_traits()? name:ident() _ ";" {
                Item::Struct(Box::new_in(StructItem { name, fields: vec![], traits: traits.unwrap_or_default() }, bump))
            }

        rule struct_item_with_fields() -> Item<'bump, 'input>
            = "struct" __ traits:satisfies_traits()? name:ident() _ "{" _
              fields:(param() ** (_ "," _)) _ ","? _
              "}" { Item::Struct(Box::new_in(StructItem { name, fields, traits: traits.unwrap_or_default() }, bump)) }

        rule satisfies_traits() -> Vec<Type<'input>>
            = "(" _ traits:(typ() ** (_ "," _)) _ ")" _ { traits }

        rule param() -> Param<'input>
            = name:ident() ty:(_ ":" _ ty:typ() { ty })? { Param { name, ty } }

        rule fn_item() -> Item<'bump, 'input>
            = "fn" __ alloc_recv:alloc_receiver()? recv:receiver()? name:ident() _
              "(" _ params:(param() ** (_ "," _)) _ ")" _
              rt:fn_rt()?
              body:block() {
                Item::Fn(Box::new_in(FnItem {
                    receiver: recv,
                    allocator_receiver: alloc_recv,
                    name,
                    params,
                    body,
                    return_type: rt
                }, bump))
              }

        rule fn_rt() -> Type<'input>
            = "->" _ typ:rt_typ() _ { typ }

        rule alloc_receiver() -> &'input str
            = "[" _ name:ident() _ "]" _ { name }

        rule receiver() -> Type<'input>
            = "(" _ p:receiver_type() _ ")" "." { p }
            / _ p:receiver_type() "." { p }

        rule receiver_type() -> Type<'input>
            = pointer:pointer_kind()? name:ident() _ params:type_params()? {
                Type { ident: Path(vec![name]), params: params.unwrap_or_default(), pointer, modifier: TypeModifier::None }
            }

        rule interface_item() -> Item<'bump, 'input>
            = "interface" __ traits:satisfies_traits()? name:ident() _ "{" _
              methods:(interface_method() ** _) _
              "}" { Item::Interface(Box::new_in(InterfaceItem { name, methods, traits: traits.unwrap_or_default() }, bump)) }

        rule interface_method() -> InterfaceMethod<'bump, 'input>
            = "fn" __ alloc_recv:alloc_receiver()? name:ident() _
              "(" _ params:(param() ** (_ "," _)) _ ")" _ rt:fn_rt()?
              body:(";" { vec![] } / b:block() { b }) {
                InterfaceMethod { name, params, return_type: rt, allocator_receiver: alloc_recv, body }
              }

        rule block() -> Vec<Stmt<'bump, 'input>>
            = "{" _ stmts:(stmt()*) _ tail:expr()? _ "}" {
                let mut stmts = stmts;
                if let Some(e) = tail {
                    stmts.push(Stmt::Tail(e));
                }
                stmts
            }

        rule block_expr() -> Expr<'bump, 'input>
            = switch_expr() / if_expr() / while_expr()

        rule if_expr() -> Expr<'bump, 'input>
            = "if" _ cond:expr() _ then_branch:block()
            else_branch:(_ "else" _ e:else_tail() { e })? {
                Expr::If { cond: Box::new_in(cond, bump), then_branch, else_branch }
            }

        rule else_tail() -> Vec<Stmt<'bump, 'input>>
            = b:block() { b }
            / e:if_expr() { vec![Stmt::Tail(e)] }

        rule while_expr() -> Expr<'bump, 'input>
            = "while" _ cond:expr() _ body:block() {
                Expr::While { cond: Box::new_in(cond, bump), body }
            }

        rule return_stmt() -> Stmt<'bump, 'input>
            = "return" _ e:expr()? _ ";" { Stmt::Return(e) }

        rule stmt() -> Stmt<'bump, 'input>
            = "let" __ name:ident() _ "=" _ e:expr() _ ";" { Stmt::Let(name, e) }
            / return_stmt()
            / "break" _ ";" { Stmt::Break }
            / "continue" _ ";" { Stmt::Continue }
            / e:block_expr() _ ";"? { Stmt::Expr(e) }
            / e:expr() _ ";" { Stmt::Expr(e) }

        pub rule expr() -> Expr<'bump, 'input> = precedence!{
            x:(@) _ "||" _ y:@ { Expr::Binary(Box::new_in(x, bump), BinOp::Or, Box::new_in(y, bump)) }
                --
            x:(@) _ "&&" _ y:@ { Expr::Binary(Box::new_in(x, bump), BinOp::And, Box::new_in(y, bump)) }
                --
            x:(@) _ "==" _ y:@ { Expr::Binary(Box::new_in(x, bump), BinOp::Eq, Box::new_in(y, bump)) }
            x:(@) _ "!=" _ y:@ { Expr::Binary(Box::new_in(x, bump), BinOp::Ne, Box::new_in(y, bump)) }
            x:(@) _ "<=" _ y:@ { Expr::Binary(Box::new_in(x, bump), BinOp::Le, Box::new_in(y, bump)) }
            x:(@) _ ">=" _ y:@ { Expr::Binary(Box::new_in(x, bump), BinOp::Ge, Box::new_in(y, bump)) }
            x:(@) _ "<" _ y:@  { Expr::Binary(Box::new_in(x, bump), BinOp::Lt, Box::new_in(y, bump)) }
            x:(@) _ ">" _ y:@  { Expr::Binary(Box::new_in(x, bump), BinOp::Gt, Box::new_in(y, bump)) }
                --
            x:(@) _ "+" _ y:@ { Expr::Binary(Box::new_in(x, bump), BinOp::Add, Box::new_in(y, bump)) }
            x:(@) _ "-" _ y:@ { Expr::Binary(Box::new_in(x, bump), BinOp::Sub, Box::new_in(y, bump)) }
                --
            x:(@) _ "*" _ y:@ { Expr::Binary(Box::new_in(x, bump), BinOp::Mul, Box::new_in(y, bump)) }
            x:(@) _ "/" _ y:@ { Expr::Binary(Box::new_in(x, bump), BinOp::Div, Box::new_in(y, bump)) }
                --
            "-" e:@ { Expr::Neg(Box::new_in(e, bump)) }
            "!" e:@ { Expr::Not(Box::new_in(e, bump)) }
                --
            e:postfix() { e }
        }

        rule postfix() -> Expr<'bump, 'input>
            = base:primary() ops:postfix_op()* {
                ops.into_iter().fold(base, |acc, op| match op {
                                LaterPostfixOp::MethodCall(name, args) => {
                                    Expr::MethodCall(
                                        Box::new_in(acc, bump),
                                        name,
                                        args,
                                    )
                                }

                                LaterPostfixOp::Field(name) => {
                                    Expr::Field(
                                        Box::new_in(acc, bump),
                                        name,
                                    )
                                }

                                LaterPostfixOp::Call(args) => {
                                    Expr::Call(
                                        Box::new_in(acc, bump),
                                        args,
                                    )
                                }

                                LaterPostfixOp::Index(idx) => {
                                    Expr::Index(
                                        Box::new_in(acc, bump),
                                        Box::new_in(idx, bump),
                                    )
                                }
                            })
            }

        rule postfix_op() -> LaterPostfixOp<'bump, 'input>
            = _ "." name:ident() _ "(" _ args:(expr() ** (_ "," _)) _ ")" {
                LaterPostfixOp::MethodCall(name, args)
            }
            / _ "." name:ident() {
                LaterPostfixOp::Field(name)
            }
            / _ "(" _ args:(expr() ** (_ "," _)) _ ")" {
                LaterPostfixOp::Call(args)
            }
            / _ "[" _ idx:expr() _ "]" {
                LaterPostfixOp::Index(idx)
            }

        rule primary() -> Expr<'bump, 'input>
            = switch_expr()
            / if_expr()
            / while_expr()
            / for_expr()
            / bool_lit()
            / for_expr()
            / struct_lit()
            / type_variant()
            / dot_variant()
            / string_expr()
            / n:number() { Expr::Number(n) }
            / "(" _ e:expr() _ ")" { e }
            / name:ident() { Expr::Ident(name) }

        rule type_variant() -> Expr<'bump, 'input>
            = ty:ident() "." v:ident() {
                Expr::EnumVariant(ty, v)
            }

        rule dot_variant() -> Expr<'bump, 'input>
            = "." v:ident() { Expr::DotVariant(v) }

        rule struct_lit() -> Expr<'bump, 'input>
            = name:ident() _ "{" _
              fields:(struct_field() ** (_ "," _)) _ ","? _
              "}" {
                  Expr::StructLit(
                      name,
                      fields
                        .into_iter()
                        .map(|(label, value)| StructInitializer { label, value })
                        .collect()
                  )
              }

        rule struct_field() -> (&'input str, Expr<'bump, 'input>)
            = name:ident() _ ":" _ e:expr() { (name, e) }

        rule switch_expr() -> Expr<'bump, 'input>
            = "switch" _ "(" _ subject:expr() _ ")" _ "{" _
              arms:(switch_arm() ** (_ "," _)) _ ","? _
              "}" { Expr::Switch(Box::new_in(subject, bump), arms) }

        rule switch_arm() -> SwitchArm<'bump, 'input>
            = patterns:(switch_pattern() ++ (_ "|" _)) _ "=>" _ body:expr() {
                SwitchArm { patterns, body }
            }

        rule switch_pattern() -> Expr<'bump, 'input>
            = dot_variant() / expr()

        rule for_expr() -> Expr<'bump, 'input>
            = "for" __ binders:(ident() ** (_ "," _)) __ "in" __ iter:expr() _ body:block() {
                Expr::For { binders, iter: Box::new_in(iter, bump), body }
            }

        rule string_expr() -> Expr<'bump, 'input>
            = multiline_string()
            / quoted_string()

        rule quoted_string() -> Expr<'bump, 'input>
            = "\"" parts:string_part()* "\"" { Expr::Str(parts) }

        rule string_lit() -> &'input str
            = "\"" lit:$((!("\"" / "\\(") [_])+) "\"" { lit }

        rule string_part() -> StringPart<'bump, 'input>
            = "\\(" _ e:expr() fmt:(":" f:$([^ ')']+) { f })? _ ")" {
                StringPart::Interp { expr: Box::new_in(e, bump), format: fmt }
            }
            / lit:$((!("\"" / "\\(") [_])+) {
                StringPart::Literal(lit)
            }

        rule multiline_string() -> Expr<'bump, 'input>
            = lines:(multiline_line() ++ (_ )) {
                let mut parts = Vec::new();
                for (i, line_parts) in lines.into_iter().enumerate() {
                    if i > 0 {
                        parts.push(StringPart::Literal("\n"));
                    }
                    parts.extend(line_parts);
                }
                Expr::Str(parts)
            }

        rule multiline_line() -> Vec<StringPart<'bump, 'input>>
            = "\\\\" parts:multiline_part()* "\n"? { parts }

        rule multiline_part() -> StringPart<'bump, 'input>
            = "\\(" _ e:expr() fmt:(":" f:$([^ ')']+) { f })? _ ")" {
                StringPart::Interp { expr: Box::new_in(e, bump), format: fmt }
            }
            / lit:$((!("\n" / "\\(") [_])+) {
                StringPart::Literal(lit)
            }
    }
}

#[inline]
pub fn parse<'bump, 'input>(
    text: &'input str,
    bump: &'bump Bump,
) -> Result<Vec<TopLevelItem<'bump, 'input>>, ParseError<LineCol>> {
    kon_parser::program(text, bump)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) {
        let bump = Bump::new();
        match kon_parser::program(src, &bump) {
            Ok(_) => (),
            Err(e) => {
                panic!("parse error: {}", e);
            }
        }
    }

    #[test]
    fn test_simple() {
        let src = r#"
    let prefix = "Hello, ";
    let name = "World and " * 2;
    let name2 = "World";

    enum IceCream {
      .Chocolate,
      .Strawberry
    }

    fn [alloca] IceCream.to_string() -> string {
      switch (cream) {
        .Chocolate => "chocolate",
        .Strawberry => "strawberry"
      }
    }

    struct User {
      name: string
    }

    interface ToString {
      fn to_string(name: *const string.string);
    }
    "#;

        parse(src);
    }

    #[test]
    fn test_extern() {
        let src = r#"
        extern "C" {
            #include <stdio.h>
            fn blyatt => suka();
            fn "blyatt" => suka();
        }"#;

        parse(src);
    }
}

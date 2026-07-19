use peg::parser;

#[derive(Debug, Clone, uniffi::Enum)]
pub enum StringPart {
    Literal(String),
    Interp {
        expr: Box<Expr>,
        format: Option<String>,
    },
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum Expr {
    Number(f64),
    Ident(String),
    Str(Vec<StringPart>),
    EnumVariant(String, String), // Type.Variant
    DotVariant(String),          // .Variant
    Neg(Box<Expr>),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    MethodCall(Box<Expr>, String, Vec<Expr>),
    Field(Box<Expr>, String),
    Index(Box<Expr>, Box<Expr>),
    StructLit(String, Vec<StructInitializer>),
    Switch(Box<Expr>, Vec<SwitchArm>),
    For {
        binders: Vec<String>,
        iter: Box<Expr>,
        body: Vec<Stmt>,
    },
    Block(Vec<Stmt>),
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct StructInitializer {
    pub label: String,
    pub value: Expr,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwitchArm {
    pub patterns: Vec<Expr>,
    pub body: Expr,
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum Stmt {
    Let(String, Expr),
    Expr(Expr),
    Tail(Expr),
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct Param {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum Item {
    Let(String, Expr),
    Enum {
        name: String,
        variants: Vec<String>,
    },
    Struct {
        name: String,
        fields: Vec<Param>,
    },
    Fn {
        receiver: Option<Param>,
        name: String,
        params: Vec<Param>,
        body: Vec<Stmt>,
    },
    Interface {
        name: String,
        methods: Vec<InterfaceMethod>,
    },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct InterfaceMethod {
    pub name: String,
    pub params: Vec<Param>,
}

parser! {
    pub grammar kon_parser() for str {

        rule _() = quiet!{ ([' ' | '\t' | '\n' | '\r'] / comment())* }
        rule comment()
            = "///" (!['\n'][_])*
            / "//" (!['\n'][_])*

        rule __() = quiet!{ [' ' | '\t' | '\n' | '\r']+ }

        rule ident() -> String
            = quiet!{ s:$(['a'..='z' | 'A'..='Z' | '_'] ['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*) { s.to_string() } }
            / expected!("identifier")

        rule number() -> f64
            = n:$(['0'..='9']+ ("." ['0'..='9']+)?) { n.parse().unwrap() }

        pub rule program() -> Vec<Item>
            = _ items:(item() ** _) _ { items }

        rule item() -> Item
            = let_item()
            / enum_item()
            / struct_item()
            / fn_item()
            / interface_item()

        rule let_item() -> Item
            = "let" __ name:ident() _ "=" _ e:expr() _ ";" { Item::Let(name, e) }

        rule enum_item() -> Item
            = "enum" __ name:ident() _ "{" _
              variants:(enum_variant() ** (_ "," _)) _ ","? _
              "}" { Item::Enum { name, variants } }

        rule enum_variant() -> String
            = "." v:ident() { v }

        rule struct_item() -> Item
            = "struct" __ name:ident() _ "{" _
              fields:(param() ** (_ "," _)) _ ","? _
              "}" { Item::Struct { name, fields } }

        rule param() -> Param
            = name:ident() _ ":" _ ty:ident() { Param { name, ty } }

        rule fn_item() -> Item
            = "fn" __ recv:receiver()? name:ident() _
              "(" _ params:(param() ** (_ "," _)) _ ")" _
              body:block() {
                Item::Fn { receiver: recv, name, params, body }
              }

        rule receiver() -> Param
            = "(" _ p:param() _ ")" _ { p }

        rule interface_item() -> Item
            = "interface" __ name:ident() _ "{" _
              methods:(interface_method() ** _) _
              "}" { Item::Interface { name, methods } }

        rule interface_method() -> InterfaceMethod
            = "fn" __ name:ident() _
              "(" _ params:(param() ** (_ "," _)) _ ")" _ ";" {
                InterfaceMethod { name, params }
              }

        rule block() -> Vec<Stmt>
            = "{" _ stmts:(stmt() ** _) _ tail:expr()? _ "}" {
                let mut stmts = stmts;
                if let Some(e) = tail {
                    stmts.push(Stmt::Tail(e));
                }
                stmts
            }

        rule block_expr() -> Expr
            = switch_expr()

        rule stmt() -> Stmt
            = "let" __ name:ident() _ "=" _ e:expr() _ ";" { Stmt::Let(name, e) }
            / e:block_expr() _ ";"? { Stmt::Expr(e) }
            / e:expr() _ ";" { Stmt::Expr(e) }

        pub rule expr() -> Expr = precedence!{
            x:(@) _ "+" _ y:@ { Expr::Binary(Box::new(x), BinOp::Add, Box::new(y)) }
            x:(@) _ "-" _ y:@ { Expr::Binary(Box::new(x), BinOp::Sub, Box::new(y)) }
                --
            x:(@) _ "*" _ y:@ { Expr::Binary(Box::new(x), BinOp::Mul, Box::new(y)) }
            x:(@) _ "/" _ y:@ { Expr::Binary(Box::new(x), BinOp::Div, Box::new(y)) }
                --
            "-" e:@ { Expr::Neg(Box::new(e)) }
                --
            e:postfix() { e }
        }

        rule postfix() -> Expr
            = base:primary() ops:postfix_op()* {
                ops.into_iter().fold(base, |acc, op| op(acc))
            }

        rule postfix_op() -> Box<dyn Fn(Expr) -> Expr>
            = _ "." name:ident() _ "(" _ args:(expr() ** (_ "," _)) _ ")" {
                Box::new(move |recv| Expr::MethodCall(Box::new(recv), name.clone(), args.clone()))
            }
            / _ "." name:ident() {
                Box::new(move |recv| Expr::Field(Box::new(recv), name.clone()))
            }
            / _ "(" _ args:(expr() ** (_ "," _)) _ ")" {
                Box::new(move |recv| Expr::Call(Box::new(recv), args.clone()))
            }
            / _ "[" _ idx:expr() _ "]" {
                Box::new(move |recv| Expr::Index(Box::new(recv), Box::new(idx.clone())))
            }

        rule primary() -> Expr
            = switch_expr()
            / for_expr()
            / struct_lit()
            / type_variant()
            / dot_variant()
            / string_lit()
            / n:number() { Expr::Number(n) }
            / "(" _ e:expr() _ ")" { e }
            / name:ident() { Expr::Ident(name) }

        rule type_variant() -> Expr
            = ty:ident() "." v:ident() {
                Expr::EnumVariant(ty, v)
            }

        rule dot_variant() -> Expr
            = "." v:ident() { Expr::DotVariant(v) }

        rule struct_lit() -> Expr
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

        rule struct_field() -> (String, Expr)
            = name:ident() _ ":" _ e:expr() { (name, e) }

        rule switch_expr() -> Expr
            = "switch" _ "(" _ subject:expr() _ ")" _ "{" _
              arms:(switch_arm() ** (_ "," _)) _ ","? _
              "}" { Expr::Switch(Box::new(subject), arms) }

        rule switch_arm() -> SwitchArm
            = patterns:(switch_pattern() ++ (_ "|" _)) _ "=>" _ body:expr() {
                SwitchArm { patterns, body }
            }

        rule switch_pattern() -> Expr
            = dot_variant() / expr()

        rule for_expr() -> Expr
            = "for" __ binders:(ident() ** (_ "," _)) __ "in" __ iter:expr() _ body:block() {
                Expr::For { binders, iter: Box::new(iter), body }
            }

        rule string_lit() -> Expr
            = multiline_string()
            / quoted_string()

        rule quoted_string() -> Expr
            = "\"" parts:string_part()* "\"" { Expr::Str(parts) }

        rule string_part() -> StringPart
            = "\\(" _ e:expr() fmt:(":" f:$([^ ')']+) { f.to_string() })? _ ")" {
                StringPart::Interp { expr: Box::new(e), format: fmt }
            }
            / lit:$((!("\"" / "\\(") [_])+) {
                StringPart::Literal(lit.to_string())
            }

        rule multiline_string() -> Expr
            = lines:(multiline_line() ++ (_ )) {
                let mut parts = Vec::new();
                for (i, line_parts) in lines.into_iter().enumerate() {
                    if i > 0 {
                        parts.push(StringPart::Literal("\n".to_string()));
                    }
                    parts.extend(line_parts);
                }
                Expr::Str(parts)
            }

        rule multiline_line() -> Vec<StringPart>
            = "\\\\" parts:multiline_part()* "\n"? { parts }

        rule multiline_part() -> StringPart
            = "\\(" _ e:expr() fmt:(":" f:$([^ ')']+) { f.to_string() })? _ ")" {
                StringPart::Interp { expr: Box::new(e), format: fmt }
            }
            / lit:$((!("\n" / "\\(") [_])+) {
                StringPart::Literal(lit.to_string())
            }
    }
}

#[uniffi::export]
pub fn parse(text: &str) -> Vec<Item> {
    kon_parser::program(text).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let source = r#"
    let prefix = "Hello, ";
    let name = "World and " * 2;
    let name2 = "World";

    enum IceCream {
      .Chocolate,
      .Strawberry
    }

    fn (cream: IceCream) to_string() {
      switch (cream) {
        .Chocolate => "chocolate",
        .Strawberry => "strawberry"
      }

      10
    }

    struct User {
      name: string
    }

    fn (u: User) greet() {
      let a = (1 + 10) * 2;
      println("hello, \(u.name)! Number: \(a)");
    }

    interface ToString {
      fn to_string(name: string);
    }
    "#;

        match kon_parser::program(source) {
            Ok(_) => (),
            Err(e) => {
                panic!("parse error: {}", e);
            }
        }
    }
}

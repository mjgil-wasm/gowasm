use std::collections::HashMap;

use gowasm_parser::{
    AssignTarget, ConstGroupDecl, Expr, FunctionDecl, MapLiteralEntry, Parameter, Stmt,
    StructLiteralField,
};

use crate::types::substitute_type_params;

pub(super) fn instantiate_function_decl(
    template: &FunctionDecl,
    substitutions: &HashMap<String, String>,
    name: String,
) -> FunctionDecl {
    FunctionDecl {
        receiver: template
            .receiver
            .as_ref()
            .map(|receiver| substitute_parameter(receiver, substitutions)),
        name,
        type_params: Vec::new(),
        params: template
            .params
            .iter()
            .map(|parameter| substitute_parameter(parameter, substitutions))
            .collect(),
        result_types: template
            .result_types
            .iter()
            .map(|result_type| substitute_type_params(result_type, substitutions))
            .collect(),
        result_names: template.result_names.clone(),
        body: template
            .body
            .iter()
            .map(|stmt| substitute_stmt(stmt, substitutions))
            .collect(),
    }
}

fn substitute_parameter(
    parameter: &Parameter,
    substitutions: &HashMap<String, String>,
) -> Parameter {
    Parameter {
        name: parameter.name.clone(),
        typ: substitute_type_params(&parameter.typ, substitutions),
        variadic: parameter.variadic,
    }
}

fn expr_uses_type_params(expr: &Expr, substitutions: &HashMap<String, String>) -> bool {
    match expr {
        Expr::Ident(name) => substitutions.contains_key(name),
        Expr::Index { target, index } => {
            expr_uses_type_params(target, substitutions)
                || expr_uses_type_params(index, substitutions)
        }
        _ => false,
    }
}

fn substitute_type_arg_expr(expr: &Expr, substitutions: &HashMap<String, String>) -> Expr {
    match expr {
        Expr::Ident(name) => substitutions
            .get(name)
            .map(|name| Expr::Ident(name.clone()))
            .unwrap_or_else(|| Expr::Ident(name.clone())),
        Expr::Index { target, index } => Expr::Index {
            target: Box::new(substitute_type_arg_expr(target, substitutions)),
            index: Box::new(substitute_type_arg_expr(index, substitutions)),
        },
        _ => substitute_expr(expr, substitutions),
    }
}

fn substitute_stmt(stmt: &Stmt, substitutions: &HashMap<String, String>) -> Stmt {
    match stmt {
        Stmt::Expr(expr) => Stmt::Expr(substitute_expr(expr, substitutions)),
        Stmt::VarDecl { name, typ, value } => Stmt::VarDecl {
            name: name.clone(),
            typ: typ
                .as_ref()
                .map(|typ| substitute_type_params(typ, substitutions)),
            value: value
                .as_ref()
                .map(|value| substitute_expr(value, substitutions)),
        },
        Stmt::ConstDecl {
            name,
            typ,
            value,
            iota,
        } => Stmt::ConstDecl {
            name: name.clone(),
            typ: typ
                .as_ref()
                .map(|typ| substitute_type_params(typ, substitutions)),
            value: substitute_expr(value, substitutions),
            iota: *iota,
        },
        Stmt::ConstGroup { decls } => Stmt::ConstGroup {
            decls: decls
                .iter()
                .map(|decl| ConstGroupDecl {
                    name: decl.name.clone(),
                    typ: decl
                        .typ
                        .as_ref()
                        .map(|typ| substitute_type_params(typ, substitutions)),
                    value: substitute_expr(&decl.value, substitutions),
                    iota: decl.iota,
                })
                .collect(),
        },
        Stmt::ShortVarDecl { name, value } => Stmt::ShortVarDecl {
            name: name.clone(),
            value: substitute_expr(value, substitutions),
        },
        Stmt::ShortVarDeclPair {
            first,
            second,
            value,
        } => Stmt::ShortVarDeclPair {
            first: first.clone(),
            second: second.clone(),
            value: substitute_expr(value, substitutions),
        },
        Stmt::ShortVarDeclTriple {
            first,
            second,
            third,
            value,
        } => Stmt::ShortVarDeclTriple {
            first: first.clone(),
            second: second.clone(),
            third: third.clone(),
            value: substitute_expr(value, substitutions),
        },
        Stmt::ShortVarDeclQuad {
            first,
            second,
            third,
            fourth,
            value,
        } => Stmt::ShortVarDeclQuad {
            first: first.clone(),
            second: second.clone(),
            third: third.clone(),
            fourth: fourth.clone(),
            value: substitute_expr(value, substitutions),
        },
        Stmt::ShortVarDeclList { names, values } => Stmt::ShortVarDeclList {
            names: names.clone(),
            values: values
                .iter()
                .map(|value| substitute_expr(value, substitutions))
                .collect(),
        },
        Stmt::Assign { target, value } => Stmt::Assign {
            target: substitute_assign_target(target, substitutions),
            value: substitute_expr(value, substitutions),
        },
        Stmt::AssignPair {
            first,
            second,
            value,
        } => Stmt::AssignPair {
            first: substitute_assign_target(first, substitutions),
            second: substitute_assign_target(second, substitutions),
            value: substitute_expr(value, substitutions),
        },
        Stmt::AssignTriple {
            first,
            second,
            third,
            value,
        } => Stmt::AssignTriple {
            first: substitute_assign_target(first, substitutions),
            second: substitute_assign_target(second, substitutions),
            third: substitute_assign_target(third, substitutions),
            value: substitute_expr(value, substitutions),
        },
        Stmt::AssignQuad {
            first,
            second,
            third,
            fourth,
            value,
        } => Stmt::AssignQuad {
            first: substitute_assign_target(first, substitutions),
            second: substitute_assign_target(second, substitutions),
            third: substitute_assign_target(third, substitutions),
            fourth: substitute_assign_target(fourth, substitutions),
            value: substitute_expr(value, substitutions),
        },
        Stmt::AssignList { targets, values } => Stmt::AssignList {
            targets: targets
                .iter()
                .map(|target| substitute_assign_target(target, substitutions))
                .collect(),
            values: values
                .iter()
                .map(|value| substitute_expr(value, substitutions))
                .collect(),
        },
        Stmt::Increment { name } => Stmt::Increment { name: name.clone() },
        Stmt::Decrement { name } => Stmt::Decrement { name: name.clone() },
        Stmt::If {
            init,
            condition,
            then_body,
            else_body,
        } => Stmt::If {
            init: init
                .as_ref()
                .map(|init| Box::new(substitute_stmt(init, substitutions))),
            condition: substitute_expr(condition, substitutions),
            then_body: then_body
                .iter()
                .map(|stmt| substitute_stmt(stmt, substitutions))
                .collect(),
            else_body: else_body.as_ref().map(|body| {
                body.iter()
                    .map(|stmt| substitute_stmt(stmt, substitutions))
                    .collect()
            }),
        },
        Stmt::For {
            init,
            condition,
            post,
            body,
        } => Stmt::For {
            init: init
                .as_ref()
                .map(|init| Box::new(substitute_stmt(init, substitutions))),
            condition: condition
                .as_ref()
                .map(|condition| substitute_expr(condition, substitutions)),
            post: post
                .as_ref()
                .map(|post| Box::new(substitute_stmt(post, substitutions))),
            body: body
                .iter()
                .map(|stmt| substitute_stmt(stmt, substitutions))
                .collect(),
        },
        Stmt::RangeFor {
            key,
            value,
            assign,
            expr,
            body,
        } => Stmt::RangeFor {
            key: key.clone(),
            value: value.clone(),
            assign: *assign,
            expr: substitute_expr(expr, substitutions),
            body: body
                .iter()
                .map(|stmt| substitute_stmt(stmt, substitutions))
                .collect(),
        },
        Stmt::Switch {
            init,
            expr,
            cases,
            default,
            default_index,
            default_fallthrough,
        } => Stmt::Switch {
            init: init
                .as_ref()
                .map(|init| Box::new(substitute_stmt(init, substitutions))),
            expr: expr
                .as_ref()
                .map(|expr| substitute_expr(expr, substitutions)),
            cases: cases
                .iter()
                .map(|case| gowasm_parser::SwitchCase {
                    expressions: case
                        .expressions
                        .iter()
                        .map(|expr| substitute_expr(expr, substitutions))
                        .collect(),
                    body: case
                        .body
                        .iter()
                        .map(|stmt| substitute_stmt(stmt, substitutions))
                        .collect(),
                    fallthrough: case.fallthrough,
                })
                .collect(),
            default: default.as_ref().map(|body| {
                body.iter()
                    .map(|stmt| substitute_stmt(stmt, substitutions))
                    .collect()
            }),
            default_index: *default_index,
            default_fallthrough: *default_fallthrough,
        },
        Stmt::TypeSwitch {
            init,
            binding,
            expr,
            cases,
            default,
            default_index,
        } => Stmt::TypeSwitch {
            init: init
                .as_ref()
                .map(|init| Box::new(substitute_stmt(init, substitutions))),
            binding: binding.clone(),
            expr: substitute_expr(expr, substitutions),
            cases: cases
                .iter()
                .map(|case| gowasm_parser::TypeSwitchCase {
                    types: case
                        .types
                        .iter()
                        .map(|typ| substitute_type_params(typ, substitutions))
                        .collect(),
                    body: case
                        .body
                        .iter()
                        .map(|stmt| substitute_stmt(stmt, substitutions))
                        .collect(),
                })
                .collect(),
            default: default.as_ref().map(|body| {
                body.iter()
                    .map(|stmt| substitute_stmt(stmt, substitutions))
                    .collect()
            }),
            default_index: *default_index,
        },
        Stmt::Select { cases, default } => Stmt::Select {
            cases: cases
                .iter()
                .map(|case| gowasm_parser::SelectCase {
                    stmt: substitute_stmt(&case.stmt, substitutions),
                    body: case
                        .body
                        .iter()
                        .map(|stmt| substitute_stmt(stmt, substitutions))
                        .collect(),
                })
                .collect(),
            default: default.as_ref().map(|body| {
                body.iter()
                    .map(|stmt| substitute_stmt(stmt, substitutions))
                    .collect()
            }),
        },
        Stmt::Send { chan, value } => Stmt::Send {
            chan: substitute_expr(chan, substitutions),
            value: substitute_expr(value, substitutions),
        },
        Stmt::Go { call } => Stmt::Go {
            call: substitute_expr(call, substitutions),
        },
        Stmt::Defer { call } => Stmt::Defer {
            call: substitute_expr(call, substitutions),
        },
        Stmt::Labeled { label, stmt } => Stmt::Labeled {
            label: label.clone(),
            stmt: Box::new(substitute_stmt(stmt, substitutions)),
        },
        Stmt::Break { label } => Stmt::Break {
            label: label.clone(),
        },
        Stmt::Continue { label } => Stmt::Continue {
            label: label.clone(),
        },
        Stmt::Return(values) => Stmt::Return(
            values
                .iter()
                .map(|value| substitute_expr(value, substitutions))
                .collect(),
        ),
    }
}

fn substitute_assign_target(
    target: &AssignTarget,
    substitutions: &HashMap<String, String>,
) -> AssignTarget {
    match target {
        AssignTarget::Ident(name) => AssignTarget::Ident(name.clone()),
        AssignTarget::Deref { target } => AssignTarget::Deref {
            target: target.clone(),
        },
        AssignTarget::DerefSelector { target, field } => AssignTarget::DerefSelector {
            target: target.clone(),
            field: field.clone(),
        },
        AssignTarget::DerefIndex { target, index } => AssignTarget::DerefIndex {
            target: target.clone(),
            index: substitute_expr(index, substitutions),
        },
        AssignTarget::Selector { receiver, field } => AssignTarget::Selector {
            receiver: receiver.clone(),
            field: field.clone(),
        },
        AssignTarget::Index { target, index } => AssignTarget::Index {
            target: target.clone(),
            index: substitute_expr(index, substitutions),
        },
    }
}

fn substitute_expr(expr: &Expr, substitutions: &HashMap<String, String>) -> Expr {
    match expr {
        Expr::Ident(name) => Expr::Ident(name.clone()),
        Expr::NilLiteral => Expr::NilLiteral,
        Expr::BoolLiteral(value) => Expr::BoolLiteral(*value),
        Expr::IntLiteral(value) => Expr::IntLiteral(*value),
        Expr::FloatLiteral(bits) => Expr::FloatLiteral(*bits),
        Expr::StringLiteral(value) => Expr::StringLiteral(value.clone()),
        Expr::Unary { op, expr } => Expr::Unary {
            op: *op,
            expr: Box::new(substitute_expr(expr, substitutions)),
        },
        Expr::Binary { left, op, right } => Expr::Binary {
            left: Box::new(substitute_expr(left, substitutions)),
            op: *op,
            right: Box::new(substitute_expr(right, substitutions)),
        },
        Expr::ArrayLiteral {
            len,
            element_type,
            elements,
        } => Expr::ArrayLiteral {
            len: *len,
            element_type: substitute_type_params(element_type, substitutions),
            elements: elements
                .iter()
                .map(|element| substitute_expr(element, substitutions))
                .collect(),
        },
        Expr::SliceLiteral {
            element_type,
            elements,
        } => Expr::SliceLiteral {
            element_type: substitute_type_params(element_type, substitutions),
            elements: elements
                .iter()
                .map(|element| substitute_expr(element, substitutions))
                .collect(),
        },
        Expr::SliceConversion { element_type, expr } => Expr::SliceConversion {
            element_type: substitute_type_params(element_type, substitutions),
            expr: Box::new(substitute_expr(expr, substitutions)),
        },
        Expr::MapLiteral {
            key_type,
            value_type,
            entries,
        } => Expr::MapLiteral {
            key_type: substitute_type_params(key_type, substitutions),
            value_type: substitute_type_params(value_type, substitutions),
            entries: entries
                .iter()
                .map(|entry| MapLiteralEntry {
                    key: substitute_expr(&entry.key, substitutions),
                    value: substitute_expr(&entry.value, substitutions),
                })
                .collect(),
        },
        Expr::StructLiteral { type_name, fields } => Expr::StructLiteral {
            type_name: substitute_type_params(type_name, substitutions),
            fields: fields
                .iter()
                .map(|field| StructLiteralField {
                    name: field.name.clone(),
                    value: substitute_expr(&field.value, substitutions),
                })
                .collect(),
        },
        Expr::Index { target, index } => Expr::Index {
            target: Box::new(substitute_expr(target, substitutions)),
            index: Box::new(substitute_expr(index, substitutions)),
        },
        Expr::SliceExpr { target, low, high } => Expr::SliceExpr {
            target: Box::new(substitute_expr(target, substitutions)),
            low: low
                .as_ref()
                .map(|low| Box::new(substitute_expr(low, substitutions))),
            high: high
                .as_ref()
                .map(|high| Box::new(substitute_expr(high, substitutions))),
        },
        Expr::Selector { receiver, field } => Expr::Selector {
            receiver: Box::new(substitute_expr(receiver, substitutions)),
            field: field.clone(),
        },
        Expr::TypeAssert {
            expr,
            asserted_type,
        } => Expr::TypeAssert {
            expr: Box::new(substitute_expr(expr, substitutions)),
            asserted_type: substitute_type_params(asserted_type, substitutions),
        },
        Expr::New { type_name } => Expr::New {
            type_name: substitute_type_params(type_name, substitutions),
        },
        Expr::Make { type_name, args } => Expr::Make {
            type_name: substitute_type_params(type_name, substitutions),
            args: args
                .iter()
                .map(|arg| substitute_expr(arg, substitutions))
                .collect(),
        },
        Expr::FunctionLiteral {
            params,
            result_types,
            body,
        } => Expr::FunctionLiteral {
            params: params
                .iter()
                .map(|parameter| substitute_parameter(parameter, substitutions))
                .collect(),
            result_types: result_types
                .iter()
                .map(|result_type| substitute_type_params(result_type, substitutions))
                .collect(),
            body: body
                .iter()
                .map(|stmt| substitute_stmt(stmt, substitutions))
                .collect(),
        },
        Expr::Call {
            callee,
            type_args,
            args,
        } => Expr::Call {
            callee: Box::new(match callee.as_ref() {
                Expr::Index { target, index } if expr_uses_type_params(index, substitutions) => {
                    Expr::Index {
                        target: Box::new(substitute_expr(target, substitutions)),
                        index: Box::new(substitute_type_arg_expr(index, substitutions)),
                    }
                }
                _ => substitute_expr(callee, substitutions),
            }),
            type_args: type_args
                .iter()
                .map(|type_arg| substitute_type_params(type_arg, substitutions))
                .collect(),
            args: args
                .iter()
                .map(|arg| substitute_expr(arg, substitutions))
                .collect(),
        },
        Expr::Spread { expr } => Expr::Spread {
            expr: Box::new(substitute_expr(expr, substitutions)),
        },
    }
}

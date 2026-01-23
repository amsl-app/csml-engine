use crate::data::ast::{DoType, Expr, Function, IfStatement, Interval, ObjectType};

#[must_use]
pub fn interval_from_expr(expr: &Expr) -> Interval {
    match expr {
        Expr::Scope {
            range: range_interval,
            ..
        }
        | Expr::ComplexLiteral(_, range_interval)
        | Expr::MapExpr {
            interval: range_interval,
            ..
        }
        | Expr::VecExpr(_, range_interval)
        | Expr::ForEachExpr(_, _, _, _, range_interval)
        | Expr::WhileExpr(_, _, range_interval) => *range_interval,
        Expr::ObjectExpr(fnexpr) => interval_from_reserved_fn(fnexpr),
        Expr::InfixExpr(_, expr, _)
        | Expr::PostfixExpr(_, expr)
        | Expr::PathExpr { literal: expr, .. } => interval_from_expr(expr), // RangeInterval ?
        Expr::IdentExpr(ident) => ident.interval,
        Expr::LitExpr { literal, .. } => literal.interval,
        Expr::IfExpr(ifstmt) => interval_from_if_stmt(ifstmt),
    }
}

#[must_use]
pub fn interval_from_if_stmt(ifstmt: &IfStatement) -> Interval {
    match ifstmt {
        IfStatement::IfStmt { cond, .. } => interval_from_expr(cond),
        IfStatement::ElseStmt(_e, range_interval) => *range_interval,
    }
}

#[must_use]
pub fn interval_from_reserved_fn(reserved_fn: &ObjectType) -> Interval {
    match reserved_fn {
        ObjectType::Goto(_, interval)
        | ObjectType::Previous(_, interval)
        | ObjectType::BuiltIn(Function { interval, .. })
        | ObjectType::Debug(_, interval)
        | ObjectType::Log { interval, .. }
        | ObjectType::Hold(interval)
        | ObjectType::HoldSecure(interval)
        | ObjectType::Forget(_, interval)
        | ObjectType::Break(interval)
        | ObjectType::Continue(interval) => *interval,
        ObjectType::Use(expr)
        | ObjectType::Do(DoType::Update(_, expr, ..) | DoType::Exec(expr))
        | ObjectType::Say(expr)
        | ObjectType::Return(expr)
        | ObjectType::Assign(_, expr, ..) => interval_from_expr(expr),
        ObjectType::As(ident, ..) | ObjectType::Remember(ident, ..) => ident.interval,
    }
}

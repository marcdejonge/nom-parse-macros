use proc_macro2::Span;
use quote::quote_spanned;
use syn::{parse, parse_str, Expr, ExprArray, ExprLit, Lit, Path, Stmt};
use syn::__private::IntoSpans;

const NOM_FUNCTIONS: phf::Map<&'static str, &'static str> = phf::phf_map! {
    // From the nom::branch module
    "alt" => "nom::branch::alt",
    // From the nom::bytes::complete module
    // "tag" => "nom::bytes::complete::tag", tag is no longer explicitly supported, we use a raw string literal instead
    "tag_no_case" => "nom::bytes::complete::tag_no_case",
    "is_not" => "nom::bytes::complete::is_not",
    "is_a" => "nom::bytes::complete::is_a",
    "take_while" => "nom::bytes::complete::take_while",
    "take_while1" => "nom::bytes::complete::take_while1",
    "take_while_m_n" => "nom::bytes::complete::take_while_m_n",
    "take_till" => "nom::bytes::complete::take_till",
    "take_till1" => "nom::bytes::complete::take_till1",
    "take" => "nom::bytes::complete::take",
    "take_until" => "nom::bytes::complete::take_until",
    "take_until1" => "nom::bytes::complete::take_until1",
    "escaped" => "nom::bytes::complete::escaped",
    "escaped_transform" => "nom::bytes::complete::escaped_transform",
    // From the nom::character::complete module
    "char" => "nom::character::complete::char",
    "satisfy" => "nom::character::complete::satisfy",
    "one_of" => "nom::character::complete::one_of",
    "none_of" => "nom::character::complete::none_of",
    "crlf" => "nom::character::complete::crlf",
    "not_line_ending" => "nom::character::complete::not_line_ending",
    "line_ending" => "nom::character::complete::line_ending",
    "newline" => "nom::character::complete::newline",
    "tab" => "nom::character::complete::tab",
    "anychar" => "nom::character::complete::anychar",
    "alpha0" => "nom::character::complete::alpha0",
    "alpha1" => "nom::character::complete::alpha1",
    "digit0" => "nom::character::complete::digit0",
    "digit1" => "nom::character::complete::digit1",
    "hex_digit0" => "nom::character::complete::hex_digit0",
    "hex_digit1" => "nom::character::complete::hex_digit1",
    "oct_digit0" => "nom::character::complete::oct_digit0",
    "oct_digit1" => "nom::character::complete::oct_digit1",
    "alphanumeric0" => "nom::character::complete::alphanumeric0",
    "alphanumeric1" => "nom::character::complete::alphanumeric1",
    "space0" => "nom::character::complete::space0",
    "space1" => "nom::character::complete::space1",
    "multispace0" => "nom::character::complete::multispace0",
    "multispace1" => "nom::character::complete::multispace1",
    "sign" => "nom::character::complete::sign",
    // From the nom::combinator module
    "rest" => "nom::combinator::rest",
    "rest_len" => "nom::combinator::rest_len",
    "map" => "nom::combinator::map",
    "map_res" => "nom::combinator::map_res",
    "map_opt" => "nom::combinator::map_opt",
    "map_parser" => "nom::combinator::map_parser",
    "flat_map" => "nom::combinator::flat_map",
    "opt" => "nom::combinator::opt",
    "cond" => "nom::combinator::cond",
    "peek" => "nom::combinator::peek",
    "eof" => "nom::combinator::eof",
    "complete" => "nom::combinator::complete",
    "all_consuming" => "nom::combinator::all_consuming",
    "verify" => "nom::combinator::verify",
    "value" => "nom::combinator::value",
    "not" => "nom::combinator::not",
    "recognize" => "nom::combinator::recognize",
    "consumed" => "nom::combinator::consumed",
    "cut" => "nom::combinator::cut",
    "into" => "nom::combinator::into",
    "success" => "nom::combinator::success",
    "fail" => "nom::combinator::fail",
    // From the nom::multi module
    "many0" => "nom::multi::many0",
    "many1" => "nom::multi::many1",
    "many_till" => "nom::multi::many_till",
    "separated_list0" => "nom::multi::separated_list0",
    "separated_list1" => "nom::multi::separated_list1",
    "many_m_n" => "nom::multi::many_m_n",
    "many0_count" => "nom::multi::many0_count",
    "many1_count" => "nom::multi::many1_count",
    "count" => "nom::multi::count",
    "fill" => "nom::multi::fill",
    "fold_many0" => "nom::multi::fold_many0",
    "fold_many1" => "nom::multi::fold_many1",
    "fold_many_m_n" => "nom::multi::fold_many_m_n",
    "length_data" => "nom::multi::length_data",
    "length_value" => "nom::multi::length_value",
    "length_count" => "nom::multi::length_count",
    // From the nom::sequence module
    "pair" => "nom::sequence::pair",
    "preceded" => "nom::sequence::preceded",
    "terminated" => "nom::sequence::terminated",
    "separated_pair" => "nom::sequence::separated_pair",
    "delimited" => "nom::sequence::delimited",
    "tuple" => "nom::sequence::tuple",
};

pub fn apply_nom_namespaces(expr: &mut Expr) {
    match expr {
        Expr::Array(array) => {
            array.elems.iter_mut().for_each(|elem| {
                apply_nom_namespaces(elem);
            });
        }
        Expr::Assign(assign) => {
            apply_nom_namespaces(&mut assign.right);
        }
        Expr::Async(async_expr) => {
            async_expr.block.stmts.iter_mut().for_each(|stmt| {
                apply_nom_namespaces_stmt(stmt);
            });
        }
        Expr::Await(block) => {
            apply_nom_namespaces(&mut block.base);
        }
        Expr::Binary(bin_expr) => {
            apply_nom_namespaces(&mut bin_expr.left);
            apply_nom_namespaces(&mut bin_expr.right);
        }
        Expr::Block(block) => {
            block.block.stmts.iter_mut().for_each(|stmt| {
                apply_nom_namespaces_stmt(stmt);
            });
        }
        Expr::Call(call) => {
            apply_nom_namespaces(&mut call.func);
            call.args.iter_mut().for_each(|arg| {
                apply_nom_namespaces(arg);
            });
        }
        Expr::Cast(cast) => {
            apply_nom_namespaces(&mut cast.expr);
        }
        Expr::Closure(closure) => {
            apply_nom_namespaces(&mut closure.body);
        }
        Expr::Field(field) => {
            apply_nom_namespaces(&mut field.base);
        }
        Expr::ForLoop(for_loop) => {
            apply_nom_namespaces(&mut for_loop.expr);
            for_loop.body.stmts.iter_mut().for_each(|stmt| {
                apply_nom_namespaces_stmt(stmt);
            });
        }
        Expr::Group(group) => {
            apply_nom_namespaces(&mut group.expr);
        }
        Expr::If(if_expr) => {
            apply_nom_namespaces(&mut if_expr.cond);
            if_expr.then_branch.stmts.iter_mut().for_each(|stmt| {
                apply_nom_namespaces_stmt(stmt);
            });
            if let Some((_, else_branch)) = &mut if_expr.else_branch {
                apply_nom_namespaces(else_branch);
            }
        }
        Expr::Index(index) => {
            apply_nom_namespaces(&mut index.expr);
            apply_nom_namespaces(&mut index.index);
        }
        Expr::Let(let_expr) => {
            apply_nom_namespaces(&mut let_expr.expr);
        }
        Expr::Lit(lit_expr) => match &lit_expr.lit {
            Lit::Str(value) => {
                *expr = generate_tag_expression(value.value().as_bytes(), value.span());
            }
            Lit::ByteStr(value) => {
                *expr = generate_tag_expression(&value.value(), value.span());
            }
            Lit::Byte(value) => {
                *expr = generate_tag_expression(&[value.value()], value.span());
            }
            Lit::Char(value) => {
                *expr = generate_tag_expression(value.value().to_string().as_bytes(), value.span());
            }
            _ => {}
        },
        Expr::Loop(loop_expr) => {
            loop_expr.body.stmts.iter_mut().for_each(|stmt| {
                apply_nom_namespaces_stmt(stmt);
            });
        }
        Expr::Match(match_expr) => {
            apply_nom_namespaces(&mut match_expr.expr);
            match_expr.arms.iter_mut().for_each(|arm| {
                apply_nom_namespaces(arm.body.as_mut());
            });
        }
        Expr::MethodCall(call) => {
            apply_nom_namespaces(&mut call.receiver);
            call.args.iter_mut().for_each(|arg| {
                apply_nom_namespaces(arg);
            });
        }
        Expr::Paren(paren) => {
            apply_nom_namespaces(&mut paren.expr);
        }
        Expr::Path(path_expr) => {
            let segments = &path_expr.path.segments;
            if segments.len() == 1 {
                let ident = &segments[0].ident;
                if let Some(nom_path) = NOM_FUNCTIONS.get(ident.to_string().as_str()) {
                    path_expr.path.segments = parse_str::<Path>(nom_path).unwrap().segments;
                }
            }
        }
        Expr::Range(range) => {
            if let Some(start) = &mut range.start {
                apply_nom_namespaces(start);
            }
            if let Some(end) = &mut range.end {
                apply_nom_namespaces(end);
            }
        }
        Expr::RawAddr(raw_expr) => {
            apply_nom_namespaces(&mut raw_expr.expr);
        }
        Expr::Reference(ref_expr) => {
            apply_nom_namespaces(&mut ref_expr.expr);
        }
        Expr::Repeat(repeat_expr) => {
            apply_nom_namespaces(&mut repeat_expr.expr);
        }
        Expr::Return(return_expr) => {
            if let Some(expr) = &mut return_expr.expr {
                apply_nom_namespaces(expr);
            }
        }
        Expr::Struct(struct_expr) => {
            struct_expr.fields.iter_mut().for_each(|field| {
                apply_nom_namespaces(&mut field.expr);
            });
        }
        Expr::Try(try_expr) => {
            apply_nom_namespaces(&mut try_expr.expr);
        }
        Expr::TryBlock(try_block) => {
            try_block.block.stmts.iter_mut().for_each(|stmt| {
                apply_nom_namespaces_stmt(stmt);
            });
        }
        Expr::Tuple(tuple_expr) => {
            tuple_expr.elems.iter_mut().for_each(|elem| {
                apply_nom_namespaces(elem);
            });
        }
        Expr::Unary(unary_expr) => {
            apply_nom_namespaces(&mut unary_expr.expr);
        }
        Expr::Unsafe(unsafe_expr) => {
            unsafe_expr.block.stmts.iter_mut().for_each(|stmt| {
                apply_nom_namespaces_stmt(stmt);
            });
        }
        Expr::While(while_expr) => {
            apply_nom_namespaces(&mut while_expr.cond);
            while_expr.body.stmts.iter_mut().for_each(|stmt| {
                apply_nom_namespaces_stmt(stmt);
            });
        }
        Expr::Yield(yield_expr) => {
            if let Some(expr) = &mut yield_expr.expr {
                apply_nom_namespaces(expr);
            }
        }
        _ => {}
    }
}

pub fn generate_tag_expression(value: &[u8], span: Span) -> Expr {
    let mut array = parse_str::<ExprArray>(format!("{:?}", value).as_str()).unwrap();
    array.bracket_token.span = span.into_spans();
    array.elems.iter_mut().for_each(|elem| {
        if let Expr::Lit(ExprLit { lit, .. }) = elem {
            lit.set_span(span);
        }
    });
    parse::<Expr>(quote_spanned! { span => nom::bytes::complete::tag(#array.as_ref()) }.into())
        .unwrap()
}

fn apply_nom_namespaces_stmt(statement: &mut Stmt) {
    match statement {
        Stmt::Local(local) => {
            local.init.iter_mut().for_each(|init| {
                apply_nom_namespaces(init.expr.as_mut());
            });
        }
        Stmt::Item(_) => {}
        Stmt::Expr(expr, ..) => {
            apply_nom_namespaces(expr);
        }
        Stmt::Macro(_) => {}
    }
}

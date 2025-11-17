use crate::jmespath_ast::*;
use nom::{
    IResult, 
    bytes::complete::{tag, take_while1, escaped},
    character::complete::{digit1, alpha1, multispace0, char, one_of},
    combinator::{map_res, map, opt, recognize},
    branch::alt,
    multi::separated_list1,
    sequence::{preceded, delimited, tuple},
};

// ============================================================================
// BASIC LITERALS
// ============================================================================

/// Parse Integer: 123, -456
fn parse_int(input: &str) -> IResult<&str, JmesPathExpr> {
    let (rest, sign) = opt(char('-'))(input)?;
    let (rest, num) = map_res(digit1, |s: &str| s.parse::<i32>())(rest)?;
    let value = if sign.is_some() { -num } else { num };
    Ok((rest, JmesPathExpr::ConstInt(value)))
}

/// Parse Float: 123.45, -67.89
fn parse_float(input: &str) -> IResult<&str, JmesPathExpr> {
    let (rest, sign) = opt(char('-'))(input)?;
    let (rest, num_str) = recognize(tuple((
        digit1,
        char('.'),
        digit1,
    )))(rest)?;
    let value: f64 = num_str.parse().unwrap();
    let final_value = if sign.is_some() { -value } else { value };
    Ok((rest, JmesPathExpr::ConstFloat(final_value)))
}

/// Parse Boolean: true, false
fn parse_bool(input: &str) -> IResult<&str, JmesPathExpr> {
    alt((
        map(tag("true"), |_| JmesPathExpr::ConstBool(true)),
        map(tag("false"), |_| JmesPathExpr::ConstBool(false)),
    ))(input)
}

/// Parse String: "hello", "world\"test"
fn parse_string(input: &str) -> IResult<&str, JmesPathExpr> {
    let (rest, s) = delimited(
        char('"'),
        escaped(
            take_while1(|c| c != '"' && c != '\\'),
            '\\',
            one_of("\"\\nrt")
        ),
        char('"')
    )(input)?;
    Ok((rest, JmesPathExpr::ConstString(s.to_string())))
}

/// Parse String literal with single quotes: 'hello'
fn parse_string_single(input: &str) -> IResult<&str, JmesPathExpr> {
    let (rest, s) = delimited(
        char('\''),
        escaped(
            take_while1(|c| c != '\'' && c != '\\'),
            '\\',
            one_of("'\\nrt")
        ),
        char('\'')
    )(input)?;
    Ok((rest, JmesPathExpr::ConstString(s.to_string())))
}

// ============================================================================
// PATHS
// ============================================================================

/// Parse identifier: abc, field_name, field123
fn parse_identifier(input: &str) -> IResult<&str, String> {
    let (rest, first) = alpha1(input)?;
    let (rest, remainder) = opt(take_while1(|c: char| c.is_alphanumeric() || c == '_'))(rest)?;
    let mut result = first.to_string();
    if let Some(rem) = remainder {
        result.push_str(rem);
    }
    Ok((rest, result))
}

/// Parse path: a.b.c or just a
fn parse_path(input: &str) -> IResult<&str, JmesPathExpr> {
    let (rest, parts) = separated_list1(tag("."), parse_identifier)(input)?;
    Ok((rest, JmesPathExpr::Path(parts)))
}

// ============================================================================
// FUNCTION CALLS
// ============================================================================

/// Parse function call: length(array), contains(str, 'test')
fn parse_function(input: &str) -> IResult<&str, JmesPathExpr> {
    let (rest, func_name) = parse_identifier(input)?;
    let (rest, _) = multispace0(rest)?;
    let (rest, _) = char('(')(rest)?;
    let (rest, _) = multispace0(rest)?;
    
    // Parse arguments (comma-separated expressions)
    let (rest, args) = if let Ok((rest2, _)) = char::<_, nom::error::Error<&str>>(')')(rest) {
        // No arguments
        (rest2, vec![])
    } else {
        let (rest, args) = separated_list1(
            delimited(multispace0, char(','), multispace0),
            parse_logical_or
        )(rest)?;
        let (rest, _) = multispace0(rest)?;
        let (rest, _) = char(')')(rest)?;
        (rest, args)
    };
    
    Ok((rest, JmesPathExpr::Func { name: func_name, args }))
}

// ============================================================================
// ARRAY OPERATIONS
// ============================================================================

/// Parse array index: [0], [5]
fn parse_index(input: &str) -> IResult<&str, i32> {
    let (rest, _) = char('[')(input)?;
    let (rest, _) = multispace0(rest)?;
    let (rest, sign) = opt(char('-'))(rest)?;
    let (rest, num) = map_res(digit1, |s: &str| s.parse::<i32>())(rest)?;
    let (rest, _) = multispace0(rest)?;
    let (rest, _) = char(']')(rest)?;
    let value = if sign.is_some() { -num } else { num };
    Ok((rest, value))
}

/// Parse array slice: [0:5], [::2], [1:10:2]
fn parse_slice(input: &str) -> IResult<&str, (Option<i32>, Option<i32>, Option<i32>)> {
    let (rest, _) = char('[')(input)?;
    let (rest, _) = multispace0(rest)?;
    
    // Parse start (optional)
    let (rest, start) = opt(map_res(recognize(tuple((opt(char('-')), digit1))), |s: &str| s.parse::<i32>()))(rest)?;
    let (rest, _) = multispace0(rest)?;
    let (rest, _) = char(':')(rest)?;
    let (rest, _) = multispace0(rest)?;
    
    // Parse stop (optional)
    let (rest, stop) = opt(map_res(recognize(tuple((opt(char('-')), digit1))), |s: &str| s.parse::<i32>()))(rest)?;
    let (rest, _) = multispace0(rest)?;
    
    // Parse step (optional)
    let (rest, step) = if let Ok((rest2, _)) = char::<_, nom::error::Error<&str>>(':')(rest) {
        let (rest3, _) = multispace0(rest2)?;
        let (rest4, s) = opt(map_res(recognize(tuple((opt(char('-')), digit1))), |s: &str| s.parse::<i32>()))(rest3)?;
        (rest4, s)
    } else {
        (rest, None)
    };
    
    let (rest, _) = multispace0(rest)?;
    let (rest, _) = char(']')(rest)?;
    
    Ok((rest, (start, stop, step)))
}

/// Parse projection: [*]
fn parse_projection_wildcard(input: &str) -> IResult<&str, ()> {
    let (rest, _) = tag("[*]")(input)?;
    Ok((rest, ()))
}

/// Parse flatten: []
fn parse_flatten_bracket(input: &str) -> IResult<&str, ()> {
    let (rest, _) = tag("[]")(input)?;
    Ok((rest, ()))
}

// ============================================================================
// MULTI-SELECT
// ============================================================================

/// Parse multi-select hash: {key1: expr1, key2: expr2}
fn parse_multi_select_hash(input: &str) -> IResult<&str, JmesPathExpr> {
    let (rest, _) = char('{')(input)?;
    let (rest, _) = multispace0(rest)?;
    
    // Parse key-value pairs
    let (rest, pairs) = if let Ok((rest2, _)) = char::<_, nom::error::Error<&str>>('}')(rest) {
        // Empty hash
        (rest2, vec![])
    } else {
        let (rest, pairs) = separated_list1(
            delimited(multispace0, char(','), multispace0),
            |input| {
                let (rest, key) = parse_identifier(input)?;
                let (rest, _) = multispace0(rest)?;
                let (rest, _) = char(':')(rest)?;
                let (rest, _) = multispace0(rest)?;
                let (rest, expr) = parse_pipe(rest)?;
                Ok((rest, (key, expr)))
            }
        )(rest)?;
        let (rest, _) = multispace0(rest)?;
        let (rest, _) = char('}')(rest)?;
        (rest, pairs)
    };
    
    Ok((rest, JmesPathExpr::MultiSelectHash(pairs)))
}

/// Parse multi-select list: [expr1, expr2, expr3]
fn parse_multi_select_list(input: &str) -> IResult<&str, JmesPathExpr> {
    let (rest, _) = char('[')(input)?;
    let (rest, _) = multispace0(rest)?;
    
    // Check if it's empty
    if let Ok((rest2, _)) = char::<_, nom::error::Error<&str>>(']')(rest) {
        return Ok((rest2, JmesPathExpr::MultiSelectList(vec![])));
    }
    
    // Parse expressions
    let (rest, exprs) = separated_list1(
        delimited(multispace0, char(','), multispace0),
        parse_pipe
    )(rest)?;
    let (rest, _) = multispace0(rest)?;
    let (rest, _) = char(']')(rest)?;
    
    Ok((rest, JmesPathExpr::MultiSelectList(exprs)))
}

// ============================================================================
// COMPARISON OPERATORS
// ============================================================================

/// Parse comparison operator
fn parse_compare_op(input: &str) -> IResult<&str, CompareOp> {
    alt((
        map(tag("=="), |_| CompareOp::Eq),
        map(tag("!="), |_| CompareOp::Ne),
        map(tag(">="), |_| CompareOp::Gte),
        map(tag("<="), |_| CompareOp::Lte),
        map(tag(">"), |_| CompareOp::Gt),
        map(tag("<"), |_| CompareOp::Lt),
    ))(input)
}

// ============================================================================
// EXPRESSION PARSING WITH PRECEDENCE
// ============================================================================

/// Parse @ (current node)
fn parse_current_node(input: &str) -> IResult<&str, JmesPathExpr> {
    let (rest, _) = char('@')(input)?;
    Ok((rest, JmesPathExpr::CurrentNode))
}

/// Parse & (expression reference)
fn parse_expr_ref(input: &str) -> IResult<&str, JmesPathExpr> {
    let (rest, _) = char('&')(input)?;
    let (rest, _) = multispace0(rest)?;
    let (rest, expr) = parse_primary(rest)?;
    Ok((rest, JmesPathExpr::ExprRef(Box::new(expr))))
}

/// Parse primary expression (literals, paths, functions, or parenthesized expressions)
/// Also handles array operations: [0], [0:5], [*].field
fn parse_primary(input: &str) -> IResult<&str, JmesPathExpr> {
    let (rest, _) = multispace0(input)?;
    
    // First parse the base expression
    let (rest, mut expr) = alt((
        parse_current_node,  // @ - must come before other operators
        parse_expr_ref,      // & - must come before other operators
        parse_bool,          // Must come before path (true/false are identifiers)
        parse_float,         // Must come before int (123.45 starts like int)
        parse_int,
        parse_string,
        parse_string_single,
        parse_multi_select_hash,  // {key: expr}
        delimited(
            char('('),
            preceded(multispace0, parse_pipe),
            preceded(multispace0, char(')'))
        ),
        parse_function,      // Must come before path (function( looks like identifier)
        parse_path,
    ))(rest)?;
    
    // Now check for array operations or additional path segments
    let mut current_rest = rest;
    loop {
        let (new_rest, _) = multispace0(current_rest)?;
        
        // Check for flatten: []
        if let Ok((rest2, _)) = parse_flatten_bracket(new_rest) {
            expr = JmesPathExpr::Flatten(Box::new(expr));
            current_rest = rest2;
            continue;
        }
        
        // Check for projection: [*]
        if let Ok((rest2, _)) = parse_projection_wildcard(new_rest) {
            let (rest3, _) = multispace0(rest2)?;
            
            // Check if there's a path after the projection
            if let Ok((rest4, _)) = char::<_, nom::error::Error<&str>>('.')(rest3) {
                let (rest5, field_expr) = parse_primary(rest4)?;
                expr = JmesPathExpr::Projection(Box::new(expr), Box::new(field_expr));
                current_rest = rest5;
                continue;
            } else {
                // Just [*] without a following path
                expr = JmesPathExpr::Projection(Box::new(expr), Box::new(JmesPathExpr::Path(vec![])));
                current_rest = rest3;
                continue;
            }
        }
        
        // Check for slice: [0:5]
        if let Ok((rest2, (start, stop, step))) = parse_slice(new_rest) {
            expr = JmesPathExpr::Slice {
                base: Box::new(expr),
                start,
                stop,
                step,
            };
            current_rest = rest2;
            continue;
        }
        
        // Check for index: [0]
        if let Ok((rest2, idx)) = parse_index(new_rest) {
            expr = JmesPathExpr::Index(Box::new(expr), idx);
            current_rest = rest2;
            continue;
        }
        
        // Check for filter: [?condition]
        if let Ok((rest2, _)) = tag::<_, _, nom::error::Error<&str>>("[?")(new_rest) {
            let (rest3, _) = multispace0(rest2)?;
            let (rest4, condition) = parse_logical_or(rest3)?;
            let (rest5, _) = multispace0(rest4)?;
            let (rest6, _) = char(']')(rest5)?;
            expr = JmesPathExpr::Filter(Box::new(expr), Box::new(condition));
            current_rest = rest6;
            continue;
        }
        
        // No more array operations found
        break;
    }
    
    Ok((current_rest, expr))
}

/// Parse comparison expression: a > 5, b == "test"
fn parse_comparison(input: &str) -> IResult<&str, JmesPathExpr> {
    let (rest, _) = multispace0(input)?;
    let (rest, lhs) = parse_primary(rest)?;
    let (rest, _) = multispace0(rest)?;
    
    // Try to parse comparison operator
    if let Ok((rest2, op)) = parse_compare_op(rest) {
        let (rest3, _) = multispace0(rest2)?;
        let (rest4, rhs) = parse_primary(rest3)?;
        Ok((rest4, JmesPathExpr::Compare {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }))
    } else {
        // No comparison operator, return primary expression
        Ok((rest, lhs))
    }
}

/// Parse logical NOT: !expression
fn parse_logical_not(input: &str) -> IResult<&str, JmesPathExpr> {
    let (rest, _) = multispace0(input)?;
    if let Ok((rest2, _)) = char::<_, nom::error::Error<&str>>('!')(rest) {
        let (rest3, _) = multispace0(rest2)?;
        let (rest4, expr) = parse_logical_not(rest3)?;
        Ok((rest4, JmesPathExpr::Logic {
            op: LogicOp::Not,
            lhs: Box::new(expr),
            rhs: None,
        }))
    } else {
        parse_comparison(rest)
    }
}

/// Parse logical AND: expr && expr
fn parse_logical_and(input: &str) -> IResult<&str, JmesPathExpr> {
    let (rest, _) = multispace0(input)?;
    let (rest, first) = parse_logical_not(rest)?;
    let (rest, _) = multispace0(rest)?;
    
    if let Ok((rest2, _)) = tag::<_, _, nom::error::Error<&str>>("&&")(rest) {
        let (rest3, _) = multispace0(rest2)?;
        let (rest4, second) = parse_logical_and(rest3)?;
        Ok((rest4, JmesPathExpr::Logic {
            op: LogicOp::And,
            lhs: Box::new(first),
            rhs: Some(Box::new(second)),
        }))
    } else {
        Ok((rest, first))
    }
}

/// Parse logical OR: expr || expr
fn parse_logical_or(input: &str) -> IResult<&str, JmesPathExpr> {
    let (rest, _) = multispace0(input)?;
    let (rest, first) = parse_logical_and(rest)?;
    let (rest, _) = multispace0(rest)?;
    
    if let Ok((rest2, _)) = tag::<_, _, nom::error::Error<&str>>("||")(rest) {
        let (rest3, _) = multispace0(rest2)?;
        let (rest4, second) = parse_logical_or(rest3)?;
        Ok((rest4, JmesPathExpr::Logic {
            op: LogicOp::Or,
            lhs: Box::new(first),
            rhs: Some(Box::new(second)),
        }))
    } else {
        Ok((rest, first))
    }
}

/// Parse pipe operator: expr | expr
fn parse_pipe(input: &str) -> IResult<&str, JmesPathExpr> {
    let (rest, _) = multispace0(input)?;
    let (rest, first) = parse_logical_or(rest)?;
    let (rest, _) = multispace0(rest)?;
    
    if let Ok((rest2, _)) = char::<_, nom::error::Error<&str>>('|')(rest) {
        // Make sure it's not || (logical OR)
        if let Ok((_, _)) = char::<_, nom::error::Error<&str>>('|')(rest2) {
            // This is ||, not a pipe
            return Ok((rest, first));
        }
        
        let (rest3, _) = multispace0(rest2)?;
        let (rest4, second) = parse_pipe(rest3)?;
        Ok((rest4, JmesPathExpr::Pipe(Box::new(first), Box::new(second))))
    } else {
        Ok((rest, first))
    }
}

// ============================================================================
// MAIN PARSER
// ============================================================================

/// Main JMESPath parser entry point
pub fn parse_jmespath(input: &str) -> IResult<&str, JmesPathExpr> {
    let (rest, _) = multispace0(input)?;
    // Start with pipe (highest level) - this will handle all nested expressions including filters
    let result = parse_pipe(rest);
    let (_rest, _) = multispace0(result.as_ref().map(|(r, _)| *r).unwrap_or(rest))?;
    result
}

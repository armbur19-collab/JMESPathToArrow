use arrow_jmespath::*;

#[cfg(test)]
mod parser_tests {
    use super::*;

    // ============================================================================
    // LITERALS
    // ============================================================================

    #[test]
    fn test_parse_int() {
        let (_, ast) = parse_jmespath("42").unwrap();
        match ast {
            JmesPathExpr::ConstInt(42) => (),
            _ => panic!("Expected ConstInt(42), got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_negative_int() {
        let (_, ast) = parse_jmespath("-123").unwrap();
        match ast {
            JmesPathExpr::ConstInt(-123) => (),
            _ => panic!("Expected ConstInt(-123), got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_float() {
        let (_, ast) = parse_jmespath("3.14").unwrap();
        match ast {
            JmesPathExpr::ConstFloat(f) if (f - 3.14).abs() < 0.001 => (),
            _ => panic!("Expected ConstFloat(3.14), got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_bool_true() {
        let (_, ast) = parse_jmespath("true").unwrap();
        match ast {
            JmesPathExpr::ConstBool(true) => (),
            _ => panic!("Expected ConstBool(true), got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_bool_false() {
        let (_, ast) = parse_jmespath("false").unwrap();
        match ast {
            JmesPathExpr::ConstBool(false) => (),
            _ => panic!("Expected ConstBool(false), got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_string_double_quotes() {
        let (_, ast) = parse_jmespath(r#""hello world""#).unwrap();
        match ast {
            JmesPathExpr::ConstString(s) if s == "hello world" => (),
            _ => panic!("Expected ConstString, got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_string_single_quotes() {
        let (_, ast) = parse_jmespath("'hello world'").unwrap();
        match ast {
            JmesPathExpr::ConstString(s) if s == "hello world" => (),
            _ => panic!("Expected ConstString, got {:?}", ast),
        }
    }

    // ============================================================================
    // PATHS
    // ============================================================================

    #[test]
    fn test_parse_simple_path() {
        let (_, ast) = parse_jmespath("name").unwrap();
        match ast {
            JmesPathExpr::Path(parts) if parts == vec!["name"] => (),
            _ => panic!("Expected Path(['name']), got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_nested_path() {
        let (_, ast) = parse_jmespath("user.address.city").unwrap();
        match ast {
            JmesPathExpr::Path(parts) if parts == vec!["user", "address", "city"] => (),
            _ => panic!("Expected nested path, got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_path_with_underscore() {
        let (_, ast) = parse_jmespath("user_name").unwrap();
        match ast {
            JmesPathExpr::Path(parts) if parts == vec!["user_name"] => (),
            _ => panic!("Expected Path(['user_name']), got {:?}", ast),
        }
    }

    // ============================================================================
    // COMPARISONS
    // ============================================================================

    #[test]
    fn test_parse_gt_comparison() {
        let (_, ast) = parse_jmespath("age > 18").unwrap();
        match ast {
            JmesPathExpr::Compare { op: CompareOp::Gt, .. } => (),
            _ => panic!("Expected Compare with Gt, got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_eq_comparison() {
        let (_, ast) = parse_jmespath("status == 'active'").unwrap();
        match ast {
            JmesPathExpr::Compare { op: CompareOp::Eq, .. } => (),
            _ => panic!("Expected Compare with Eq, got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_neq_comparison() {
        let (_, ast) = parse_jmespath("name != 'test'").unwrap();
        match ast {
            JmesPathExpr::Compare { op: CompareOp::Ne, .. } => (),
            _ => panic!("Expected Compare with Ne, got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_gte_comparison() {
        let (_, ast) = parse_jmespath("score >= 100").unwrap();
        match ast {
            JmesPathExpr::Compare { op: CompareOp::Gte, .. } => (),
            _ => panic!("Expected Compare with Gte, got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_lte_comparison() {
        let (_, ast) = parse_jmespath("price <= 50.0").unwrap();
        match ast {
            JmesPathExpr::Compare { op: CompareOp::Lte, .. } => (),
            _ => panic!("Expected Compare with Lte, got {:?}", ast),
        }
    }

    // ============================================================================
    // LOGICAL OPERATORS
    // ============================================================================

    #[test]
    fn test_parse_and() {
        let (_, ast) = parse_jmespath("age > 18 && status == 'active'").unwrap();
        match ast {
            JmesPathExpr::Logic { op: LogicOp::And, .. } => (),
            _ => panic!("Expected Logic And, got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_or() {
        let (_, ast) = parse_jmespath("age < 18 || age > 65").unwrap();
        match ast {
            JmesPathExpr::Logic { op: LogicOp::Or, .. } => (),
            _ => panic!("Expected Logic Or, got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_not() {
        let (_, ast) = parse_jmespath("!active").unwrap();
        match ast {
            JmesPathExpr::Logic { op: LogicOp::Not, .. } => (),
            _ => panic!("Expected Logic Not, got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_complex_logic() {
        let (_, ast) = parse_jmespath("age > 18 && (status == 'active' || premium == true)").unwrap();
        match ast {
            JmesPathExpr::Logic { op: LogicOp::And, .. } => (),
            _ => panic!("Expected complex logic expression, got {:?}", ast),
        }
    }

    // ============================================================================
    // FILTERS
    // ============================================================================

    #[test]
    fn test_parse_simple_filter() {
        let (_, ast) = parse_jmespath("users[?age > 18]").unwrap();
        match ast {
            JmesPathExpr::Filter(array_expr, cond_expr) => {
                match *array_expr {
                    JmesPathExpr::Path(ref parts) if parts == &vec!["users"] => (),
                    _ => panic!("Expected users path in filter"),
                }
                match *cond_expr {
                    JmesPathExpr::Compare { .. } => (),
                    _ => panic!("Expected comparison in filter condition"),
                }
            },
            _ => panic!("Expected Filter, got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_filter_with_string() {
        let (_, ast) = parse_jmespath("items[?status == 'active']").unwrap();
        match ast {
            JmesPathExpr::Filter(..) => (),
            _ => panic!("Expected Filter, got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_filter_with_logic() {
        let (_, ast) = parse_jmespath("items[?age > 18 && status == 'active']").unwrap();
        match ast {
            JmesPathExpr::Filter(_, cond) => {
                match *cond {
                    JmesPathExpr::Logic { op: LogicOp::And, .. } => (),
                    _ => panic!("Expected And logic in filter"),
                }
            },
            _ => panic!("Expected Filter, got {:?}", ast),
        }
    }

    // ============================================================================
    // PARENTHESES
    // ============================================================================

    #[test]
    fn test_parse_parentheses() {
        let (_, ast) = parse_jmespath("(age > 18)").unwrap();
        match ast {
            JmesPathExpr::Compare { op: CompareOp::Gt, .. } => (),
            _ => panic!("Expected Compare, got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_nested_parentheses() {
        let (_, ast) = parse_jmespath("((age > 18) && (status == 'active'))").unwrap();
        match ast {
            JmesPathExpr::Logic { op: LogicOp::And, .. } => (),
            _ => panic!("Expected Logic And, got {:?}", ast),
        }
    }

    // ============================================================================
    // WHITESPACE HANDLING
    // ============================================================================

    #[test]
    fn test_parse_with_extra_spaces() {
        let (_, ast) = parse_jmespath("  age   >   18  ").unwrap();
        match ast {
            JmesPathExpr::Compare { op: CompareOp::Gt, .. } => (),
            _ => panic!("Expected Compare, got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_no_spaces() {
        let (_, ast) = parse_jmespath("age>18").unwrap();
        match ast {
            JmesPathExpr::Compare { op: CompareOp::Gt, .. } => (),
            _ => panic!("Expected Compare, got {:?}", ast),
        }
    }
}

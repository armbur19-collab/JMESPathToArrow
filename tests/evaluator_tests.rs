use arrow_jmespath::*;
use serde_json::json;
use arrow::array::Array;

#[cfg(test)]
mod evaluator_tests {
    use super::*;

    #[test]
    fn test_int_comparison_gt() {
        let data = json!([
            {"age": 25},
            {"age": 30},
            {"age": 20}
        ]);
        let rb = json_to_arrow(&data);
        let (_, ast) = parse_jmespath("age > 25").unwrap();
        let result = eval_jmespath(&ast, &rb);
        
        if let EvalResult::BoolArray(mask) = result {
            assert_eq!(mask.len(), 3);
            assert_eq!(mask.value(0), false); // 25 > 25 is false
            assert_eq!(mask.value(1), true);  // 30 > 25 is true
            assert_eq!(mask.value(2), false); // 20 > 25 is false
        } else {
            panic!("Expected BoolArray");
        }
    }

    #[test]
    fn test_string_comparison_eq() {
        let data = json!([
            {"name": "Alice"},
            {"name": "Bob"},
            {"name": "Charlie"}
        ]);
        let rb = json_to_arrow(&data);
        let (_, ast) = parse_jmespath("name == 'Bob'").unwrap();
        let result = eval_jmespath(&ast, &rb);
        
        if let EvalResult::BoolArray(mask) = result {
            assert_eq!(mask.len(), 3);
            assert_eq!(mask.value(0), false);
            assert_eq!(mask.value(1), true);
            assert_eq!(mask.value(2), false);
        } else {
            panic!("Expected BoolArray");
        }
    }

    #[test]
    fn test_float_comparison() {
        let data = json!([
            {"price": 19.99},
            {"price": 29.99},
            {"price": 15.50}
        ]);
        let rb = json_to_arrow(&data);
        let (_, ast) = parse_jmespath("price < 25.0").unwrap();
        let result = eval_jmespath(&ast, &rb);
        
        if let EvalResult::BoolArray(mask) = result {
            assert_eq!(mask.len(), 3);
            assert_eq!(mask.value(0), true);  // 19.99 < 25.0
            assert_eq!(mask.value(1), false); // 29.99 < 25.0
            assert_eq!(mask.value(2), true);  // 15.50 < 25.0
        } else {
            panic!("Expected BoolArray");
        }
    }

    #[test]
    fn test_boolean_comparison() {
        let data = json!([
            {"active": true},
            {"active": false},
            {"active": true}
        ]);
        let rb = json_to_arrow(&data);
        let (_, ast) = parse_jmespath("active == true").unwrap();
        let result = eval_jmespath(&ast, &rb);
        
        if let EvalResult::BoolArray(mask) = result {
            assert_eq!(mask.len(), 3);
            assert_eq!(mask.value(0), true);
            assert_eq!(mask.value(1), false);
            assert_eq!(mask.value(2), true);
        } else {
            panic!("Expected BoolArray");
        }
    }

    #[test]
    fn test_logical_and() {
        let data = json!([
            {"age": 25, "active": true},
            {"age": 30, "active": false},
            {"age": 20, "active": true}
        ]);
        let rb = json_to_arrow(&data);
        let (_, ast) = parse_jmespath("age > 22 && active == true").unwrap();
        let result = eval_jmespath(&ast, &rb);
        
        if let EvalResult::BoolArray(mask) = result {
            assert_eq!(mask.len(), 3);
            assert_eq!(mask.value(0), true);  // 25 > 22 && true
            assert_eq!(mask.value(1), false); // 30 > 22 && false
            assert_eq!(mask.value(2), false); // 20 > 22 && true
        } else {
            panic!("Expected BoolArray");
        }
    }

    #[test]
    fn test_logical_or() {
        let data = json!([
            {"age": 25},
            {"age": 30},
            {"age": 15}
        ]);
        let rb = json_to_arrow(&data);
        let (_, ast) = parse_jmespath("age < 20 || age > 28").unwrap();
        let result = eval_jmespath(&ast, &rb);
        
        if let EvalResult::BoolArray(mask) = result {
            assert_eq!(mask.len(), 3);
            assert_eq!(mask.value(0), false); // 25 < 20 || 25 > 28 = false
            assert_eq!(mask.value(1), true);  // 30 < 20 || 30 > 28 = true
            assert_eq!(mask.value(2), true);  // 15 < 20 || 15 > 28 = true
        } else {
            panic!("Expected BoolArray");
        }
    }

    #[test]
    fn test_logical_not() {
        let data = json!([
            {"active": true},
            {"active": false}
        ]);
        let rb = json_to_arrow(&data);
        let (_, ast) = parse_jmespath("!active").unwrap();
        let result = eval_jmespath(&ast, &rb);
        
        if let EvalResult::BoolArray(mask) = result {
            assert_eq!(mask.len(), 2);
            assert_eq!(mask.value(0), false);
            assert_eq!(mask.value(1), true);
        } else {
            panic!("Expected BoolArray");
        }
    }

    #[test]
    fn test_complex_expression() {
        let data = json!([
            {"age": 25, "salary": 50000, "name": "Alice"},
            {"age": 30, "salary": 60000, "name": "Bob"},
            {"age": 20, "salary": 45000, "name": "Charlie"}
        ]);
        let rb = json_to_arrow(&data);
        let (_, ast) = parse_jmespath("(age > 25 && salary < 65000) || name == 'Charlie'").unwrap();
        let result = eval_jmespath(&ast, &rb);
        
        if let EvalResult::BoolArray(mask) = result {
            assert_eq!(mask.len(), 3);
            assert_eq!(mask.value(0), false); // (25 > 25 && 50000 < 65000) || false = false
            assert_eq!(mask.value(1), true);  // (30 > 25 && 60000 < 65000) || false = true
            assert_eq!(mask.value(2), true);  // (20 > 25 && 45000 < 65000) || true = true
        } else {
            panic!("Expected BoolArray");
        }
    }

    #[test]
    fn test_all_comparison_operators() {
        let data = json!([
            {"value": 10},
            {"value": 20},
            {"value": 30}
        ]);
        let rb = json_to_arrow(&data);
        
        // Test all operators
        let tests = vec![
            ("value == 20", vec![false, true, false]),
            ("value != 20", vec![true, false, true]),
            ("value > 15", vec![false, true, true]),
            ("value < 25", vec![true, true, false]),
            ("value >= 20", vec![false, true, true]),
            ("value <= 20", vec![true, true, false]),
        ];
        
        for (expr, expected) in tests {
            let (_, ast) = parse_jmespath(expr).unwrap();
            let result = eval_jmespath(&ast, &rb);
            
            if let EvalResult::BoolArray(mask) = result {
                for (i, &exp) in expected.iter().enumerate() {
                    assert_eq!(mask.value(i), exp, "Failed for expression: {} at index {}", expr, i);
                }
            } else {
                panic!("Expected BoolArray for expression: {}", expr);
            }
        }
    }

    #[test]
    fn test_filter_record_batch() {
        let data = json!([
            {"name": "Alice", "age": 25},
            {"name": "Bob", "age": 30},
            {"name": "Charlie", "age": 20}
        ]);
        let rb = json_to_arrow(&data);
        
        let (_, ast) = parse_jmespath("age > 22").unwrap();
        let result = eval_jmespath(&ast, &rb);
        
        if let EvalResult::BoolArray(mask) = result {
            use arrow::compute::filter_record_batch;
            let filtered = filter_record_batch(&rb, &mask).unwrap();
            
            assert_eq!(filtered.num_rows(), 2); // Alice and Bob
            assert_eq!(rb.num_rows(), 3); // Original unchanged
        } else {
            panic!("Expected BoolArray");
        }
    }
}

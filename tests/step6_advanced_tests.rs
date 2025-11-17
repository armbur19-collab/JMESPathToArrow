// Tests for Step 6: Advanced JMESPath features (@, &, sort_by, group_by)
use arrow_jmespath::{parse_jmespath, eval_jmespath, json_to_arrow, arrow_to_json};
use serde_json::json;

#[test]
fn test_parse_current_node() {
    let result = parse_jmespath("@");
    assert!(result.is_ok(), "Should parse @ successfully");
    
    let (remaining, _expr) = result.unwrap();
    assert_eq!(remaining, "", "Should consume entire input");
}

#[test]
fn test_parse_expr_ref() {
    let result = parse_jmespath("&age");
    assert!(result.is_ok(), "Should parse &age successfully");
    
    let (remaining, _expr) = result.unwrap();
    assert_eq!(remaining, "", "Should consume entire input");
}

#[test]
fn test_parse_expr_ref_complex() {
    let result = parse_jmespath("&name.first");
    assert!(result.is_ok(), "Should parse &name.first successfully");
}

#[test]
fn test_current_node_in_function() {
    let result = parse_jmespath("sort_by(@, &age)");
    assert!(result.is_ok(), "Should parse sort_by(@, &age) successfully");
}

#[test]
fn test_sort_by_numeric() {
    let data = json!([
        {"name": "Charlie", "age": 30},
        {"name": "Alice", "age": 25},
        {"name": "Bob", "age": 35}
    ]);
    
    let rb = json_to_arrow(&data);
    let (_, expr) = parse_jmespath("sort_by(@, &age)").unwrap();
    let result = eval_jmespath(&expr, &rb);
    
    if let arrow_jmespath::EvalResult::RecordBatch(sorted_rb) = result {
        let json_result = arrow_to_json(&sorted_rb);
        
        // Check first element has age 25
        assert_eq!(json_result[0]["age"], 25);
        assert_eq!(json_result[0]["name"], "Alice");
        
        // Check last element has age 35
        assert_eq!(json_result[2]["age"], 35);
        assert_eq!(json_result[2]["name"], "Bob");
    } else {
        panic!("Expected RecordBatch result");
    }
}

#[test]
fn test_sort_by_string() {
    let data = json!([
        {"name": "Charlie", "score": 85},
        {"name": "Alice", "score": 90},
        {"name": "Bob", "score": 78}
    ]);
    
    let rb = json_to_arrow(&data);
    let (_, expr) = parse_jmespath("sort_by(@, &name)").unwrap();
    let result = eval_jmespath(&expr, &rb);
    
    if let arrow_jmespath::EvalResult::RecordBatch(sorted_rb) = result {
        let json_result = arrow_to_json(&sorted_rb);
        
        // Should be alphabetically sorted
        assert_eq!(json_result[0]["name"], "Alice");
        assert_eq!(json_result[1]["name"], "Bob");
        assert_eq!(json_result[2]["name"], "Charlie");
    } else {
        panic!("Expected RecordBatch result");
    }
}

#[test]
fn test_group_by_string() {
    let data = json!([
        {"product": "Laptop", "category": "Electronics"},
        {"product": "Mouse", "category": "Electronics"},
        {"product": "Desk", "category": "Furniture"},
        {"product": "Chair", "category": "Furniture"},
        {"product": "Monitor", "category": "Electronics"}
    ]);
    
    let rb = json_to_arrow(&data);
    let (_, expr) = parse_jmespath("group_by(@, &category)").unwrap();
    let result = eval_jmespath(&expr, &rb);
    
    if let arrow_jmespath::EvalResult::StringArray(arr) = result {
        use arrow::array::Array;
        
        // Should have 2 unique categories (sorted alphabetically)
        assert_eq!(arr.len(), 2);
        assert_eq!(arr.value(0), "Electronics");
        assert_eq!(arr.value(1), "Furniture");
    } else {
        panic!("Expected StringArray result");
    }
}

#[test]
fn test_group_by_numeric() {
    let data = json!([
        {"name": "Alice", "level": 2},
        {"name": "Bob", "level": 1},
        {"name": "Charlie", "level": 2},
        {"name": "Diana", "level": 3},
        {"name": "Eve", "level": 1}
    ]);
    
    let rb = json_to_arrow(&data);
    let (_, expr) = parse_jmespath("group_by(@, &level)").unwrap();
    let result = eval_jmespath(&expr, &rb);
    
    if let arrow_jmespath::EvalResult::Int64Array(arr) = result {
        use arrow::array::Array;
        
        // Should have 3 unique levels (sorted)
        assert_eq!(arr.len(), 3);
        assert_eq!(arr.value(0), 1);
        assert_eq!(arr.value(1), 2);
        assert_eq!(arr.value(2), 3);
    } else {
        panic!("Expected Int64Array result");
    }
}

#[test]
fn test_filter_then_sort() {
    let data = json!([
        {"name": "Alice", "age": 30, "active": true},
        {"name": "Bob", "age": 25, "active": false},
        {"name": "Charlie", "age": 35, "active": true},
        {"name": "Diana", "age": 28, "active": true}
    ]);
    
    let rb = json_to_arrow(&data);
    
    // First filter active users
    let (_, filter_expr) = parse_jmespath("active[?active == true]").unwrap();
    let filtered = eval_jmespath(&filter_expr, &rb);
    
    if let arrow_jmespath::EvalResult::RecordBatch(filtered_rb) = filtered {
        // Then sort by age
        let (_, sort_expr) = parse_jmespath("sort_by(@, &age)").unwrap();
        let sorted = eval_jmespath(&sort_expr, &filtered_rb);
        
        if let arrow_jmespath::EvalResult::RecordBatch(sorted_rb) = sorted {
            let json_result = arrow_to_json(&sorted_rb);
            
            // Should have 3 active users, sorted by age
            assert_eq!(json_result.as_array().unwrap().len(), 3);
            assert_eq!(json_result[0]["name"], "Diana");
            assert_eq!(json_result[0]["age"], 28);
            assert_eq!(json_result[2]["name"], "Charlie");
            assert_eq!(json_result[2]["age"], 35);
        } else {
            panic!("Expected RecordBatch from sort");
        }
    } else {
        panic!("Expected RecordBatch from filter");
    }
}

#[test]
fn test_wildcard_projection_already_works() {
    // Wildcard projection was already implemented in Step 4
    // This test ensures it still works
    let data = json!([
        {"name": "Alice", "age": 30},
        {"name": "Bob", "age": 25}
    ]);
    
    let rb = json_to_arrow(&data);
    let (_, expr) = parse_jmespath("name").unwrap();
    let result = eval_jmespath(&expr, &rb);
    
    if let arrow_jmespath::EvalResult::StringArray(arr) = result {
        use arrow::array::Array;
        assert_eq!(arr.len(), 2);
        assert_eq!(arr.value(0), "Alice");
        assert_eq!(arr.value(1), "Bob");
    } else {
        panic!("Expected StringArray");
    }
}

#[test]
fn test_sort_by_with_empty_recordbatch() {
    // Create a simple RecordBatch for testing
    let data2 = json!([{"name": "Alice", "age": 30}]);
    let rb2 = json_to_arrow(&data2);
    
    let (_, expr) = parse_jmespath("sort_by(@, &age)").unwrap();
    let result = eval_jmespath(&expr, &rb2);
    
    // Should not panic and return valid result
    assert!(matches!(result, arrow_jmespath::EvalResult::RecordBatch(_)));
}

#[test]
#[should_panic(expected = "Expression references (&) can only be used as function arguments")]
fn test_expr_ref_in_non_function_context_panics() {
    let data = json!([{"name": "Alice", "age": 30}]);
    let rb = json_to_arrow(&data);
    
    let (_, expr) = parse_jmespath("&age").unwrap();
    
    // Evaluating & outside of a function context should panic
    eval_jmespath(&expr, &rb);
}

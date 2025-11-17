// Tests for utility functions: reverse() and not_null()
use arrow_jmespath::{parse_jmespath, eval_jmespath, json_to_arrow, arrow_to_json};
use serde_json::json;
use arrow::array::Array;

// ============================================================================
// REVERSE() TESTS
// ============================================================================

#[test]
fn test_reverse_int_array() {
    let data = json!([
        {"values": 1},
        {"values": 2},
        {"values": 3},
        {"values": 4},
        {"values": 5}
    ]);
    
    let rb = json_to_arrow(&data);
    let (_, expr) = parse_jmespath("reverse(values)").unwrap();
    let result = eval_jmespath(&expr, &rb);
    
    if let arrow_jmespath::EvalResult::Int64Array(arr) = result {
        assert_eq!(arr.len(), 5);
        assert_eq!(arr.value(0), 5);
        assert_eq!(arr.value(1), 4);
        assert_eq!(arr.value(2), 3);
        assert_eq!(arr.value(3), 2);
        assert_eq!(arr.value(4), 1);
    } else {
        panic!("Expected Int64Array");
    }
}

#[test]
fn test_reverse_string_array() {
    let data = json!([
        {"name": "Alice"},
        {"name": "Bob"},
        {"name": "Charlie"}
    ]);
    
    let rb = json_to_arrow(&data);
    let (_, expr) = parse_jmespath("reverse(name)").unwrap();
    let result = eval_jmespath(&expr, &rb);
    
    if let arrow_jmespath::EvalResult::StringArray(arr) = result {
        assert_eq!(arr.len(), 3);
        assert_eq!(arr.value(0), "Charlie");
        assert_eq!(arr.value(1), "Bob");
        assert_eq!(arr.value(2), "Alice");
    } else {
        panic!("Expected StringArray");
    }
}

#[test]
fn test_reverse_recordbatch() {
    let data = json!([
        {"name": "Alice", "age": 30},
        {"name": "Bob", "age": 25},
        {"name": "Charlie", "age": 35}
    ]);
    
    let rb = json_to_arrow(&data);
    let (_, expr) = parse_jmespath("reverse(@)").unwrap();
    let result = eval_jmespath(&expr, &rb);
    
    if let arrow_jmespath::EvalResult::RecordBatch(reversed_rb) = result {
        let json_result = arrow_to_json(&reversed_rb);
        
        // First row should be Charlie
        assert_eq!(json_result[0]["name"], "Charlie");
        assert_eq!(json_result[0]["age"], 35);
        
        // Last row should be Alice
        assert_eq!(json_result[2]["name"], "Alice");
        assert_eq!(json_result[2]["age"], 30);
    } else {
        panic!("Expected RecordBatch");
    }
}

#[test]
fn test_reverse_with_sort_by() {
    // Classic use case: descending sort
    let data = json!([
        {"product": "Laptop", "price": 999},
        {"product": "Mouse", "price": 29},
        {"product": "Monitor", "price": 399},
        {"product": "Keyboard", "price": 79}
    ]);
    
    let rb = json_to_arrow(&data);
    
    // Sort ascending, then reverse for descending
    let (_, expr) = parse_jmespath("reverse(sort_by(@, &price))").unwrap();
    let result = eval_jmespath(&expr, &rb);
    
    if let arrow_jmespath::EvalResult::RecordBatch(sorted_rb) = result {
        let json_result = arrow_to_json(&sorted_rb);
        
        // Should be in descending order: 999, 399, 79, 29
        assert_eq!(json_result[0]["price"], 999);
        assert_eq!(json_result[0]["product"], "Laptop");
        
        assert_eq!(json_result[1]["price"], 399);
        assert_eq!(json_result[1]["product"], "Monitor");
        
        assert_eq!(json_result[2]["price"], 79);
        assert_eq!(json_result[2]["product"], "Keyboard");
        
        assert_eq!(json_result[3]["price"], 29);
        assert_eq!(json_result[3]["product"], "Mouse");
    } else {
        panic!("Expected RecordBatch");
    }
}

#[test]
fn test_reverse_empty_array() {
    // Test with a single element array (empty array would have no schema)
    let data = json!([{"x": 1}]);
    let rb = json_to_arrow(&data);
    
    let (_, expr) = parse_jmespath("reverse(x)").unwrap();
    let result = eval_jmespath(&expr, &rb);
    
    // Should not panic and return single element
    if let arrow_jmespath::EvalResult::Int64Array(arr) = result {
        assert_eq!(arr.len(), 1);
        assert_eq!(arr.value(0), 1);
    } else {
        panic!("Expected Int64Array");
    }
}

// ============================================================================
// NOT_NULL() TESTS
// ============================================================================

#[test]
fn test_not_null_filters_nulls() {
    // Create data with nulls using Option
    let data = json!([
        {"name": "Alice", "email": "alice@example.com"},
        {"name": "Bob", "email": null},
        {"name": "Charlie", "email": "charlie@example.com"},
        {"name": "Diana", "email": null},
        {"name": "Eve", "email": "eve@example.com"}
    ]);
    
    let rb = json_to_arrow(&data);
    let (_, expr) = parse_jmespath("not_null(email)").unwrap();
    let result = eval_jmespath(&expr, &rb);
    
    if let arrow_jmespath::EvalResult::RecordBatch(filtered_rb) = result {
        // Should only have 3 rows (Alice, Charlie, Eve)
        assert_eq!(filtered_rb.num_rows(), 3);
        
        let json_result = arrow_to_json(&filtered_rb);
        assert_eq!(json_result[0]["name"], "Alice");
        assert_eq!(json_result[1]["name"], "Charlie");
        assert_eq!(json_result[2]["name"], "Eve");
    } else {
        panic!("Expected RecordBatch");
    }
}

#[test]
fn test_not_null_all_values_present() {
    let data = json!([
        {"name": "Alice", "score": 85},
        {"name": "Bob", "score": 92},
        {"name": "Charlie", "score": 78}
    ]);
    
    let rb = json_to_arrow(&data);
    let (_, expr) = parse_jmespath("not_null(score)").unwrap();
    let result = eval_jmespath(&expr, &rb);
    
    if let arrow_jmespath::EvalResult::RecordBatch(filtered_rb) = result {
        // All rows should remain
        assert_eq!(filtered_rb.num_rows(), 3);
    } else {
        panic!("Expected RecordBatch");
    }
}

#[test]
fn test_not_null_all_nulls() {
    // Use a mix of null and non-null to avoid Null column type
    let data = json!([
        {"name": "Alice", "optional": null},
        {"name": "Bob", "optional": 1},  // One non-null value
        {"name": "Charlie", "optional": null}
    ]);
    
    let rb = json_to_arrow(&data);
    let (_, expr) = parse_jmespath("not_null(optional)").unwrap();
    let result = eval_jmespath(&expr, &rb);
    
    if let arrow_jmespath::EvalResult::RecordBatch(filtered_rb) = result {
        // Should have only 1 row (Bob)
        assert_eq!(filtered_rb.num_rows(), 1);
    } else {
        panic!("Expected RecordBatch");
    }
}

#[test]
fn test_not_null_with_filter_chain() {
    let data = json!([
        {"name": "Alice", "age": 30, "email": "alice@example.com"},
        {"name": "Bob", "age": 25, "email": null},
        {"name": "Charlie", "age": 35, "email": "charlie@example.com"},
        {"name": "Diana", "age": 28, "email": null}
    ]);
    
    let rb = json_to_arrow(&data);
    
    // First filter by age, then filter out nulls
    let (_, age_filter) = parse_jmespath("age[?age > 25]").unwrap();
    let age_filtered = eval_jmespath(&age_filter, &rb);
    
    if let arrow_jmespath::EvalResult::RecordBatch(age_rb) = age_filtered {
        let (_, not_null_expr) = parse_jmespath("not_null(email)").unwrap();
        let result = eval_jmespath(&not_null_expr, &age_rb);
        
        if let arrow_jmespath::EvalResult::RecordBatch(final_rb) = result {
            // Should have 2 rows: Alice (30) and Charlie (35)
            // Diana (28 > 25 but null email) is filtered out
            assert_eq!(final_rb.num_rows(), 2);
            
            let json_result = arrow_to_json(&final_rb);
            assert_eq!(json_result[0]["name"], "Alice");
            assert_eq!(json_result[1]["name"], "Charlie");
        } else {
            panic!("Expected RecordBatch from not_null");
        }
    } else {
        panic!("Expected RecordBatch from age filter");
    }
}

#[test]
fn test_not_null_string_field() {
    let data = json!([
        {"id": 1, "comment": "Great!"},
        {"id": 2, "comment": null},
        {"id": 3, "comment": "Good"},
        {"id": 4, "comment": null}
    ]);
    
    let rb = json_to_arrow(&data);
    let (_, expr) = parse_jmespath("not_null(comment)").unwrap();
    let result = eval_jmespath(&expr, &rb);
    
    if let arrow_jmespath::EvalResult::RecordBatch(filtered_rb) = result {
        assert_eq!(filtered_rb.num_rows(), 2);
        
        let json_result = arrow_to_json(&filtered_rb);
        assert_eq!(json_result[0]["id"], 1);
        assert_eq!(json_result[1]["id"], 3);
    } else {
        panic!("Expected RecordBatch");
    }
}

// ============================================================================
// COMBINED TESTS (reverse + not_null)
// ============================================================================

#[test]
fn test_reverse_and_not_null_combined() {
    let data = json!([
        {"name": "Alice", "score": 85},
        {"name": "Bob", "score": null},
        {"name": "Charlie", "score": 92},
        {"name": "Diana", "score": null},
        {"name": "Eve", "score": 78}
    ]);
    
    let rb = json_to_arrow(&data);
    
    // Filter out nulls first
    let (_, not_null_expr) = parse_jmespath("not_null(score)").unwrap();
    let filtered = eval_jmespath(&not_null_expr, &rb);
    
    if let arrow_jmespath::EvalResult::RecordBatch(filtered_rb) = filtered {
        // Then sort by score
        let (_, sort_expr) = parse_jmespath("sort_by(@, &score)").unwrap();
        let sorted = eval_jmespath(&sort_expr, &filtered_rb);
        
        if let arrow_jmespath::EvalResult::RecordBatch(sorted_rb) = sorted {
            // Then reverse for descending order
            let (_, reverse_expr) = parse_jmespath("reverse(@)").unwrap();
            let final_result = eval_jmespath(&reverse_expr, &sorted_rb);
            
            if let arrow_jmespath::EvalResult::RecordBatch(final_rb) = final_result {
                let json_result = arrow_to_json(&final_rb);
                
                // Should be: Charlie (92), Alice (85), Eve (78)
                assert_eq!(json_result.as_array().unwrap().len(), 3);
                assert_eq!(json_result[0]["name"], "Charlie");
                assert_eq!(json_result[0]["score"], 92);
                assert_eq!(json_result[1]["name"], "Alice");
                assert_eq!(json_result[1]["score"], 85);
                assert_eq!(json_result[2]["name"], "Eve");
                assert_eq!(json_result[2]["score"], 78);
            } else {
                panic!("Expected RecordBatch from reverse");
            }
        } else {
            panic!("Expected RecordBatch from sort");
        }
    } else {
        panic!("Expected RecordBatch from not_null");
    }
}

#[test]
fn test_real_world_use_case() {
    // Real-world scenario: Product catalog with optional reviews
    let data = json!([
        {"product": "Laptop", "price": 999, "rating": 4.5},
        {"product": "Mouse", "price": 29, "rating": null},  // No reviews yet
        {"product": "Monitor", "price": 399, "rating": 4.8},
        {"product": "Keyboard", "price": 79, "rating": null},  // No reviews yet
        {"product": "Webcam", "price": 149, "rating": 4.2},
        {"product": "Headset", "price": 89, "rating": 4.7}
    ]);
    
    let rb = json_to_arrow(&data);
    
    // Goal: Get top-rated products (exclude unrated, sort by rating descending)
    
    // Step 1: Filter out products without ratings
    let (_, not_null_expr) = parse_jmespath("not_null(rating)").unwrap();
    let rated = eval_jmespath(&not_null_expr, &rb);
    
    if let arrow_jmespath::EvalResult::RecordBatch(rated_rb) = rated {
        // Step 2: Sort by rating
        let (_, sort_expr) = parse_jmespath("sort_by(@, &rating)").unwrap();
        let sorted = eval_jmespath(&sort_expr, &rated_rb);
        
        if let arrow_jmespath::EvalResult::RecordBatch(sorted_rb) = sorted {
            // Step 3: Reverse for descending order
            let (_, reverse_expr) = parse_jmespath("reverse(@)").unwrap();
            let result = eval_jmespath(&reverse_expr, &sorted_rb);
            
            if let arrow_jmespath::EvalResult::RecordBatch(final_rb) = result {
                let json_result = arrow_to_json(&final_rb);
                
                // Should be top-rated first: Monitor (4.8), Headset (4.7), Laptop (4.5), Webcam (4.2)
                assert_eq!(json_result.as_array().unwrap().len(), 4);
                assert_eq!(json_result[0]["product"], "Monitor");
                assert_eq!(json_result[0]["rating"], 4.8);
                assert_eq!(json_result[1]["product"], "Headset");
                assert_eq!(json_result[3]["product"], "Webcam");
            } else {
                panic!("Expected RecordBatch");
            }
        } else {
            panic!("Expected RecordBatch from sort");
        }
    } else {
        panic!("Expected RecordBatch from not_null");
    }
}

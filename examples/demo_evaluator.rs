use arrow_jmespath::*;
use serde_json::json;
use arrow::array::Array;

fn main() {
    println!("=== JMESPath Evaluator Demo ===\n");

    // Test 1: Integer comparisons
    println!("--- Test 1: Integer Comparisons ---");
    let data = json!([
        {"name": "Alice", "age": 25, "salary": 50000},
        {"name": "Bob", "age": 30, "salary": 60000},
        {"name": "Charlie", "age": 20, "salary": 45000},
        {"name": "Diana", "age": 35, "salary": 70000}
    ]);
    
    let rb = json_to_arrow(&data);
    println!("Original data: {} rows", rb.num_rows());
    
    // Filter: age > 25
    let (_, ast) = parse_jmespath("age > 25").unwrap();
    let result = eval_jmespath(&ast, &rb);
    println!("Expression: age > 25");
    println!("Result: {:?}\n", result);
    
    // Test 2: String comparisons
    println!("--- Test 2: String Comparisons ---");
    let (_, ast) = parse_jmespath("name == 'Bob'").unwrap();
    let result = eval_jmespath(&ast, &rb);
    println!("Expression: name == 'Bob'");
    println!("Result: {:?}\n", result);
    
    // Test 3: Logical AND
    println!("--- Test 3: Logical AND ---");
    let (_, ast) = parse_jmespath("age > 25 && salary < 65000").unwrap();
    let result = eval_jmespath(&ast, &rb);
    println!("Expression: age > 25 && salary < 65000");
    println!("Result: {:?}\n", result);
    
    // Test 4: Logical OR
    println!("--- Test 4: Logical OR ---");
    let (_, ast) = parse_jmespath("age < 22 || age > 32").unwrap();
    let result = eval_jmespath(&ast, &rb);
    println!("Expression: age < 22 || age > 32");
    println!("Result: {:?}\n", result);
    
    // Test 5: NOT
    println!("--- Test 5: NOT ---");
    let data_bool = json!([
        {"name": "Alice", "active": true},
        {"name": "Bob", "active": false},
        {"name": "Charlie", "active": true}
    ]);
    let rb_bool = json_to_arrow(&data_bool);
    let (_, ast) = parse_jmespath("!active").unwrap();
    let result = eval_jmespath(&ast, &rb_bool);
    println!("Expression: !active");
    println!("Result: {:?}\n", result);
    
    // Test 6: Complex expression with parentheses
    println!("--- Test 6: Complex Expression ---");
    let (_, ast) = parse_jmespath("(age > 25 && salary < 65000) || name == 'Charlie'").unwrap();
    let result = eval_jmespath(&ast, &rb);
    println!("Expression: (age > 25 && salary < 65000) || name == 'Charlie'");
    println!("Result: {:?}\n", result);
    
    // Test 7: Filter (complete filtering)
    println!("--- Test 7: Filter ---");
    let (_, ast) = parse_jmespath("age > 25").unwrap();
    let result = eval_jmespath(&ast, &rb);
    
    if let EvalResult::BoolArray(mask) = result {
        use arrow::compute::filter_record_batch;
        let filtered = filter_record_batch(&rb, &mask).unwrap();
        println!("Expression: age > 25");
        println!("Filtered rows: {} (from {})", filtered.num_rows(), rb.num_rows());
        
        // Print filtered names
        if let Some(name_col) = filtered.column_by_name("name") {
            use arrow::array::StringArray;
            let names = name_col.as_any().downcast_ref::<StringArray>().unwrap();
            print!("Names: ");
            for i in 0..names.len() {
                if !names.is_null(i) {
                    print!("{} ", names.value(i));
                }
            }
            println!("\n");
        }
    }
    
    // Test 8: Multiple data types
    println!("--- Test 8: Multiple Data Types ---");
    let data_mixed = json!([
        {"name": "Product A", "price": 19.99, "in_stock": true, "quantity": 100},
        {"name": "Product B", "price": 29.99, "in_stock": false, "quantity": 0},
        {"name": "Product C", "price": 15.50, "in_stock": true, "quantity": 50}
    ]);
    let rb_mixed = json_to_arrow(&data_mixed);
    
    let (_, ast) = parse_jmespath("price < 25.0 && in_stock == true").unwrap();
    let result = eval_jmespath(&ast, &rb_mixed);
    println!("Expression: price < 25.0 && in_stock == true");
    println!("Result: {:?}\n", result);
    
    // Test 9: String inequality
    println!("--- Test 9: String Inequality ---");
    let (_, ast) = parse_jmespath("name != 'Bob'").unwrap();
    let result = eval_jmespath(&ast, &rb);
    println!("Expression: name != 'Bob'");
    if let EvalResult::BoolArray(mask) = result {
        println!("Result: {:?}", mask);
        println!("True count: {}\n", mask.true_count());
    }
    
    println!("=== All tests completed successfully! ===");
}

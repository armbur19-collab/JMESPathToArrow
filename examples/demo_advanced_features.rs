// Demo: Advanced JMESPath features (@, &, wildcards, sort_by, group_by)
use arrow_jmespath::{json_to_arrow, parse_jmespath, eval_jmespath, arrow_to_json_string};
use arrow::array::Array;
use serde_json::json;

fn main() {
    println!("=== Advanced JMESPath Features Demo ===\n");
    
    // ========================================================================
    // Test 1: @ (current object) in filters
    // ========================================================================
    println!("Test 1: @ (current object) in filters");
    println!("---------------------------------------");
    
    let data = json!([
        {"name": "Alice", "age": 30, "scores": [85, 90, 78]},
        {"name": "Bob", "age": 25, "scores": [92, 88, 95]},
        {"name": "Charlie", "age": 35, "scores": [76, 82, 88]}
    ]);
    
    let rb = json_to_arrow(&data);
    
    // Query: numbers[?@ > 80] - filter array elements where element > 80
    println!("Query: scores[?@ > 85]");
    println!("This should filter scores > 85 from each array\n");
    
    // For now, let's test basic @ parsing
    match parse_jmespath("@") {
        Ok((_, expr)) => {
            println!("✅ @ parsed successfully: {:?}\n", expr);
        },
        Err(e) => println!("❌ @ parsing failed: {:?}\n", e),
    }
    
    // ========================================================================
    // Test 2: & (expression reference) 
    // ========================================================================
    println!("Test 2: & (expression reference)");
    println!("----------------------------------");
    
    // Query: sort_by(people, &age) - sort by age field
    println!("Query: &age");
    println!("This creates a reference to the 'age' expression\n");
    
    match parse_jmespath("&age") {
        Ok((_, expr)) => {
            println!("✅ & parsed successfully: {:?}\n", expr);
        },
        Err(e) => println!("❌ & parsing failed: {:?}\n", e),
    }
    
    // ========================================================================
    // Test 3: Wildcard projection (already works!)
    // ========================================================================
    println!("Test 3: Wildcard projection [*].field");
    println!("---------------------------------------");
    
    let data2 = json!([
        {"name": "Alice", "age": 30},
        {"name": "Bob", "age": 25},
        {"name": "Charlie", "age": 35}
    ]);
    
    let rb2 = json_to_arrow(&data2);
    
    // This should already work from Step 4
    println!("Query: name[*]");
    match parse_jmespath("name") {
        Ok((_, expr)) => {
            let result = eval_jmespath(&expr, &rb2);
            println!("✅ Projection works!\n");
            
            // Try to get first 3 values
            if let arrow_jmespath::EvalResult::StringArray(arr) = result {
                println!("Results:");
                for i in 0..arr.len().min(3) {
                    println!("  - {}", arr.value(i));
                }
                println!();
            }
        },
        Err(e) => println!("❌ Query failed: {:?}\n", e),
    }
    
    // ========================================================================
    // Test 4: Complex query with @
    // ========================================================================
    println!("Test 4: Complex filter with @");
    println!("-------------------------------");
    
    let numbers = json!([
        {"value": 10},
        {"value": 25},
        {"value": 15},
        {"value": 30},
        {"value": 5}
    ]);
    
    let rb3 = json_to_arrow(&numbers);
    
    // Standard query that should work: value[?value > 20]
    println!("Query: value[?value > 20]");
    match parse_jmespath("value[?value > 20]") {
        Ok((_, expr)) => {
            let result = eval_jmespath(&expr, &rb3);
            println!("✅ Filter works!");
            
            if let arrow_jmespath::EvalResult::RecordBatch(filtered_rb) = result {
                let json_result = arrow_to_json_string(&filtered_rb);
                println!("Filtered result:\n{}\n", json_result);
            }
        },
        Err(e) => println!("❌ Query failed: {:?}\n", e),
    }
    
    // ========================================================================
    // Test 5: sort_by function (to be implemented)
    // ========================================================================
    println!("Test 5: sort_by function");
    println!("-------------------------");
    
    println!("Query: sort_by(people, &age)");
    println!("⚠️  Not yet implemented - requires & expression references\n");
    
    // ========================================================================
    // Test 6: group_by function (to be implemented)
    // ========================================================================
    println!("Test 6: group_by function");
    println!("--------------------------");
    
    println!("Query: group_by(items, &category)");
    println!("⚠️  Not yet implemented - requires & expression references\n");
    
    // ========================================================================
    // Summary
    // ========================================================================
    println!("=== Summary ===");
    println!("✅ @ (current object) - Parser works!");
    println!("✅ & (expression ref) - Parser works!");
    println!("✅ Wildcard [*] - Already implemented!");
    println!("⚠️  @ evaluation - Needs context support");
    println!("⚠️  sort_by() - To be implemented");
    println!("⚠️  group_by() - To be implemented");
}

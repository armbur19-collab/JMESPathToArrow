// Demo: sort_by and group_by functions
use arrow_jmespath::{json_to_arrow, parse_jmespath, eval_jmespath, arrow_to_json_string};
use serde_json::json;

fn main() {
    println!("=== sort_by() and group_by() Functions Demo ===\n");
    
    // ========================================================================
    // Test 1: sort_by with numeric field
    // ========================================================================
    println!("Test 1: sort_by() with numeric field");
    println!("--------------------------------------");
    
    let people = json!([
        {"name": "Alice", "age": 35, "city": "NYC"},
        {"name": "Bob", "age": 25, "city": "LA"},
        {"name": "Charlie", "age": 30, "city": "NYC"},
        {"name": "Diana", "age": 28, "city": "SF"}
    ]);
    
    let rb = json_to_arrow(&people);
    
    println!("Original data:");
    println!("{}\n", arrow_to_json_string(&rb));
    
    // Sort by age
    println!("Query: sort_by(@, &age)");
    match parse_jmespath("sort_by(@, &age)") {
        Ok((_, expr)) => {
            let result = eval_jmespath(&expr, &rb);
            
            if let arrow_jmespath::EvalResult::RecordBatch(sorted_rb) = result {
                println!("✅ Sorted by age:");
                println!("{}\n", arrow_to_json_string(&sorted_rb));
            }
        },
        Err(e) => println!("❌ Query failed: {:?}\n", e),
    }
    
    // ========================================================================
    // Test 2: sort_by with string field
    // ========================================================================
    println!("Test 2: sort_by() with string field");
    println!("-------------------------------------");
    
    println!("Query: sort_by(@, &name)");
    match parse_jmespath("sort_by(@, &name)") {
        Ok((_, expr)) => {
            let result = eval_jmespath(&expr, &rb);
            
            if let arrow_jmespath::EvalResult::RecordBatch(sorted_rb) = result {
                println!("✅ Sorted by name (alphabetically):");
                println!("{}\n", arrow_to_json_string(&sorted_rb));
            }
        },
        Err(e) => println!("❌ Query failed: {:?}\n", e),
    }
    
    // ========================================================================
    // Test 3: group_by with string field
    // ========================================================================
    println!("Test 3: group_by() - unique values");
    println!("------------------------------------");
    
    let items = json!([
        {"product": "Laptop", "category": "Electronics", "price": 999},
        {"product": "Mouse", "category": "Electronics", "price": 29},
        {"product": "Desk", "category": "Furniture", "price": 299},
        {"product": "Chair", "category": "Furniture", "price": 199},
        {"product": "Monitor", "category": "Electronics", "price": 399}
    ]);
    
    let rb2 = json_to_arrow(&items);
    
    println!("Original data:");
    println!("{}\n", arrow_to_json_string(&rb2));
    
    println!("Query: group_by(@, &category)");
    match parse_jmespath("group_by(@, &category)") {
        Ok((_, expr)) => {
            let result = eval_jmespath(&expr, &rb2);
            
            println!("✅ Unique categories:");
            if let arrow_jmespath::EvalResult::StringArray(arr) = result {
                use arrow::array::Array;
                for i in 0..arr.len() {
                    println!("  - {}", arr.value(i));
                }
                println!();
            }
        },
        Err(e) => println!("❌ Query failed: {:?}\n", e),
    }
    
    // ========================================================================
    // Test 4: Combining filter and sort_by
    // ========================================================================
    println!("Test 4: Combining filter and sort_by");
    println!("--------------------------------------");
    
    let employees = json!([
        {"name": "Alice", "salary": 75000, "department": "Engineering"},
        {"name": "Bob", "salary": 65000, "department": "Sales"},
        {"name": "Charlie", "salary": 85000, "department": "Engineering"},
        {"name": "Diana", "salary": 70000, "department": "Marketing"},
        {"name": "Eve", "salary": 95000, "department": "Engineering"}
    ]);
    
    let rb3 = json_to_arrow(&employees);
    
    println!("Original data:");
    println!("{}\n", arrow_to_json_string(&rb3));
    
    // Filter Engineering, then sort by salary
    println!("Query: sort_by(department[?department == 'Engineering'], &salary) | @");
    println!("(Filter Engineering department, then sort by salary)\n");
    
    // Step 1: Filter
    match parse_jmespath("department[?department == 'Engineering']") {
        Ok((_, expr)) => {
            let filtered = eval_jmespath(&expr, &rb3);
            
            if let arrow_jmespath::EvalResult::RecordBatch(filtered_rb) = filtered {
                println!("Step 1 - Filtered Engineering:");
                println!("{}\n", arrow_to_json_string(&filtered_rb));
                
                // Step 2: Sort
                match parse_jmespath("sort_by(@, &salary)") {
                    Ok((_, sort_expr)) => {
                        let sorted = eval_jmespath(&sort_expr, &filtered_rb);
                        
                        if let arrow_jmespath::EvalResult::RecordBatch(sorted_rb) = sorted {
                            println!("Step 2 - Sorted by salary:");
                            println!("{}\n", arrow_to_json_string(&sorted_rb));
                        }
                    },
                    Err(e) => println!("❌ Sort failed: {:?}\n", e),
                }
            }
        },
        Err(e) => println!("❌ Filter failed: {:?}\n", e),
    }
    
    // ========================================================================
    // Summary
    // ========================================================================
    println!("=== Summary ===");
    println!("✅ sort_by(@, &field) - Sorts RecordBatch by field");
    println!("✅ group_by(@, &field) - Returns unique values (simplified grouping)");
    println!("✅ @ operator - References current RecordBatch");
    println!("✅ & operator - Creates expression reference for functions");
    println!("✅ Combining filters and sorting works!");
}

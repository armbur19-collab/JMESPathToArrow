// Demo: Utility Functions - reverse() and not_null()
use arrow_jmespath::{json_to_arrow, parse_jmespath, eval_jmespath, arrow_to_json_string};
use serde_json::json;

fn main() {
    println!("=== Utility Functions Demo: reverse() and not_null() ===\n");
    
    // ========================================================================
    // Demo 1: reverse() - Simple Array Reversal
    // ========================================================================
    println!("Demo 1: reverse() - Simple Array Reversal");
    println!("==========================================");
    
    let numbers = json!([
        {"value": 1},
        {"value": 2},
        {"value": 3},
        {"value": 4},
        {"value": 5}
    ]);
    
    let rb = json_to_arrow(&numbers);
    
    println!("Original: [1, 2, 3, 4, 5]");
    let (_, expr) = parse_jmespath("reverse(value)").unwrap();
    let result = eval_jmespath(&expr, &rb);
    
    if let arrow_jmespath::EvalResult::Int64Array(arr) = result {
        use arrow::array::Array;
        let reversed: Vec<i64> = (0..arr.len()).map(|i| arr.value(i)).collect();
        println!("Reversed: {:?}\n", reversed);
    }
    
    // ========================================================================
    // Demo 2: reverse() for Descending Sort
    // ========================================================================
    println!("Demo 2: reverse() for Descending Sort");
    println!("======================================");
    
    let products = json!([
        {"product": "Laptop", "price": 999},
        {"product": "Mouse", "price": 29},
        {"product": "Monitor", "price": 399},
        {"product": "Keyboard", "price": 79}
    ]);
    
    let rb2 = json_to_arrow(&products);
    
    println!("Products sorted by price (DESCENDING):\n");
    
    // Sort ascending, then reverse for descending
    let (_, expr) = parse_jmespath("reverse(sort_by(@, &price))").unwrap();
    let result = eval_jmespath(&expr, &rb2);
    
    if let arrow_jmespath::EvalResult::RecordBatch(sorted_rb) = result {
        let json_result = arrow_to_json_string(&sorted_rb);
        println!("{}\n", json_result);
    }
    
    // ========================================================================
    // Demo 3: not_null() - Data Cleaning
    // ========================================================================
    println!("Demo 3: not_null() - Filter Null Values");
    println!("========================================");
    
    let users = json!([
        {"name": "Alice", "email": "alice@example.com"},
        {"name": "Bob", "email": null},
        {"name": "Charlie", "email": "charlie@example.com"},
        {"name": "Diana", "email": null},
        {"name": "Eve", "email": "eve@example.com"}
    ]);
    
    let rb3 = json_to_arrow(&users);
    
    println!("Original: 5 users (2 with null email)");
    println!("\nFiltered (only users with email):\n");
    
    let (_, expr) = parse_jmespath("not_null(email)").unwrap();
    let result = eval_jmespath(&expr, &rb3);
    
    if let arrow_jmespath::EvalResult::RecordBatch(filtered_rb) = result {
        println!("Rows after filtering: {}", filtered_rb.num_rows());
        let json_result = arrow_to_json_string(&filtered_rb);
        println!("{}\n", json_result);
    }
    
    // ========================================================================
    // Demo 4: Combined - Real-World Use Case
    // ========================================================================
    println!("Demo 4: Real-World Use Case - Top Rated Products");
    println!("=================================================");
    
    let catalog = json!([
        {"product": "Laptop", "price": 999, "rating": 4.5},
        {"product": "Mouse", "price": 29, "rating": null},  // No reviews yet
        {"product": "Monitor", "price": 399, "rating": 4.8},
        {"product": "Keyboard", "price": 79, "rating": null},
        {"product": "Webcam", "price": 149, "rating": 4.2},
        {"product": "Headset", "price": 89, "rating": 4.7}
    ]);
    
    let rb4 = json_to_arrow(&catalog);
    
    println!("Goal: Show top-rated products (exclude unrated, sort by rating DESC)\n");
    
    // Step 1: Filter out products without ratings
    println!("Step 1: Filter out null ratings");
    let (_, not_null_expr) = parse_jmespath("not_null(rating)").unwrap();
    let rated = eval_jmespath(&not_null_expr, &rb4);
    
    if let arrow_jmespath::EvalResult::RecordBatch(rated_rb) = rated {
        println!("  → {} products with ratings\n", rated_rb.num_rows());
        
        // Step 2: Sort by rating
        println!("Step 2: Sort by rating (ascending)");
        let (_, sort_expr) = parse_jmespath("sort_by(@, &rating)").unwrap();
        let sorted = eval_jmespath(&sort_expr, &rated_rb);
        
        if let arrow_jmespath::EvalResult::RecordBatch(sorted_rb) = sorted {
            // Step 3: Reverse for descending order
            println!("Step 3: Reverse for descending order\n");
            let (_, reverse_expr) = parse_jmespath("reverse(@)").unwrap();
            let result = eval_jmespath(&reverse_expr, &sorted_rb);
            
            if let arrow_jmespath::EvalResult::RecordBatch(final_rb) = result {
                println!("Top-Rated Products:\n");
                let json_result = arrow_to_json_string(&final_rb);
                println!("{}\n", json_result);
            }
        }
    }
    
    // ========================================================================
    // Demo 5: Multi-Stage Pipeline
    // ========================================================================
    println!("Demo 5: Multi-Stage Data Pipeline");
    println!("==================================");
    
    let sales = json!([
        {"customer": "Alice", "amount": 150, "status": "completed"},
        {"customer": "Bob", "amount": null, "status": "pending"},
        {"customer": "Charlie", "amount": 200, "status": "completed"},
        {"customer": "Diana", "amount": 75, "status": "completed"},
        {"customer": "Eve", "amount": null, "status": "cancelled"}
    ]);
    
    let rb5 = json_to_arrow(&sales);
    
    println!("Goal: Top completed sales (descending)\n");
    
    // Pipeline: Filter completed → Remove nulls → Sort → Reverse
    
    println!("Step 1: Filter completed status");
    let (_, status_filter) = parse_jmespath("status[?status == 'completed']").unwrap();
    let completed = eval_jmespath(&status_filter, &rb5);
    
    if let arrow_jmespath::EvalResult::RecordBatch(completed_rb) = completed {
        println!("  → {} completed sales\n", completed_rb.num_rows());
        
        println!("Step 2: Remove null amounts");
        let (_, not_null_expr) = parse_jmespath("not_null(amount)").unwrap();
        let valid = eval_jmespath(&not_null_expr, &completed_rb);
        
        if let arrow_jmespath::EvalResult::RecordBatch(valid_rb) = valid {
            println!("  → {} sales with valid amounts\n", valid_rb.num_rows());
            
            println!("Step 3: Sort by amount and reverse");
            let (_, sort_expr) = parse_jmespath("sort_by(@, &amount)").unwrap();
            let sorted = eval_jmespath(&sort_expr, &valid_rb);
            
            if let arrow_jmespath::EvalResult::RecordBatch(sorted_rb) = sorted {
                let (_, reverse_expr) = parse_jmespath("reverse(@)").unwrap();
                let result = eval_jmespath(&reverse_expr, &sorted_rb);
                
                if let arrow_jmespath::EvalResult::RecordBatch(final_rb) = result {
                    println!("\nFinal Result (Top Sales):\n");
                    let json_result = arrow_to_json_string(&final_rb);
                    println!("{}\n", json_result);
                }
            }
        }
    }
    
    // ========================================================================
    // Summary
    // ========================================================================
    println!("=== Summary ===");
    println!("✅ reverse(array) - Reverses array order");
    println!("   Use case: Descending sorts");
    println!("   Example: reverse(sort_by(@, &price))");
    println!();
    println!("✅ not_null(field) - Filters out null values");
    println!("   Use case: Data cleaning, quality checks");
    println!("   Example: not_null(email)");
    println!();
    println!("✅ Combined: Build powerful data pipelines!");
    println!("   Filter → Clean → Sort → Reverse");
}

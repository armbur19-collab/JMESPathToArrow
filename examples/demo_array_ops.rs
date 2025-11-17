use arrow_jmespath::{parse_jmespath, eval_jmespath, json_to_arrow};
use serde_json::json;

fn main() {
    println!("=== JMESPath Array Operations Demo ===\n");
    
    // Create test data
    let json_data = json!([
        {"name": "Alice", "scores": [85, 90, 78], "age": 30},
        {"name": "Bob", "scores": [92, 88, 95], "age": 25},
        {"name": "Charlie", "scores": [78, 85, 88], "age": 35},
        {"name": "Diana", "scores": [95, 92, 98], "age": 28}
    ]);
    
    let rb = json_to_arrow(&json_data);
    
    println!("Test Data:");
    println!("- Alice:   scores=[85, 90, 78], age=30");
    println!("- Bob:     scores=[92, 88, 95], age=25");
    println!("- Charlie: scores=[78, 85, 88], age=35");
    println!("- Diana:   scores=[95, 92, 98], age=28\n");
    
    // Test 1: Array Index
    println!("--- Test 1: Array Index ---");
    test_query("age[0]", &rb);
    test_query("age[1]", &rb);
    test_query("age[-1]", &rb); // Last element
    
    // Test 2: Array Slicing
    println!("\n--- Test 2: Array Slicing ---");
    test_query("age[0:2]", &rb);  // First 2 elements
    test_query("age[1:3]", &rb);  // Middle elements
    test_query("age[:2]", &rb);   // First 2 elements
    test_query("age[2:]", &rb);   // Skip first 2
    
    // Test 3: Array Slicing with Step
    println!("\n--- Test 3: Array Slicing with Step ---");
    test_query("age[::2]", &rb);  // Every 2nd element
    test_query("age[1::2]", &rb); // Every 2nd element starting from index 1
    
    // Test 4: Pipe Operator with Filter
    println!("\n--- Test 4: Pipe Operator ---");
    println!("Note: Pipe operator filters produce RecordBatches");
    test_query("age[?age > 27]", &rb);
    
    // Test 5: Combining Operations
    println!("\n--- Test 5: Combined Operations ---");
    test_query("name[?age >= 30]", &rb);
    test_query("age[1:3]", &rb);
    
    println!("\n=== All array operation tests completed! ===");
}

fn test_query(query: &str, rb: &arrow::record_batch::RecordBatch) {
    print!("{}: ", query);
    
    match parse_jmespath(query) {
        Ok((_rest, expr)) => {
            match eval_jmespath(&expr, rb) {
                result => println!("{:?}", result),
            }
        }
        Err(e) => {
            println!("Parse Error: {:?}", e);
        }
    }
}

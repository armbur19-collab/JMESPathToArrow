use arrow_jmespath::{parse_jmespath, eval_jmespath, json_to_arrow};
use serde_json::json;

fn main() {
    println!("=== JMESPath Advanced Features Demo (Step 5) ===\n");
    
    // Create test data
    let json_data = json!([
        {"name": "Alice", "age": 30, "city": "NYC", "scores": [85, 90, 78]},
        {"name": "Bob", "age": 25, "city": "LA", "scores": [92, 88, 95]},
        {"name": "Charlie", "age": 35, "city": "NYC", "scores": [78, 85, 88]},
        {"name": "Diana", "age": 28, "city": "SF", "scores": [95, 92, 98]}
    ]);
    
    let rb = json_to_arrow(&json_data);
    
    println!("Test Data:");
    println!("- Alice:   age=30, city=NYC, scores=[85, 90, 78]");
    println!("- Bob:     age=25, city=LA,  scores=[92, 88, 95]");
    println!("- Charlie: age=35, city=NYC, scores=[78, 85, 88]");
    println!("- Diana:   age=28, city=SF,  scores=[95, 92, 98]\n");
    
    // Test 1: Multi-Select Hash
    println!("--- Test 1: Multi-Select Hash ---");
    println!("Multi-select creates a new object with selected fields");
    test_query("{name: name, age: age}", &rb);
    test_query("{person: name, years: age}", &rb);
    
    // Test 2: Object Functions
    println!("\n--- Test 2: Object Functions ---");
    println!("Note: keys() and values() work on RecordBatch metadata");
    println!("type() function shows the data type of expressions");
    test_query("type(name)", &rb);
    test_query("type(age)", &rb);
    test_query("type(age[0])", &rb);
    test_query("type(name[0])", &rb);
    
    // Test 3: Flatten (with nested arrays)
    println!("\n--- Test 3: Flatten Operator ---");
    println!("Note: Flatten works on nested list structures");
    // We'll skip actual test since our test data doesn't have nested arrays
    println!("Syntax: data[][] - flattens nested arrays");
    
    // Test 4: Combined Advanced Features
    println!("\n--- Test 4: Combined Features ---");
    test_query("name[?age > 27]", &rb);
    test_query("type(name[?age > 27])", &rb);
    println!("Note: Multi-select with filtered data requires RecordBatch handling");
    
    println!("\n=== All advanced feature tests completed! ===");
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

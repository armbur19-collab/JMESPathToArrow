use arrow_jmespath::*;
use serde_json::json;
use arrow::array::Array;

fn main() {
    println!("=== JMESPath Functions Demo ===\n");

    // Test 1: String functions
    println!("--- Test 1: String Functions ---");
    let data = json!([
        {"name": "Alice Smith", "email": "alice@example.com"},
        {"name": "Bob Jones", "email": "bob@test.com"},
        {"name": "Charlie Brown", "email": "charlie@example.com"}
    ]);
    let rb = json_to_arrow(&data);
    
    // length()
    let (_, ast) = parse_jmespath("length(name)").unwrap();
    let result = eval_jmespath(&ast, &rb);
    println!("length(name): {:?}\n", result);
    
    // contains()
    let (_, ast) = parse_jmespath("contains(email, 'example')").unwrap();
    let result = eval_jmespath(&ast, &rb);
    println!("contains(email, 'example'): {:?}\n", result);
    
    // starts_with()
    let (_, ast) = parse_jmespath("starts_with(name, 'Bob')").unwrap();
    let result = eval_jmespath(&ast, &rb);
    println!("starts_with(name, 'Bob'): {:?}\n", result);
    
    // ends_with()
    let (_, ast) = parse_jmespath("ends_with(email, '.com')").unwrap();
    let result = eval_jmespath(&ast, &rb);
    println!("ends_with(email, '.com'): {:?}\n", result);
    
    // Test 2: Aggregation functions
    println!("--- Test 2: Aggregation Functions ---");
    let data_nums = json!([
        {"product": "A", "price": 19.99, "quantity": 10},
        {"product": "B", "price": 29.99, "quantity": 5},
        {"product": "C", "price": 15.50, "quantity": 20}
    ]);
    let rb_nums = json_to_arrow(&data_nums);
    
    // min()
    let (_, ast) = parse_jmespath("min(quantity)").unwrap();
    let result = eval_jmespath(&ast, &rb_nums);
    println!("min(quantity): {:?}", result);
    
    // max()
    let (_, ast) = parse_jmespath("max(quantity)").unwrap();
    let result = eval_jmespath(&ast, &rb_nums);
    println!("max(quantity): {:?}", result);
    
    // sum()
    let (_, ast) = parse_jmespath("sum(quantity)").unwrap();
    let result = eval_jmespath(&ast, &rb_nums);
    println!("sum(quantity): {:?}", result);
    
    // avg()
    let (_, ast) = parse_jmespath("avg(price)").unwrap();
    let result = eval_jmespath(&ast, &rb_nums);
    println!("avg(price): {:?}\n", result);
    
    // Test 3: abs()
    println!("--- Test 3: abs() Function ---");
    let data_neg = json!([
        {"value": -10},
        {"value": 20},
        {"value": -5}
    ]);
    let rb_neg = json_to_arrow(&data_neg);
    
    let (_, ast) = parse_jmespath("abs(value)").unwrap();
    let result = eval_jmespath(&ast, &rb_neg);
    println!("abs(value): {:?}\n", result);
    
    // Test 4: to_string()
    println!("--- Test 4: to_string() Function ---");
    let (_, ast) = parse_jmespath("to_string(quantity)").unwrap();
    let result = eval_jmespath(&ast, &rb_nums);
    println!("to_string(quantity): {:?}\n", result);
    
    // Test 5: Functions in filters
    println!("--- Test 5: Functions in Filters ---");
    let (_, ast) = parse_jmespath("contains(email, 'example')").unwrap();
    let result = eval_jmespath(&ast, &rb);
    
    if let EvalResult::BoolArray(mask) = result {
        use arrow::compute::filter_record_batch;
        let filtered = filter_record_batch(&rb, &mask).unwrap();
        println!("Filter: contains(email, 'example')");
        println!("Filtered rows: {} (from {})", filtered.num_rows(), rb.num_rows());
        
        // Show filtered names
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
    
    // Test 6: Combining functions with comparisons
    println!("--- Test 6: Functions with Comparisons ---");
    let (_, ast) = parse_jmespath("length(name) > 10").unwrap();
    let result = eval_jmespath(&ast, &rb);
    println!("length(name) > 10: {:?}\n", result);
    
    let (_, ast) = parse_jmespath("max(quantity) > 15").unwrap();
    let result = eval_jmespath(&ast, &rb_nums);
    println!("max(quantity) > 15: {:?}\n", result);
    
    println!("=== All function tests completed! ===");
}

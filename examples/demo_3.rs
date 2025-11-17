use arrow_jmespath::*;
use serde_json::json;

fn main() {
    let data = json!([
        {"name": "Alice", "age": 25, "active": true},
        {"name": "Bob", "age": 30, "active": false},
        {"name": "Charlie", "age": 20, "active": true}
    ]);

    let rb = json_to_arrow(&data);

    // Filter: age > 22 && active == true
    let (_, ast) = parse_jmespath("age > 22 && active == true").unwrap();
    let result = eval_jmespath(&ast, &rb);

    println!("Expression: age > 22 && active == true");
    println!("Result: {:?}", result);
    
    if let EvalResult::BoolArray(mask) = result {
        use arrow::compute::filter_record_batch;
        let filtered = filter_record_batch(&rb, &mask).unwrap();
        println!("Filtered {} rows from {} rows", filtered.num_rows(), rb.num_rows());
    }
}

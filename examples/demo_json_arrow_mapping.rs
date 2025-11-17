use arrow_jmespath::{json_to_arrow, arrow_to_json, arrow_to_json_string, parse_jmespath, eval_jmespath};
use serde_json::json;

fn main() {
    println!("=== JSON ↔ Arrow Mapping Funktionen Demo ===\n");
    
    // ========================================
    // 1. JSON → Arrow
    // ========================================
    println!("--- 1. JSON → Arrow RecordBatch ---");
    
    let json_data = json!([
        {"name": "Alice", "age": 30, "city": "NYC", "salary": 75000},
        {"name": "Bob", "age": 25, "city": "LA", "salary": 65000},
        {"name": "Charlie", "age": 35, "city": "NYC", "salary": 85000}
    ]);
    
    println!("Original JSON:");
    println!("{}\n", serde_json::to_string_pretty(&json_data).unwrap());
    
    // JSON zu Arrow konvertieren
    let record_batch = json_to_arrow(&json_data);
    
    println!("Konvertiert zu Arrow RecordBatch:");
    println!("  - Zeilen: {}", record_batch.num_rows());
    println!("  - Spalten: {}", record_batch.num_columns());
    println!("  - Schema: {:?}\n", record_batch.schema());
    
    // ========================================
    // 2. JMESPath Query auf Arrow
    // ========================================
    println!("--- 2. JMESPath Query auf Arrow ---");
    
    let query = "name[?age > 27]";
    println!("Query: {}", query);
    
    let (_, expr) = parse_jmespath(query).unwrap();
    let result = eval_jmespath(&expr, &record_batch);
    
    println!("Result: {:?}\n", result);
    
    // ========================================
    // 3. Arrow → JSON (zurück konvertieren)
    // ========================================
    println!("--- 3. Arrow → JSON (Zurück-Konvertierung) ---");
    
    // Variante A: Zu JSON Value
    let json_result = arrow_to_json(&record_batch);
    println!("Als JSON Value:");
    println!("{}\n", serde_json::to_string_pretty(&json_result).unwrap());
    
    // Variante B: Direkt zu Pretty String
    let json_string = arrow_to_json_string(&record_batch);
    println!("Als Pretty String:");
    println!("{}\n", json_string);
    
    // ========================================
    // 4. Kompletter Roundtrip
    // ========================================
    println!("--- 4. Kompletter Roundtrip: JSON → Arrow → Query → Arrow → JSON ---");
    
    // Start mit JSON
    let original = json!([
        {"product": "Laptop", "price": 999, "stock": 5},
        {"product": "Mouse", "price": 29, "stock": 150},
        {"product": "Keyboard", "price": 79, "stock": 80}
    ]);
    
    println!("1. Original JSON:");
    println!("{}", serde_json::to_string_pretty(&original).unwrap());
    
    // JSON → Arrow
    let rb = json_to_arrow(&original);
    println!("\n2. Konvertiert zu Arrow: {} rows, {} cols", rb.num_rows(), rb.num_columns());
    
    // JMESPath Query
    let query = "{product: product, price: price}";
    println!("\n3. JMESPath Query: {}", query);
    let (_, expr) = parse_jmespath(query).unwrap();
    let result = eval_jmespath(&expr, &rb);
    
    // Extrahiere RecordBatch aus Result
    if let arrow_jmespath::EvalResult::RecordBatch(result_rb) = result {
        println!("   Result: {} rows, {} cols", result_rb.num_rows(), result_rb.num_columns());
        
        // Arrow → JSON
        let final_json = arrow_to_json_string(&result_rb);
        println!("\n4. Zurück zu JSON:");
        println!("{}", final_json);
    }
    
    // ========================================
    // 5. Mit verschachtelten Daten (Arrays)
    // ========================================
    println!("\n--- 5. Mit verschachtelten Arrays ---");
    
    let nested_data = json!([
        {"name": "Alice", "scores": [85, 90, 78]},
        {"name": "Bob", "scores": [92, 88, 95]}
    ]);
    
    println!("JSON mit Arrays:");
    println!("{}", serde_json::to_string_pretty(&nested_data).unwrap());
    
    let nested_rb = json_to_arrow(&nested_data);
    println!("\nArrow RecordBatch: {} rows", nested_rb.num_rows());
    
    let json_back = arrow_to_json_string(&nested_rb);
    println!("\nZurück zu JSON:");
    println!("{}", json_back);
    
    println!("\n=== Alle Mapping-Funktionen getestet! ===");
}

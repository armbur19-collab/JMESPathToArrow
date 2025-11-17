use arrow_jmespath::{parse_jmespath, eval_jmespath};
use arrow::array::{Int64Array, StringArray, Array};
use arrow::datatypes::{Schema, Field, DataType};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

fn main() {
    println!("=== JMESPath DIREKT auf Arrow RecordBatch (ohne JSON) ===\n");
    
    // ========================================
    // RecordBatch DIREKT erstellen (kein JSON!)
    // ========================================
    
    // 1. Schema definieren
    let schema = Schema::new(vec![
        Field::new("name", DataType::Utf8, false),
        Field::new("age", DataType::Int64, false),
        Field::new("city", DataType::Utf8, false),
        Field::new("salary", DataType::Int64, false),
    ]);
    
    // 2. Spalten-Arrays erstellen
    let names = StringArray::from(vec!["Alice", "Bob", "Charlie", "Diana", "Eve"]);
    let ages = Int64Array::from(vec![30, 25, 35, 28, 42]);
    let cities = StringArray::from(vec!["NYC", "LA", "NYC", "SF", "NYC"]);
    let salaries = Int64Array::from(vec![75000, 65000, 85000, 72000, 95000]);
    
    // 3. RecordBatch erstellen
    let record_batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(names),
            Arc::new(ages),
            Arc::new(cities),
            Arc::new(salaries),
        ],
    ).unwrap();
    
    println!("RecordBatch erstellt mit {} Zeilen und {} Spalten\n", 
             record_batch.num_rows(), 
             record_batch.num_columns());
    
    // ========================================
    // JMESPath Queries ausführen
    // ========================================
    
    println!("--- Query 1: Einfache Spaltenauswahl ---");
    run_query("name", &record_batch);
    
    println!("\n--- Query 2: Filter nach Alter ---");
    run_query("name[?age > 30]", &record_batch);
    
    println!("\n--- Query 3: Array-Slicing ---");
    run_query("age[0:3]", &record_batch);
    
    println!("\n--- Query 4: Multi-Select ---");
    run_query("{name: name, age: age}", &record_batch);
    
    println!("\n--- Query 5: Funktionen ---");
    run_query("max(salary)", &record_batch);
    run_query("avg(age)", &record_batch);
    
    println!("\n--- Query 6: Kombinierte Filter ---");
    run_query("name[?age >= 30 && salary > 70000]", &record_batch);
    
    println!("\n--- Query 7: Index-Zugriff ---");
    run_query("name[0]", &record_batch);
    run_query("age[-1]", &record_batch);
    
    println!("\n=== Fertig! Alles ohne JSON! ===");
}

fn run_query(query: &str, rb: &RecordBatch) {
    print!("Query: {} → ", query);
    
    match parse_jmespath(query) {
        Ok((_rest, expr)) => {
            let result = eval_jmespath(&expr, rb);
            
            // Kompakte Ausgabe
            match result {
                arrow_jmespath::EvalResult::StringArray(arr) => {
                    let values: Vec<&str> = (0..arr.len().min(5))
                        .map(|i| arr.value(i))
                        .collect();
                    println!("StringArray{:?}", values);
                },
                arrow_jmespath::EvalResult::Int64Array(arr) => {
                    let values: Vec<i64> = (0..arr.len().min(5))
                        .map(|i| arr.value(i))
                        .collect();
                    println!("Int64Array{:?}", values);
                },
                arrow_jmespath::EvalResult::Int64Const(v) => {
                    println!("Int64: {}", v);
                },
                arrow_jmespath::EvalResult::FloatConst(v) => {
                    println!("Float: {:.2}", v);
                },
                arrow_jmespath::EvalResult::StringConst(v) => {
                    println!("String: {}", v);
                },
                arrow_jmespath::EvalResult::RecordBatch(rb) => {
                    println!("RecordBatch({} rows, {} cols)", rb.num_rows(), rb.num_columns());
                },
                _ => println!("{:?}", result),
            }
        }
        Err(e) => {
            println!("Parse Error: {:?}", e);
        }
    }
}

use arrow::json::ReaderBuilder;
use arrow::record_batch::RecordBatch;
use serde_json::Value;
use std::io::Cursor;

/// JSON → Arrow RecordBatch (allgemein)
/// Accepts either an array of objects or a single object
pub fn json_to_arrow(json_data: &Value) -> RecordBatch {
    // Convert to newline-delimited JSON format
    let json_lines = match json_data {
        Value::Array(arr) => {
            arr.iter()
                .map(|obj| serde_json::to_string(obj).unwrap())
                .collect::<Vec<_>>()
                .join("\n")
        },
        Value::Object(_) => serde_json::to_string(json_data).unwrap(),
        _ => panic!("JSON data must be an object or array of objects"),
    };
    
    let cursor = Cursor::new(json_lines.as_bytes().to_vec());

    // Schema inferieren und dann den Reader erstellen
    let (schema, _) = arrow::json::reader::infer_json_schema(cursor.clone(), None).unwrap();
    let mut reader = ReaderBuilder::new(std::sync::Arc::new(schema))
        .build(cursor)
        .unwrap();

    // Erstes RecordBatch zurückgeben
    reader.next().unwrap().unwrap()
}

/// Arrow RecordBatch → JSON (Array von Objekten)
pub fn arrow_to_json(record_batch: &RecordBatch) -> Value {
    use arrow::array::*;
    use arrow::datatypes::DataType;
    use serde_json::json;
    
    let schema = record_batch.schema();
    let num_rows = record_batch.num_rows();
    let mut rows = Vec::new();
    
    // Für jede Zeile ein JSON-Objekt erstellen
    for row_idx in 0..num_rows {
        let mut row_obj = serde_json::Map::new();
        
        // Für jede Spalte den Wert extrahieren
        for (col_idx, field) in schema.fields().iter().enumerate() {
            let column = record_batch.column(col_idx);
            let field_name = field.name().clone();
            
            // Wert basierend auf Datentyp extrahieren
            let value = if column.is_null(row_idx) {
                Value::Null
            } else {
                match field.data_type() {
                    DataType::Int32 => {
                        let arr = column.as_any().downcast_ref::<Int32Array>().unwrap();
                        json!(arr.value(row_idx))
                    },
                    DataType::Int64 => {
                        let arr = column.as_any().downcast_ref::<Int64Array>().unwrap();
                        json!(arr.value(row_idx))
                    },
                    DataType::Float32 => {
                        let arr = column.as_any().downcast_ref::<Float32Array>().unwrap();
                        json!(arr.value(row_idx))
                    },
                    DataType::Float64 => {
                        let arr = column.as_any().downcast_ref::<Float64Array>().unwrap();
                        json!(arr.value(row_idx))
                    },
                    DataType::Utf8 => {
                        let arr = column.as_any().downcast_ref::<StringArray>().unwrap();
                        json!(arr.value(row_idx))
                    },
                    DataType::Boolean => {
                        let arr = column.as_any().downcast_ref::<BooleanArray>().unwrap();
                        json!(arr.value(row_idx))
                    },
                    DataType::List(_) => {
                        let arr = column.as_any().downcast_ref::<ListArray>().unwrap();
                        let list = arr.value(row_idx);
                        
                        // Liste zu JSON-Array konvertieren
                        let mut json_array = Vec::new();
                        if let Some(int_arr) = list.as_any().downcast_ref::<Int64Array>() {
                            for i in 0..int_arr.len() {
                                if !int_arr.is_null(i) {
                                    json_array.push(json!(int_arr.value(i)));
                                }
                            }
                        } else if let Some(str_arr) = list.as_any().downcast_ref::<StringArray>() {
                            for i in 0..str_arr.len() {
                                if !str_arr.is_null(i) {
                                    json_array.push(json!(str_arr.value(i)));
                                }
                            }
                        }
                        json!(json_array)
                    },
                    _ => {
                        // Für nicht unterstützte Typen null zurückgeben
                        Value::Null
                    }
                }
            };
            
            row_obj.insert(field_name, value);
        }
        
        rows.push(Value::Object(row_obj));
    }
    
    Value::Array(rows)
}

/// Arrow RecordBatch → Pretty JSON String
pub fn arrow_to_json_string(record_batch: &RecordBatch) -> String {
    let json = arrow_to_json(record_batch);
    serde_json::to_string_pretty(&json).unwrap()
}

/// Arrow RecordBatch → Compact JSON String
pub fn arrow_to_json_string_compact(record_batch: &RecordBatch) -> String {
    let json = arrow_to_json(record_batch);
    serde_json::to_string(&json).unwrap()
}

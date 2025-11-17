use arrow_jmespath::*;
use serde_json::json;

fn main() {
    let data = json!([
        {"a": {"b": {"c": 10}}, "lst": [1,2,3]},
        {"a": {"b": {"c": 5}},  "lst": [2,3,4]}
    ]);

    let rb = json_to_arrow(&data);

    let (_, ast) = parse_jmespath("a.b.c").unwrap();
    let result = eval_jmespath(&ast, &rb);

    println!("Eval result: {:?}", result);
    println!("RecordBatch has {} rows and {} columns", rb.num_rows(), rb.num_columns());
}

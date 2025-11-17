use arrow::array::{Array, StructArray, ListArray, Int32Array, BooleanArray};
use arrow::compute::filter_record_batch;
use arrow::record_batch::RecordBatch;

/// Rekursiver Zugriff auf Int32Array über Pfad "a.b.c"
pub fn get_int32_column<'a>(record_batch: &'a RecordBatch, path: &str) -> &'a Int32Array {
    let mut current: &dyn Array = record_batch
        .column(record_batch.schema().index_of(path.split('.').next().unwrap()).unwrap());

    for key in path.split('.').skip(1) {
        let struct_col = current.as_any().downcast_ref::<StructArray>().unwrap();
        if let arrow::datatypes::DataType::Struct(fields) = struct_col.data_type() {
            let field_index = fields.iter().position(|f| f.name() == key).unwrap();
            current = struct_col.column(field_index);
        }
    }
    current.as_any().downcast_ref::<Int32Array>().unwrap()
}

/// Rekursiver Zugriff auf ListArray über Pfad
pub fn get_list_array<'a>(record_batch: &'a RecordBatch, path: &str) -> &'a ListArray {
    let mut current: &dyn Array = record_batch
        .column(record_batch.schema().index_of(path.split('.').next().unwrap()).unwrap());

    for key in path.split('.').skip(1) {
        let struct_col = current.as_any().downcast_ref::<StructArray>().unwrap();
        if let arrow::datatypes::DataType::Struct(fields) = struct_col.data_type() {
            let field_index = fields.iter().position(|f| f.name() == key).unwrap();
            current = struct_col.column(field_index);
        }
    }
    current.as_any().downcast_ref::<ListArray>().unwrap()
}

/// Boolean-Filter auf RecordBatch
pub fn filter_record_batch_fn(record_batch: &RecordBatch, mask: &BooleanArray) -> RecordBatch {
    filter_record_batch(record_batch, mask).unwrap()
}

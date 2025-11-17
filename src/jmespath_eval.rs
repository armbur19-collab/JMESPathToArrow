use crate::jmespath_ast::*;
use arrow::array::{
    Int32Array, Int64Array, Float32Array, Float64Array, 
    BooleanArray, StringArray, ListArray, StructArray, Array
};
use arrow::compute::{and, or, not, filter_record_batch};
use arrow::compute::kernels::cmp;
use arrow::record_batch::RecordBatch;
use arrow::datatypes::DataType;

/// Evaluator Result - unterstützt verschiedene Array-Typen
#[derive(Debug)]
pub enum EvalResult<'a> {
    // Integer types
    Int32Array(&'a Int32Array),
    Int64Array(&'a Int64Array),
    
    // Float types
    Float32Array(&'a Float32Array),
    Float64Array(&'a Float64Array),
    
    // String types
    StringArray(&'a StringArray),
    
    // Boolean
    BoolArray(BooleanArray),
    
    // Complex types
    ListArray(&'a ListArray),
    RecordBatch(RecordBatch),
    
    // Constants
    IntConst(i32),
    Int64Const(i64),
    FloatConst(f64),
    BoolConst(bool),
    StringConst(String),
}


/// Helper: Get column from RecordBatch by path, returns generic array
fn get_column_by_path<'a>(rb: &'a RecordBatch, path: &[String]) -> &'a dyn Array {
    if path.is_empty() {
        panic!("Empty path");
    }
    
    let mut current: &dyn Array = rb.column(rb.schema().index_of(&path[0]).unwrap());
    
    for key in path.iter().skip(1) {
        let struct_col = current.as_any().downcast_ref::<StructArray>().unwrap();
        if let DataType::Struct(fields) = struct_col.data_type() {
            let field_index = fields.iter().position(|f| f.name() == key).unwrap();
            current = struct_col.column(field_index);
        }
    }
    
    current
}

/// Evaluator
pub fn eval_jmespath<'a>(expr: &JmesPathExpr, rb: &'a RecordBatch) -> EvalResult<'a> {
    // Call internal evaluator with root context
    eval_jmespath_internal(expr, rb, rb)
}

/// Internal evaluator with context support for @ operator
fn eval_jmespath_internal<'a>(expr: &JmesPathExpr, rb: &'a RecordBatch, context: &'a RecordBatch) -> EvalResult<'a> {
    match expr {
        // @ returns the current context
        JmesPathExpr::CurrentNode => {
            // Return the entire context as RecordBatch
            EvalResult::RecordBatch(context.clone())
        },
        
        // & creates an expression reference (stored for later evaluation by functions)
        JmesPathExpr::ExprRef(_inner_expr) => {
            // Expression references are handled by functions like sort_by
            // For now, we just panic - they should only be used in function arguments
            panic!("Expression references (&) can only be used as function arguments");
        },
        
        JmesPathExpr::Path(parts) => {
            let col = get_column_by_path(rb, parts);
            
            // Match on column data type
            match col.data_type() {
                DataType::Int32 => EvalResult::Int32Array(col.as_any().downcast_ref::<Int32Array>().unwrap()),
                DataType::Int64 => EvalResult::Int64Array(col.as_any().downcast_ref::<Int64Array>().unwrap()),
                DataType::Float32 => EvalResult::Float32Array(col.as_any().downcast_ref::<Float32Array>().unwrap()),
                DataType::Float64 => EvalResult::Float64Array(col.as_any().downcast_ref::<Float64Array>().unwrap()),
                DataType::Utf8 => EvalResult::StringArray(col.as_any().downcast_ref::<StringArray>().unwrap()),
                DataType::Boolean => EvalResult::BoolArray(col.as_any().downcast_ref::<BooleanArray>().unwrap().clone()),
                DataType::List(_) => EvalResult::ListArray(col.as_any().downcast_ref::<ListArray>().unwrap()),
                _ => panic!("Unsupported column type: {:?}", col.data_type()),
            }
        },
        
        JmesPathExpr::ConstInt(v) => EvalResult::IntConst(*v),
        JmesPathExpr::ConstFloat(v) => EvalResult::FloatConst(*v),
        JmesPathExpr::ConstBool(v) => EvalResult::BoolConst(*v),
        JmesPathExpr::ConstString(v) => EvalResult::StringConst(v.clone()),
        
        JmesPathExpr::Compare { op, lhs, rhs } => {
            let lhs_val = eval_jmespath(lhs, rb);
            let rhs_val = eval_jmespath(rhs, rb);
            
            // Int32 comparisons
            if let (EvalResult::Int32Array(col), EvalResult::IntConst(c)) = (&lhs_val, &rhs_val) {
                let scalar_value = arrow_array::Scalar::new(Int32Array::from(vec![*c]));
                let mask = match op {
                    CompareOp::Eq => cmp::eq(col, &scalar_value).unwrap(),
                    CompareOp::Ne => cmp::neq(col, &scalar_value).unwrap(),
                    CompareOp::Gt => cmp::gt(col, &scalar_value).unwrap(),
                    CompareOp::Lt => cmp::lt(col, &scalar_value).unwrap(),
                    CompareOp::Gte => cmp::gt_eq(col, &scalar_value).unwrap(),
                    CompareOp::Lte => cmp::lt_eq(col, &scalar_value).unwrap(),
                };
                return EvalResult::BoolArray(mask);
            }
            
            // Int64 comparisons (also accepts Int32 constants)
            if let (EvalResult::Int64Array(col), EvalResult::IntConst(c)) = (&lhs_val, &rhs_val) {
                let scalar_value = arrow_array::Scalar::new(Int64Array::from(vec![*c as i64]));
                let mask = match op {
                    CompareOp::Eq => cmp::eq(col, &scalar_value).unwrap(),
                    CompareOp::Ne => cmp::neq(col, &scalar_value).unwrap(),
                    CompareOp::Gt => cmp::gt(col, &scalar_value).unwrap(),
                    CompareOp::Lt => cmp::lt(col, &scalar_value).unwrap(),
                    CompareOp::Gte => cmp::gt_eq(col, &scalar_value).unwrap(),
                    CompareOp::Lte => cmp::lt_eq(col, &scalar_value).unwrap(),
                };
                return EvalResult::BoolArray(mask);
            }
            
            // Float comparisons (handle both Float32 and Float64)
            if let (EvalResult::Float64Array(col), EvalResult::FloatConst(c)) = (&lhs_val, &rhs_val) {
                let scalar_value = arrow_array::Scalar::new(Float64Array::from(vec![*c]));
                let mask = match op {
                    CompareOp::Eq => cmp::eq(col, &scalar_value).unwrap(),
                    CompareOp::Ne => cmp::neq(col, &scalar_value).unwrap(),
                    CompareOp::Gt => cmp::gt(col, &scalar_value).unwrap(),
                    CompareOp::Lt => cmp::lt(col, &scalar_value).unwrap(),
                    CompareOp::Gte => cmp::gt_eq(col, &scalar_value).unwrap(),
                    CompareOp::Lte => cmp::lt_eq(col, &scalar_value).unwrap(),
                };
                return EvalResult::BoolArray(mask);
            }
            
            // Float32 comparisons
            if let (EvalResult::Float32Array(col), EvalResult::FloatConst(c)) = (&lhs_val, &rhs_val) {
                let scalar_value = arrow_array::Scalar::new(Float32Array::from(vec![*c as f32]));
                let mask = match op {
                    CompareOp::Eq => cmp::eq(col, &scalar_value).unwrap(),
                    CompareOp::Ne => cmp::neq(col, &scalar_value).unwrap(),
                    CompareOp::Gt => cmp::gt(col, &scalar_value).unwrap(),
                    CompareOp::Lt => cmp::lt(col, &scalar_value).unwrap(),
                    CompareOp::Gte => cmp::gt_eq(col, &scalar_value).unwrap(),
                    CompareOp::Lte => cmp::lt_eq(col, &scalar_value).unwrap(),
                };
                return EvalResult::BoolArray(mask);
            }
            
            // String comparisons
            if let (EvalResult::StringArray(col), EvalResult::StringConst(s)) = (&lhs_val, &rhs_val) {
                let scalar_value = arrow_array::Scalar::new(StringArray::from(vec![s.as_str()]));
                let mask = match op {
                    CompareOp::Eq => cmp::eq(col, &scalar_value).unwrap(),
                    CompareOp::Ne => cmp::neq(col, &scalar_value).unwrap(),
                    CompareOp::Gt => cmp::gt(col, &scalar_value).unwrap(),
                    CompareOp::Lt => cmp::lt(col, &scalar_value).unwrap(),
                    CompareOp::Gte => cmp::gt_eq(col, &scalar_value).unwrap(),
                    CompareOp::Lte => cmp::lt_eq(col, &scalar_value).unwrap(),
                };
                return EvalResult::BoolArray(mask);
            }
            
            // Boolean comparisons
            if let (EvalResult::BoolArray(col), EvalResult::BoolConst(b)) = (&lhs_val, &rhs_val) {
                let scalar_value = arrow_array::Scalar::new(BooleanArray::from(vec![*b]));
                let mask = match op {
                    CompareOp::Eq => cmp::eq(col, &scalar_value).unwrap(),
                    CompareOp::Ne => cmp::neq(col, &scalar_value).unwrap(),
                    _ => panic!("Only == and != supported for boolean comparisons"),
                };
                return EvalResult::BoolArray(mask);
            }
            
            // Constant comparisons (Int64Const vs IntConst, etc.)
            if let (EvalResult::Int64Const(a), EvalResult::IntConst(b)) = (&lhs_val, &rhs_val) {
                let result = match op {
                    CompareOp::Eq => a == &(*b as i64),
                    CompareOp::Ne => a != &(*b as i64),
                    CompareOp::Gt => a > &(*b as i64),
                    CompareOp::Lt => a < &(*b as i64),
                    CompareOp::Gte => a >= &(*b as i64),
                    CompareOp::Lte => a <= &(*b as i64),
                };
                return EvalResult::BoolConst(result);
            }
            
            if let (EvalResult::IntConst(a), EvalResult::Int64Const(b)) = (&lhs_val, &rhs_val) {
                let result = match op {
                    CompareOp::Eq => (*a as i64) == *b,
                    CompareOp::Ne => (*a as i64) != *b,
                    CompareOp::Gt => (*a as i64) > *b,
                    CompareOp::Lt => (*a as i64) < *b,
                    CompareOp::Gte => (*a as i64) >= *b,
                    CompareOp::Lte => (*a as i64) <= *b,
                };
                return EvalResult::BoolConst(result);
            }
            
            // IntConst vs IntConst
            if let (EvalResult::IntConst(a), EvalResult::IntConst(b)) = (&lhs_val, &rhs_val) {
                let result = match op {
                    CompareOp::Eq => a == b,
                    CompareOp::Ne => a != b,
                    CompareOp::Gt => a > b,
                    CompareOp::Lt => a < b,
                    CompareOp::Gte => a >= b,
                    CompareOp::Lte => a <= b,
                };
                return EvalResult::BoolConst(result);
            }
            
            panic!("Unsupported comparison types: {:?} {:?} {:?}", lhs_val, op, rhs_val);
        },
        
        JmesPathExpr::Projection(base_expr, _field_expr) => {
            // Evaluate the base expression
            let base_result = eval_jmespath(base_expr, rb);
            
            // For now, we support projection on ListArray
            if let EvalResult::ListArray(_list_arr) = base_result {
                // For each element in the list, evaluate the field expression
                // This is a simplified implementation
                panic!("List projection not yet fully implemented");
            } else {
                panic!("Projection requires a list/array: got {:?}", base_result);
            }
        },
        
        JmesPathExpr::Slice { base, start, stop, step } => {
            let base_result = eval_jmespath(base, rb);
            
            match base_result {
                EvalResult::Int32Array(arr) => {
                    let values = slice_array(&|i| arr.value(i), arr.len(), *start, *stop, *step);
                    // Create new array from sliced values
                    let sliced = Int32Array::from(values);
                    // We need to leak this to get a 'a lifetime
                    return EvalResult::Int32Array(Box::leak(Box::new(sliced)));
                },
                EvalResult::Int64Array(arr) => {
                    let values = slice_array(&|i| arr.value(i), arr.len(), *start, *stop, *step);
                    let sliced = Int64Array::from(values);
                    return EvalResult::Int64Array(Box::leak(Box::new(sliced)));
                },
                EvalResult::StringArray(arr) => {
                    let values: Vec<&str> = slice_array(&|i| arr.value(i), arr.len(), *start, *stop, *step);
                    let sliced = StringArray::from(values);
                    return EvalResult::StringArray(Box::leak(Box::new(sliced)));
                },
                _ => panic!("Slice not supported for this type: {:?}", base_result),
            }
        },
        
        JmesPathExpr::Index(base, idx) => {
            let base_result = eval_jmespath(base, rb);
            
            match base_result {
                EvalResult::Int32Array(arr) => {
                    let index = normalize_index(*idx, arr.len());
                    if arr.is_valid(index) {
                        return EvalResult::IntConst(arr.value(index));
                    } else {
                        panic!("Index {} is null", idx);
                    }
                },
                EvalResult::Int64Array(arr) => {
                    let index = normalize_index(*idx, arr.len());
                    if arr.is_valid(index) {
                        return EvalResult::Int64Const(arr.value(index));
                    } else {
                        panic!("Index {} is null", idx);
                    }
                },
                EvalResult::Float32Array(arr) => {
                    let index = normalize_index(*idx, arr.len());
                    if arr.is_valid(index) {
                        return EvalResult::FloatConst(arr.value(index) as f64);
                    } else {
                        panic!("Index {} is null", idx);
                    }
                },
                EvalResult::Float64Array(arr) => {
                    let index = normalize_index(*idx, arr.len());
                    if arr.is_valid(index) {
                        return EvalResult::FloatConst(arr.value(index));
                    } else {
                        panic!("Index {} is null", idx);
                    }
                },
                EvalResult::StringArray(arr) => {
                    let index = normalize_index(*idx, arr.len());
                    if arr.is_valid(index) {
                        return EvalResult::StringConst(arr.value(index).to_string());
                    } else {
                        panic!("Index {} is null", idx);
                    }
                },
                EvalResult::BoolArray(arr) => {
                    let index = normalize_index(*idx, arr.len());
                    if arr.is_valid(index) {
                        return EvalResult::BoolConst(arr.value(index));
                    } else {
                        panic!("Index {} is null", idx);
                    }
                },
                _ => panic!("Index not supported for this type: {:?}", base_result),
            }
        },
        
        JmesPathExpr::Pipe(left, right) => {
            // Evaluate left side first
            let left_result = eval_jmespath(left, rb);
            
            // If left result is a RecordBatch, use it for right evaluation
            if let EvalResult::RecordBatch(new_rb) = left_result {
                // Leak the RecordBatch to get proper lifetime
                let leaked_rb = Box::leak(Box::new(new_rb));
                return eval_jmespath(right, leaked_rb);
            } else {
                // For other result types, we need to handle differently
                // For now, we'll create a temporary context
                panic!("Pipe operator requires RecordBatch result from left side");
            }
        },
        
        JmesPathExpr::Flatten(base_expr) => {
            let base_result = eval_jmespath(base_expr, rb);
            
            // Flatten only works on ListArray
            if let EvalResult::ListArray(list_arr) = base_result {
                // Flatten the list array by concatenating all inner arrays
                let mut all_values = Vec::new();
                for i in 0..list_arr.len() {
                    if list_arr.is_valid(i) {
                        let inner_arr = list_arr.value(i);
                        // Assuming inner array is Int64Array for now
                        if let Some(int_arr) = inner_arr.as_any().downcast_ref::<Int64Array>() {
                            for j in 0..int_arr.len() {
                                if int_arr.is_valid(j) {
                                    all_values.push(int_arr.value(j));
                                }
                            }
                        }
                    }
                }
                let flattened = Int64Array::from(all_values);
                return EvalResult::Int64Array(Box::leak(Box::new(flattened)));
            } else {
                panic!("Flatten requires a list/array: got {:?}", base_result);
            }
        },
        
        JmesPathExpr::MultiSelectHash(pairs) => {
            // Create a new RecordBatch with selected fields
            use arrow::datatypes::{Schema, Field};
            use std::sync::Arc;
            
            let mut fields = Vec::new();
            let mut columns: Vec<Arc<dyn Array>> = Vec::new();
            
            for (key, expr) in pairs {
                let result = eval_jmespath(expr, rb);
                
                // Convert result to array
                match result {
                    EvalResult::Int32Array(arr) => {
                        fields.push(Field::new(key, DataType::Int32, true));
                        columns.push(Arc::new(arr.clone()) as Arc<dyn Array>);
                    },
                    EvalResult::Int64Array(arr) => {
                        fields.push(Field::new(key, DataType::Int64, true));
                        columns.push(Arc::new(arr.clone()) as Arc<dyn Array>);
                    },
                    EvalResult::StringArray(arr) => {
                        fields.push(Field::new(key, DataType::Utf8, true));
                        columns.push(Arc::new(arr.clone()) as Arc<dyn Array>);
                    },
                    EvalResult::BoolArray(arr) => {
                        fields.push(Field::new(key, DataType::Boolean, true));
                        columns.push(Arc::new(arr) as Arc<dyn Array>);
                    },
                    _ => panic!("Unsupported result type for multi-select: {:?}", result),
                }
            }
            
            let schema = Arc::new(Schema::new(fields));
            let new_rb = RecordBatch::try_new(schema, columns).unwrap();
            return EvalResult::RecordBatch(new_rb);
        },
        
        JmesPathExpr::MultiSelectList(_exprs) => {
            // For now, return error - this requires more complex handling
            panic!("MultiSelectList not yet fully implemented");
        },
        
        JmesPathExpr::Filter(_array_expr, cond_expr) => {
            // Evaluate array expression - should be a path to the RecordBatch
            // For now, we assume the filter is applied to the entire RecordBatch
            let condition_result = eval_jmespath(cond_expr, rb);
            
            if let EvalResult::BoolArray(mask) = condition_result {
                // Filter the RecordBatch using the boolean mask
                let filtered = filter_record_batch(rb, &mask).unwrap();
                return EvalResult::RecordBatch(filtered);
            } else {
                panic!("Filter condition must evaluate to BoolArray");
            }
        },
        
        JmesPathExpr::Logic { op, lhs, rhs } => {
            let l = eval_jmespath(lhs, rb);
            
            match op {
                LogicOp::Not => {
                    if let EvalResult::BoolArray(a) = l {
                        return EvalResult::BoolArray(not(&a).unwrap());
                    } else if let EvalResult::BoolConst(b) = l {
                        return EvalResult::BoolConst(!b);
                    }
                    panic!("NOT requires boolean operand");
                },
                LogicOp::And | LogicOp::Or => {
                    let r = rhs.as_ref().map(|r| eval_jmespath(r, rb)).expect("AND/OR require two operands");
                    
                    match (l, r) {
                        (EvalResult::BoolArray(a), EvalResult::BoolArray(b)) => {
                            let result = match op {
                                LogicOp::And => and(&a, &b).unwrap(),
                                LogicOp::Or => or(&a, &b).unwrap(),
                                _ => unreachable!(),
                            };
                            EvalResult::BoolArray(result)
                        },
                        (EvalResult::BoolConst(a), EvalResult::BoolConst(b)) => {
                            let result = match op {
                                LogicOp::And => a && b,
                                LogicOp::Or => a || b,
                                _ => unreachable!(),
                            };
                            EvalResult::BoolConst(result)
                        },
                        _ => panic!("Logic operators require boolean operands"),
                    }
                }
            }
        },
        
        JmesPathExpr::Func { name, args } => {
            eval_function(name, args, rb)
        },
    }
}

// ============================================================================
// FUNCTION IMPLEMENTATIONS
// ============================================================================

/// Evaluate a function call
fn eval_function<'a>(name: &str, args: &[JmesPathExpr], rb: &'a RecordBatch) -> EvalResult<'a> {
    match name {
        "length" => func_length(args, rb),
        "contains" => func_contains(args, rb),
        "starts_with" => func_starts_with(args, rb),
        "ends_with" => func_ends_with(args, rb),
        "to_string" => func_to_string(args, rb),
        "min" => func_min(args, rb),
        "max" => func_max(args, rb),
        "sum" => func_sum(args, rb),
        "avg" => func_avg(args, rb),
        "abs" => func_abs(args, rb),
        "keys" => func_keys(args, rb),
        "values" => func_values(args, rb),
        "type" => func_type(args, rb),
        "sort_by" => func_sort_by(args, rb),
        "group_by" => func_group_by(args, rb),
        "reverse" => func_reverse(args, rb),
        "not_null" => func_not_null(args, rb),
        _ => panic!("Unknown function: {}", name),
    }
}

/// length(array) or length(string) - returns length
fn func_length<'a>(args: &[JmesPathExpr], rb: &'a RecordBatch) -> EvalResult<'a> {
    if args.len() != 1 {
        panic!("length() requires exactly 1 argument");
    }
    
    let arg = eval_jmespath(&args[0], rb);
    
    match arg {
        EvalResult::StringArray(arr) => {
            // String length for each element
            let lengths: Vec<i32> = (0..arr.len())
                .map(|i| if arr.is_null(i) { 0 } else { arr.value(i).len() as i32 })
                .collect();
            let result = Int32Array::from(lengths);
            // We need to return a reference, so we leak it (not ideal but works for demo)
            EvalResult::Int32Array(Box::leak(Box::new(result)))
        },
        EvalResult::ListArray(_arr) => {
            // Array length
            // For now, return constant
            // TODO: Implement proper list length
            panic!("length() for arrays not yet fully implemented");
        },
        EvalResult::StringConst(s) => {
            EvalResult::IntConst(s.len() as i32)
        },
        _ => panic!("length() requires string or array argument"),
    }
}

/// contains(string, substring) - checks if string contains substring
fn func_contains<'a>(args: &[JmesPathExpr], rb: &'a RecordBatch) -> EvalResult<'a> {
    if args.len() != 2 {
        panic!("contains() requires exactly 2 arguments");
    }
    
    let haystack = eval_jmespath(&args[0], rb);
    let needle = eval_jmespath(&args[1], rb);
    
    if let (EvalResult::StringArray(arr), EvalResult::StringConst(search)) = (haystack, needle) {
        let results: Vec<bool> = (0..arr.len())
            .map(|i| if arr.is_null(i) { false } else { arr.value(i).contains(&search) })
            .collect();
        let result = BooleanArray::from(results);
        return EvalResult::BoolArray(result);
    }
    
    panic!("contains() requires (string_array, string_const) arguments");
}

/// starts_with(string, prefix) - checks if string starts with prefix
fn func_starts_with<'a>(args: &[JmesPathExpr], rb: &'a RecordBatch) -> EvalResult<'a> {
    if args.len() != 2 {
        panic!("starts_with() requires exactly 2 arguments");
    }
    
    let haystack = eval_jmespath(&args[0], rb);
    let prefix = eval_jmespath(&args[1], rb);
    
    if let (EvalResult::StringArray(arr), EvalResult::StringConst(search)) = (haystack, prefix) {
        let results: Vec<bool> = (0..arr.len())
            .map(|i| if arr.is_null(i) { false } else { arr.value(i).starts_with(&search) })
            .collect();
        let result = BooleanArray::from(results);
        return EvalResult::BoolArray(result);
    }
    
    panic!("starts_with() requires (string_array, string_const) arguments");
}

/// ends_with(string, suffix) - checks if string ends with suffix
fn func_ends_with<'a>(args: &[JmesPathExpr], rb: &'a RecordBatch) -> EvalResult<'a> {
    if args.len() != 2 {
        panic!("ends_with() requires exactly 2 arguments");
    }
    
    let haystack = eval_jmespath(&args[0], rb);
    let suffix = eval_jmespath(&args[1], rb);
    
    if let (EvalResult::StringArray(arr), EvalResult::StringConst(search)) = (haystack, suffix) {
        let results: Vec<bool> = (0..arr.len())
            .map(|i| if arr.is_null(i) { false } else { arr.value(i).ends_with(&search) })
            .collect();
        let result = BooleanArray::from(results);
        return EvalResult::BoolArray(result);
    }
    
    panic!("ends_with() requires (string_array, string_const) arguments");
}

/// to_string(value) - converts value to string
fn func_to_string<'a>(args: &[JmesPathExpr], rb: &'a RecordBatch) -> EvalResult<'a> {
    if args.len() != 1 {
        panic!("to_string() requires exactly 1 argument");
    }
    
    let arg = eval_jmespath(&args[0], rb);
    
    match arg {
        EvalResult::Int32Array(arr) => {
            let strings: Vec<String> = (0..arr.len())
                .map(|i| if arr.is_null(i) { "".to_string() } else { arr.value(i).to_string() })
                .collect();
            let result = StringArray::from(strings);
            EvalResult::StringArray(Box::leak(Box::new(result)))
        },
        EvalResult::Int64Array(arr) => {
            let strings: Vec<String> = (0..arr.len())
                .map(|i| if arr.is_null(i) { "".to_string() } else { arr.value(i).to_string() })
                .collect();
            let result = StringArray::from(strings);
            EvalResult::StringArray(Box::leak(Box::new(result)))
        },
        EvalResult::IntConst(v) => EvalResult::StringConst(v.to_string()),
        _ => panic!("to_string() not implemented for this type"),
    }
}

/// min(array) - returns minimum value
fn func_min<'a>(args: &[JmesPathExpr], rb: &'a RecordBatch) -> EvalResult<'a> {
    if args.len() != 1 {
        panic!("min() requires exactly 1 argument");
    }
    
    let arg = eval_jmespath(&args[0], rb);
    
    match arg {
        EvalResult::Int32Array(arr) => {
            use arrow::compute::min;
            if let Some(min_val) = min(arr) {
                EvalResult::IntConst(min_val)
            } else {
                panic!("Cannot compute min of empty array");
            }
        },
        EvalResult::Int64Array(arr) => {
            use arrow::compute::min;
            if let Some(min_val) = min(arr) {
                EvalResult::Int64Const(min_val)
            } else {
                panic!("Cannot compute min of empty array");
            }
        },
        EvalResult::Float64Array(arr) => {
            use arrow::compute::min;
            if let Some(min_val) = min(arr) {
                EvalResult::FloatConst(min_val)
            } else {
                panic!("Cannot compute min of empty array");
            }
        },
        _ => panic!("min() requires numeric array"),
    }
}

/// max(array) - returns maximum value
fn func_max<'a>(args: &[JmesPathExpr], rb: &'a RecordBatch) -> EvalResult<'a> {
    if args.len() != 1 {
        panic!("max() requires exactly 1 argument");
    }
    
    let arg = eval_jmespath(&args[0], rb);
    
    match arg {
        EvalResult::Int32Array(arr) => {
            use arrow::compute::max;
            if let Some(max_val) = max(arr) {
                EvalResult::IntConst(max_val)
            } else {
                panic!("Cannot compute max of empty array");
            }
        },
        EvalResult::Int64Array(arr) => {
            use arrow::compute::max;
            if let Some(max_val) = max(arr) {
                EvalResult::Int64Const(max_val)
            } else {
                panic!("Cannot compute max of empty array");
            }
        },
        EvalResult::Float64Array(arr) => {
            use arrow::compute::max;
            if let Some(max_val) = max(arr) {
                EvalResult::FloatConst(max_val)
            } else {
                panic!("Cannot compute max of empty array");
            }
        },
        _ => panic!("max() requires numeric array"),
    }
}

/// sum(array) - returns sum of all values
fn func_sum<'a>(args: &[JmesPathExpr], rb: &'a RecordBatch) -> EvalResult<'a> {
    if args.len() != 1 {
        panic!("sum() requires exactly 1 argument");
    }
    
    let arg = eval_jmespath(&args[0], rb);
    
    match arg {
        EvalResult::Int32Array(arr) => {
            use arrow::compute::sum;
            if let Some(sum_val) = sum(arr) {
                EvalResult::IntConst(sum_val as i32)
            } else {
                EvalResult::IntConst(0)
            }
        },
        EvalResult::Int64Array(arr) => {
            use arrow::compute::sum;
            if let Some(sum_val) = sum(arr) {
                EvalResult::Int64Const(sum_val)
            } else {
                EvalResult::Int64Const(0)
            }
        },
        EvalResult::Float64Array(arr) => {
            use arrow::compute::sum;
            if let Some(sum_val) = sum(arr) {
                EvalResult::FloatConst(sum_val)
            } else {
                EvalResult::FloatConst(0.0)
            }
        },
        _ => panic!("sum() requires numeric array"),
    }
}

/// avg(array) - returns average of all values
fn func_avg<'a>(args: &[JmesPathExpr], rb: &'a RecordBatch) -> EvalResult<'a> {
    if args.len() != 1 {
        panic!("avg() requires exactly 1 argument");
    }
    
    let arg = eval_jmespath(&args[0], rb);
    
    match arg {
        EvalResult::Int32Array(arr) => {
            use arrow::compute::sum;
            if let Some(sum_val) = sum(arr) {
                let avg = sum_val as f64 / arr.len() as f64;
                EvalResult::FloatConst(avg)
            } else {
                EvalResult::FloatConst(0.0)
            }
        },
        EvalResult::Int64Array(arr) => {
            use arrow::compute::sum;
            if let Some(sum_val) = sum(arr) {
                let avg = sum_val as f64 / arr.len() as f64;
                EvalResult::FloatConst(avg)
            } else {
                EvalResult::FloatConst(0.0)
            }
        },
        EvalResult::Float64Array(arr) => {
            use arrow::compute::sum;
            if let Some(sum_val) = sum(arr) {
                let avg = sum_val / arr.len() as f64;
                EvalResult::FloatConst(avg)
            } else {
                EvalResult::FloatConst(0.0)
            }
        },
        _ => panic!("avg() requires numeric array"),
    }
}

/// abs(value) - returns absolute value
fn func_abs<'a>(args: &[JmesPathExpr], rb: &'a RecordBatch) -> EvalResult<'a> {
    if args.len() != 1 {
        panic!("abs() requires exactly 1 argument");
    }
    
    let arg = eval_jmespath(&args[0], rb);
    
    match arg {
        EvalResult::Int32Array(arr) => {
            let values: Vec<i32> = (0..arr.len())
                .map(|i| if arr.is_null(i) { 0 } else { arr.value(i).abs() })
                .collect();
            let result = Int32Array::from(values);
            EvalResult::Int32Array(Box::leak(Box::new(result)))
        },
        EvalResult::Int64Array(arr) => {
            let values: Vec<i64> = (0..arr.len())
                .map(|i| if arr.is_null(i) { 0 } else { arr.value(i).abs() })
                .collect();
            let result = Int64Array::from(values);
            EvalResult::Int64Array(Box::leak(Box::new(result)))
        },
        EvalResult::Float64Array(arr) => {
            let values: Vec<f64> = (0..arr.len())
                .map(|i| if arr.is_null(i) { 0.0 } else { arr.value(i).abs() })
                .collect();
            let result = Float64Array::from(values);
            EvalResult::Float64Array(Box::leak(Box::new(result)))
        },
        EvalResult::IntConst(v) => EvalResult::IntConst(v.abs()),
        EvalResult::FloatConst(v) => EvalResult::FloatConst(v.abs()),
        _ => panic!("abs() requires numeric argument"),
    }
}

// ============================================================================
// HELPER FUNCTIONS FOR ARRAY OPERATIONS
// ============================================================================

/// Normalize negative indices: -1 means last element, -2 means second-to-last, etc.
fn normalize_index(idx: i32, len: usize) -> usize {
    if idx < 0 {
        let pos = len as i32 + idx;
        if pos < 0 {
            panic!("Index {} out of bounds for length {}", idx, len);
        }
        pos as usize
    } else {
        if idx as usize >= len {
            panic!("Index {} out of bounds for length {}", idx, len);
        }
        idx as usize
    }
}

/// Generic slice helper for Arrow arrays
fn slice_array<T: Clone>(arr: &dyn Fn(usize) -> T, len: usize, start: Option<i32>, stop: Option<i32>, step: Option<i32>) -> Vec<T> {
    let arr_len = len as i32;
    let step_val = step.unwrap_or(1);
    
    if step_val == 0 {
        panic!("Slice step cannot be zero");
    }
    
    let (start_idx, stop_idx) = if step_val > 0 {
        let s = start.unwrap_or(0);
        let e = stop.unwrap_or(arr_len);
        let s = if s < 0 { (arr_len + s).max(0) } else { s.min(arr_len) };
        let e = if e < 0 { (arr_len + e).max(0) } else { e.min(arr_len) };
        (s, e)
    } else {
        let s = start.unwrap_or(arr_len - 1);
        let e = stop.unwrap_or(-arr_len - 1);
        let s = if s < 0 { (arr_len + s).max(-1) } else { s.min(arr_len - 1) };
        let e = if e < 0 { (arr_len + e).max(-1) } else { e.min(arr_len - 1) };
        (s, e)
    };
    
    let mut result = Vec::new();
    
    if step_val > 0 {
        let mut i = start_idx;
        while i < stop_idx {
            if i >= 0 && (i as usize) < len {
                result.push(arr(i as usize));
            }
            i += step_val;
        }
    } else {
        let mut i = start_idx;
        while i > stop_idx {
            if i >= 0 && (i as usize) < len {
                result.push(arr(i as usize));
            }
            i += step_val; // step_val is negative
        }
    }
    
    result
}

// ============================================================================
// OBJECT FUNCTIONS
// ============================================================================

/// keys(object) - returns array of field names from RecordBatch
fn func_keys<'a>(_args: &[JmesPathExpr], rb: &'a RecordBatch) -> EvalResult<'a> {
    let schema = rb.schema();
    let field_names: Vec<String> = schema
        .fields()
        .iter()
        .map(|f| f.name().clone())
        .collect();
    
    let field_name_refs: Vec<&str> = field_names.iter().map(|s| s.as_str()).collect();
    let keys_array = StringArray::from(field_name_refs);
    EvalResult::StringArray(Box::leak(Box::new(keys_array)))
}

/// values(object) - returns first row values (simplified implementation)
fn func_values<'a>(_args: &[JmesPathExpr], rb: &'a RecordBatch) -> EvalResult<'a> {
    // For simplicity, return the number of columns
    // A full implementation would need to handle multiple rows and types
    let num_fields = rb.schema().fields().len() as i32;
    EvalResult::IntConst(num_fields)
}

/// type(expr) - returns type of expression as string
fn func_type<'a>(args: &[JmesPathExpr], rb: &'a RecordBatch) -> EvalResult<'a> {
    if args.len() != 1 {
        panic!("type() requires exactly 1 argument");
    }
    
    let arg = eval_jmespath(&args[0], rb);
    
    let type_name = match arg {
        EvalResult::Int32Array(_) => "array<int32>",
        EvalResult::Int64Array(_) => "array<int64>",
        EvalResult::Float32Array(_) => "array<float32>",
        EvalResult::Float64Array(_) => "array<float64>",
        EvalResult::StringArray(_) => "array<string>",
        EvalResult::BoolArray(_) => "array<boolean>",
        EvalResult::ListArray(_) => "array<list>",
        EvalResult::RecordBatch(_) => "object",
        EvalResult::IntConst(_) => "number",
        EvalResult::Int64Const(_) => "number",
        EvalResult::FloatConst(_) => "number",
        EvalResult::BoolConst(_) => "boolean",
        EvalResult::StringConst(_) => "string",
    };
    
    EvalResult::StringConst(type_name.to_string())
}

// ============================================================================
// ADVANCED FUNCTIONS (sort_by, group_by)
// ============================================================================

/// sort_by(array, &expr) - sort RecordBatch by expression (typically a field)
/// Example: sort_by(people, &age) sorts people by age field
fn func_sort_by<'a>(args: &[JmesPathExpr], rb: &'a RecordBatch) -> EvalResult<'a> {
    if args.len() != 2 {
        panic!("sort_by() requires exactly 2 arguments: sort_by(array, &expr)");
    }
    
    // First argument should evaluate to RecordBatch (or we use the current one)
    let data_result = eval_jmespath(&args[0], rb);
    
    // Second argument should be an ExprRef
    let sort_field = match &args[1] {
        JmesPathExpr::ExprRef(inner) => {
            // Extract field name from the expression
            match inner.as_ref() {
                JmesPathExpr::Path(parts) if parts.len() == 1 => &parts[0],
                _ => panic!("sort_by() requires a simple field reference like &age"),
            }
        },
        _ => panic!("sort_by() second argument must be an expression reference (&field)"),
    };
    
    // Get the RecordBatch to sort
    let rb_to_sort = match data_result {
        EvalResult::RecordBatch(batch) => batch,
        _ => rb.clone(), // Use input RecordBatch if first arg doesn't evaluate to RecordBatch
    };
    
    // Get the column to sort by
    let sort_col_idx = rb_to_sort.schema().index_of(sort_field)
        .unwrap_or_else(|_| panic!("Field '{}' not found in RecordBatch", sort_field));
    
    // Create sort indices based on the column type
    use arrow::compute::sort_to_indices;
    let sort_col = rb_to_sort.column(sort_col_idx);
    let sort_options = None; // Use default options (ascending, nulls first)
    
    let indices = sort_to_indices(sort_col, sort_options, None)
        .unwrap_or_else(|e| panic!("Failed to create sort indices: {}", e));
    
    // Apply indices to all columns
    use arrow::compute::take;
    let sorted_columns: Vec<_> = (0..rb_to_sort.num_columns())
        .map(|i| {
            take(rb_to_sort.column(i).as_ref(), &indices, None)
                .unwrap_or_else(|e| panic!("Failed to apply sort indices: {}", e))
        })
        .collect();
    
    // Create new RecordBatch
    use arrow::record_batch::RecordBatch as RB;
    let sorted_rb = RB::try_new(rb_to_sort.schema(), sorted_columns)
        .unwrap_or_else(|e| panic!("Failed to create sorted RecordBatch: {}", e));
    
    EvalResult::RecordBatch(sorted_rb)
}

/// group_by(array, &expr) - group RecordBatch by expression (simplified: returns unique values)
/// Example: group_by(people, &city) returns unique cities
/// Note: Full grouping would require aggregation functions
fn func_group_by<'a>(args: &[JmesPathExpr], rb: &'a RecordBatch) -> EvalResult<'a> {
    if args.len() != 2 {
        panic!("group_by() requires exactly 2 arguments: group_by(array, &expr)");
    }
    
    // First argument should evaluate to RecordBatch (or we use the current one)
    let data_result = eval_jmespath(&args[0], rb);
    
    // Second argument should be an ExprRef
    let group_field = match &args[1] {
        JmesPathExpr::ExprRef(inner) => {
            // Extract field name from the expression
            match inner.as_ref() {
                JmesPathExpr::Path(parts) if parts.len() == 1 => &parts[0],
                _ => panic!("group_by() requires a simple field reference like &category"),
            }
        },
        _ => panic!("group_by() second argument must be an expression reference (&field)"),
    };
    
    // Get the RecordBatch to group
    let rb_to_group = match data_result {
        EvalResult::RecordBatch(batch) => batch,
        _ => rb.clone(), // Use input RecordBatch if first arg doesn't evaluate to RecordBatch
    };
    
    // Get the column to group by
    let group_col_idx = rb_to_group.schema().index_of(group_field)
        .unwrap_or_else(|_| panic!("Field '{}' not found in RecordBatch", group_field));
    
    let group_col = rb_to_group.column(group_col_idx);
    
    // For now, just return unique values from that column
    // A full implementation would need aggregation support
    // We'll use a HashSet to track unique values
    use std::collections::HashSet;
    
    match group_col.data_type() {
        DataType::Int32 => {
            let arr = group_col.as_any().downcast_ref::<Int32Array>().unwrap();
            let mut unique_vals = HashSet::new();
            for i in 0..arr.len() {
                if !arr.is_null(i) {
                    unique_vals.insert(arr.value(i));
                }
            }
            let mut sorted_vals: Vec<i32> = unique_vals.into_iter().collect();
            sorted_vals.sort();
            let result_arr = Int32Array::from(sorted_vals);
            EvalResult::Int32Array(Box::leak(Box::new(result_arr)))
        },
        DataType::Int64 => {
            let arr = group_col.as_any().downcast_ref::<Int64Array>().unwrap();
            let mut unique_vals = HashSet::new();
            for i in 0..arr.len() {
                if !arr.is_null(i) {
                    unique_vals.insert(arr.value(i));
                }
            }
            let mut sorted_vals: Vec<i64> = unique_vals.into_iter().collect();
            sorted_vals.sort();
            let result_arr = Int64Array::from(sorted_vals);
            EvalResult::Int64Array(Box::leak(Box::new(result_arr)))
        },
        DataType::Utf8 => {
            let arr = group_col.as_any().downcast_ref::<StringArray>().unwrap();
            let mut unique_vals = HashSet::new();
            for i in 0..arr.len() {
                if !arr.is_null(i) {
                    unique_vals.insert(arr.value(i).to_string());
                }
            }
            let mut sorted_vals: Vec<String> = unique_vals.into_iter().collect();
            sorted_vals.sort();
            let sorted_refs: Vec<&str> = sorted_vals.iter().map(|s| s.as_str()).collect();
            let result_arr = StringArray::from(sorted_refs);
            EvalResult::StringArray(Box::leak(Box::new(result_arr)))
        },
        _ => panic!("group_by() not supported for type: {:?}", group_col.data_type()),
    }
}

// ============================================================================
// UTILITY FUNCTIONS (reverse, not_null)
// ============================================================================

/// reverse(array) - reverses the order of array elements or RecordBatch rows
/// Example: reverse(sort_by(@, &price)) - descending sort
fn func_reverse<'a>(args: &[JmesPathExpr], rb: &'a RecordBatch) -> EvalResult<'a> {
    if args.len() != 1 {
        panic!("reverse() requires exactly 1 argument");
    }
    
    let arg = eval_jmespath(&args[0], rb);
    
    match arg {
        EvalResult::Int32Array(arr) => {
            let values: Vec<i32> = (0..arr.len()).rev().map(|i| arr.value(i)).collect();
            let reversed = Int32Array::from(values);
            EvalResult::Int32Array(Box::leak(Box::new(reversed)))
        },
        EvalResult::Int64Array(arr) => {
            let values: Vec<i64> = (0..arr.len()).rev().map(|i| arr.value(i)).collect();
            let reversed = Int64Array::from(values);
            EvalResult::Int64Array(Box::leak(Box::new(reversed)))
        },
        EvalResult::Float32Array(arr) => {
            let values: Vec<f32> = (0..arr.len()).rev().map(|i| arr.value(i)).collect();
            let reversed = Float32Array::from(values);
            EvalResult::Float32Array(Box::leak(Box::new(reversed)))
        },
        EvalResult::Float64Array(arr) => {
            let values: Vec<f64> = (0..arr.len()).rev().map(|i| arr.value(i)).collect();
            let reversed = Float64Array::from(values);
            EvalResult::Float64Array(Box::leak(Box::new(reversed)))
        },
        EvalResult::StringArray(arr) => {
            let values: Vec<&str> = (0..arr.len()).rev().map(|i| arr.value(i)).collect();
            let reversed = StringArray::from(values);
            EvalResult::StringArray(Box::leak(Box::new(reversed)))
        },
        EvalResult::BoolArray(arr) => {
            let values: Vec<bool> = (0..arr.len()).rev().map(|i| arr.value(i)).collect();
            let reversed = BooleanArray::from(values);
            EvalResult::BoolArray(reversed)
        },
        EvalResult::RecordBatch(batch) => {
            // Reverse all rows in the RecordBatch
            let num_rows = batch.num_rows();
            let reversed_indices: Vec<usize> = (0..num_rows).rev().collect();
            
            // Create indices array for take operation
            use arrow::array::UInt64Array;
            let indices = UInt64Array::from(reversed_indices.iter().map(|&i| i as u64).collect::<Vec<u64>>());
            
            // Apply take to all columns
            use arrow::compute::take;
            let reversed_columns: Vec<_> = (0..batch.num_columns())
                .map(|i| {
                    take(batch.column(i).as_ref(), &indices, None)
                        .unwrap_or_else(|e| panic!("Failed to reverse RecordBatch: {}", e))
                })
                .collect();
            
            // Create new RecordBatch
            use arrow::record_batch::RecordBatch as RB;
            let reversed_rb = RB::try_new(batch.schema(), reversed_columns)
                .unwrap_or_else(|e| panic!("Failed to create reversed RecordBatch: {}", e));
            
            EvalResult::RecordBatch(reversed_rb)
        },
        _ => panic!("reverse() not supported for this type: {:?}", arg),
    }
}

/// not_null(field) - filters out null values from a column, returns RecordBatch with non-null rows
/// Example: not_null(email) - only rows where email is not null
fn func_not_null<'a>(args: &[JmesPathExpr], rb: &'a RecordBatch) -> EvalResult<'a> {
    if args.len() != 1 {
        panic!("not_null() requires exactly 1 argument");
    }
    
    // Evaluate the argument to get the column
    let arg = eval_jmespath(&args[0], rb);
    
    // Create boolean mask: true where NOT null
    let mask = match arg {
        EvalResult::Int32Array(arr) => {
            let null_mask: Vec<bool> = (0..arr.len()).map(|i| !arr.is_null(i)).collect();
            BooleanArray::from(null_mask)
        },
        EvalResult::Int64Array(arr) => {
            let null_mask: Vec<bool> = (0..arr.len()).map(|i| !arr.is_null(i)).collect();
            BooleanArray::from(null_mask)
        },
        EvalResult::Float32Array(arr) => {
            let null_mask: Vec<bool> = (0..arr.len()).map(|i| !arr.is_null(i)).collect();
            BooleanArray::from(null_mask)
        },
        EvalResult::Float64Array(arr) => {
            let null_mask: Vec<bool> = (0..arr.len()).map(|i| !arr.is_null(i)).collect();
            BooleanArray::from(null_mask)
        },
        EvalResult::StringArray(arr) => {
            let null_mask: Vec<bool> = (0..arr.len()).map(|i| !arr.is_null(i)).collect();
            BooleanArray::from(null_mask)
        },
        EvalResult::BoolArray(arr) => {
            let null_mask: Vec<bool> = (0..arr.len()).map(|i| !arr.is_null(i)).collect();
            BooleanArray::from(null_mask)
        },
        EvalResult::ListArray(arr) => {
            let null_mask: Vec<bool> = (0..arr.len()).map(|i| !arr.is_null(i)).collect();
            BooleanArray::from(null_mask)
        },
        _ => panic!("not_null() requires an array column, got: {:?}", arg),
    };
    
    // Filter the RecordBatch using the mask
    let filtered = filter_record_batch(rb, &mask)
        .unwrap_or_else(|e| panic!("Failed to filter null values: {}", e));
    
    EvalResult::RecordBatch(filtered)
}



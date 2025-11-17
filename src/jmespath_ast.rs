#[derive(Debug, Clone)]
pub enum JmesPathExpr {
    Path(Vec<String>),
    Filter(Box<JmesPathExpr>, Box<JmesPathExpr>),
    Compare { op: CompareOp, lhs: Box<JmesPathExpr>, rhs: Box<JmesPathExpr> },
    Logic { op: LogicOp, lhs: Box<JmesPathExpr>, rhs: Option<Box<JmesPathExpr>> },
    Func { name: String, args: Vec<JmesPathExpr> },
    // Array operations
    Projection(Box<JmesPathExpr>, Box<JmesPathExpr>), // base[*].field
    Slice { base: Box<JmesPathExpr>, start: Option<i32>, stop: Option<i32>, step: Option<i32> }, // array[0:5:1]
    Index(Box<JmesPathExpr>, i32), // array[0]
    Pipe(Box<JmesPathExpr>, Box<JmesPathExpr>), // expr | expr
    Flatten(Box<JmesPathExpr>), // expr[]
    // Multi-select
    MultiSelectHash(Vec<(String, JmesPathExpr)>), // {key1: expr1, key2: expr2}
    MultiSelectList(Vec<JmesPathExpr>), // [expr1, expr2, expr3]
    // Special operators
    CurrentNode, // @ - refers to current object in filter/projection
    ExprRef(Box<JmesPathExpr>), // & - expression reference for functions like sort_by
    // Constants
    ConstInt(i32),
    ConstFloat(f64),
    ConstBool(bool),
    ConstString(String),
}

#[derive(Debug, Clone)]
pub enum CompareOp {
    Eq, Ne, Gt, Lt, Gte, Lte
}

#[derive(Debug, Clone)]
pub enum LogicOp {
    And, Or, Not
}

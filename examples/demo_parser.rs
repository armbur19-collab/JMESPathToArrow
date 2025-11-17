use arrow_jmespath::parse_jmespath;

fn main() {
    println!("=== JMESPath Parser Demo ===\n");

    // Test verschiedene Parser-Features
    let test_cases = vec![
        // Basics
        ("42", "Integer literal"),
        ("3.14", "Float literal"),
        ("true", "Boolean literal"),
        ("\"hello\"", "String literal (double quotes)"),
        ("'world'", "String literal (single quotes)"),
        
        // Paths
        ("name", "Simple path"),
        ("user.address.city", "Nested path"),
        ("user_name", "Path with underscore"),
        
        // Comparisons
        ("age > 18", "Greater than"),
        ("status == 'active'", "Equals string"),
        ("score >= 100", "Greater or equal"),
        ("price <= 50.0", "Less or equal"),
        ("name != 'test'", "Not equals"),
        
        // Logic
        ("age > 18 && status == 'active'", "AND logic"),
        ("age < 18 || age > 65", "OR logic"),
        ("!active", "NOT logic"),
        ("(age > 18)", "Parentheses"),
        ("age > 18 && (status == 'active' || premium == true)", "Complex logic"),
        
        // Filters
        ("users[?age > 18]", "Simple filter"),
        ("items[?status == 'active']", "Filter with string"),
        ("items[?age > 18 && status == 'active']", "Filter with logic"),
    ];

    for (expr, description) in test_cases {
        match parse_jmespath(expr) {
            Ok((remaining, ast)) => {
                println!("✓ {}", description);
                println!("  Expression: {}", expr);
                if !remaining.is_empty() {
                    println!("  Remaining: '{}'", remaining);
                }
                println!("  AST: {:?}\n", ast);
            }
            Err(e) => {
                println!("✗ {} - FAILED", description);
                println!("  Expression: {}", expr);
                println!("  Error: {:?}\n", e);
            }
        }
    }
}

use arrow_jmespath::parse_jmespath;

#[cfg(test)]
mod advanced_parser_tests {
    use super::*;
    
    #[test]
    fn test_parse_multi_select_hash_simple() {
        let result = parse_jmespath("{name: name, age: age}");
        assert!(result.is_ok());
        println!("{{name: name, age: age}} parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_multi_select_hash_renamed() {
        let result = parse_jmespath("{person: name, years: age}");
        assert!(result.is_ok());
        println!("{{person: name, years: age}} parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_multi_select_hash_complex() {
        let result = parse_jmespath("{user: name, info: age[0]}");
        assert!(result.is_ok());
        println!("{{user: name, info: age[0]}} parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_multi_select_empty() {
        let result = parse_jmespath("{}");
        assert!(result.is_ok());
        println!("{{}} parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_flatten() {
        let result = parse_jmespath("data[]");
        assert!(result.is_ok());
        println!("data[] parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_flatten_double() {
        let result = parse_jmespath("data[][]");
        assert!(result.is_ok());
        println!("data[][] parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_function_keys() {
        let result = parse_jmespath("keys(data)");
        assert!(result.is_ok());
        println!("keys(data) parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_function_values() {
        let result = parse_jmespath("values(data)");
        assert!(result.is_ok());
        println!("values(data) parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_function_type() {
        let result = parse_jmespath("type(name)");
        assert!(result.is_ok());
        println!("type(name) parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_type_with_index() {
        let result = parse_jmespath("type(age[0])");
        assert!(result.is_ok());
        println!("type(age[0]) parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_multi_select_with_filter() {
        let result = parse_jmespath("{names: name[?age > 25]}");
        assert!(result.is_ok());
        println!("{{names: name[?age > 25]}} parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_multi_select_with_function() {
        let result = parse_jmespath("{count: length(name), total: sum(age)}");
        assert!(result.is_ok());
        println!("{{count: length(name), total: sum(age)}} parsed: {:?}", result);
    }
}

fn main() {
    println!("Run with: cargo test --test advanced_parser_tests -- --nocapture");
}

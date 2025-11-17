use arrow_jmespath::parse_jmespath;

#[cfg(test)]
mod array_ops_parser_tests {
    use super::*;
    
    #[test]
    fn test_parse_index() {
        let result = parse_jmespath("age[0]");
        assert!(result.is_ok());
        println!("age[0] parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_negative_index() {
        let result = parse_jmespath("age[-1]");
        assert!(result.is_ok());
        println!("age[-1] parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_slice_simple() {
        let result = parse_jmespath("age[0:5]");
        assert!(result.is_ok());
        println!("age[0:5] parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_slice_start_only() {
        let result = parse_jmespath("age[2:]");
        assert!(result.is_ok());
        println!("age[2:] parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_slice_stop_only() {
        let result = parse_jmespath("age[:3]");
        assert!(result.is_ok());
        println!("age[:3] parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_slice_with_step() {
        let result = parse_jmespath("age[::2]");
        assert!(result.is_ok());
        println!("age[::2] parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_slice_full() {
        let result = parse_jmespath("age[1:10:2]");
        assert!(result.is_ok());
        println!("age[1:10:2] parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_projection_simple() {
        let result = parse_jmespath("users[*]");
        assert!(result.is_ok());
        println!("users[*] parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_projection_with_field() {
        let result = parse_jmespath("users[*].name");
        assert!(result.is_ok());
        println!("users[*].name parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_pipe_simple() {
        let result = parse_jmespath("users | length(users)");
        assert!(result.is_ok());
        println!("users | length(users) parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_pipe_with_filter() {
        let result = parse_jmespath("users[?age > 30] | name");
        assert!(result.is_ok());
        println!("users[?age > 30] | name parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_combined_index_and_slice() {
        let result = parse_jmespath("data[0:5][2]");
        assert!(result.is_ok());
        println!("data[0:5][2] parsed: {:?}", result);
    }
    
    #[test]
    fn test_parse_filter_with_index() {
        let result = parse_jmespath("users[?age > 25][0]");
        assert!(result.is_ok());
        println!("users[?age > 25][0] parsed: {:?}", result);
    }
}

fn main() {
    println!("Run with: cargo test --test array_ops_parser_tests -- --nocapture");
}

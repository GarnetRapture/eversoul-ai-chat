use llama_cpp_2::token::LlamaBatch; fn test() { let mut batch = LlamaBatch::new(10, 1); batch.clear(); }

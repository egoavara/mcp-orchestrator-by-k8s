use std::collections::HashMap;

#[allow(dead_code)]
pub trait FormValidation {
    fn validate(&self) -> HashMap<String, String>;

    fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }
}

pub fn validate_name(name: &str) -> Option<String> {
    if name.is_empty() {
        return Some("Name is required".to_string());
    }
    if name.len() > 63 {
        return Some("Name must be 63 characters or less".to_string());
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Some("Name can only contain alphanumeric and hyphens".to_string());
    }
    if let Some(first) = name.chars().next() {
        if !first.is_alphanumeric() {
            return Some("Name must start with alphanumeric".to_string());
        }
    }
    if let Some(last) = name.chars().last() {
        if !last.is_alphanumeric() {
            return Some("Name must end with alphanumeric".to_string());
        }
    }
    None
}

pub fn validate_docker_image(image: &str) -> Option<String> {
    if image.is_empty() {
        return Some("Docker image is required".to_string());
    }
    let parts: Vec<&str> = image.split(':').collect();
    if parts.len() > 2 {
        return Some("Invalid image format".to_string());
    }
    None
}

pub fn validate_cpu(cpu: &str) -> Option<String> {
    if cpu.is_empty() {
        return None;
    }

    if let Some(stripped) = cpu.strip_suffix('m') {
        if stripped.parse::<u32>().is_ok() {
            return None;
        }
    } else if cpu.parse::<f64>().is_ok() {
        return None;
    }

    Some("Invalid CPU format (e.g., '2', '500m')".to_string())
}

pub fn validate_memory(memory: &str) -> Option<String> {
    if memory.is_empty() {
        return None;
    }

    for suffix in &["Ki", "Mi", "Gi", "Ti"] {
        if memory.ends_with(suffix) && memory[..memory.len() - 2].parse::<u64>().is_ok() {
            return None;
        }
    }

    Some("Invalid memory format (e.g., '512Mi', '4Gi')".to_string())
}

pub fn validate_arg_env_key(key: &str) -> Option<String> {
    if key.is_empty() {
        return Some("Key is required".to_string());
    }
    
    // ^[a-z][a-z0-9-]*$ - 소문자로 시작해야 함
    let first_char = key.chars().next().unwrap();
    if !first_char.is_ascii_lowercase() {
        return Some("Key must start with a lowercase letter (a-z)".to_string());
    }
    
    // 나머지는 소문자, 숫자, 하이픈만 허용
    if !key.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Some("Key can only contain lowercase letters (a-z), digits (0-9), and hyphens (-)".to_string());
    }
    
    None
}

pub fn validate_arg_env_value(value: &str) -> Option<String> {
    if value.is_empty() {
        return Some("Value is required".to_string());
    }
    
    // ^((?<ENV_NAME>[A-Za-z0-9_-]+)\s*:\s*)?(?<TYPENAME>[a-z][a-z0-9-]*\??)$
    
    // {ENV_NAME}: {TYPE} 형태인지 확인
    if let Some((env_name, type_part)) = value.split_once(':') {
        let env_name = env_name.trim();
        let type_part = type_part.trim();
        
        // 환경변수명 검증: [A-Za-z0-9_-]+
        if env_name.is_empty() {
            return Some("Environment variable name is required before ':'".to_string());
        }
        
        if !env_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return Some("Environment variable name can only contain letters (A-Z, a-z), digits (0-9), underscores (_), and hyphens (-)".to_string());
        }
        
        // 타입 검증: ^[a-z][a-z0-9-]*\??$ (끝에 선택적으로 ? 허용)
        if type_part.is_empty() {
            return Some("Type is required after ':'".to_string());
        }
        
        let first_char = type_part.chars().next().unwrap();
        if !first_char.is_ascii_lowercase() {
            return Some("Type must start with a lowercase letter".to_string());
        }
        
        if !type_part.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '?') {
            return Some("Type can only contain lowercase letters, digits, hyphens, and optional '?' at the end".to_string());
        }
        
        // ? 는 끝에만 올 수 있고 최대 1개만 허용
        let question_count = type_part.chars().filter(|&c| c == '?').count();
        if question_count > 1 {
            return Some("Type can only have one '?' at the end".to_string());
        }
        if question_count == 1 && !type_part.ends_with('?') {
            return Some("'?' can only appear at the end of the type".to_string());
        }
        
        return None;
    }
    
    // {TYPE} 형태만 있는 경우: ^[a-z][a-z0-9-]*\??$ (끝에 선택적으로 ? 허용)
    let first_char = value.chars().next().unwrap();
    if !first_char.is_ascii_lowercase() {
        return Some("Type must start with a lowercase letter".to_string());
    }
    
    if !value.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '?') {
        return Some("Type can only contain lowercase letters, digits, hyphens, and optional '?' at the end".to_string());
    }
    
    // ? 는 끝에만 올 수 있고 최대 1개만 허용
    let question_count = value.chars().filter(|&c| c == '?').count();
    if question_count > 1 {
        return Some("Type can only have one '?' at the end".to_string());
    }
    if question_count == 1 && !value.ends_with('?') {
        return Some("'?' can only appear at the end of the type".to_string());
    }
    
    None
}

pub fn validate_arg_env_name(env_name: &str) -> Option<String> {
    if env_name.is_empty() {
        return None; // 선택적이므로 빈 값 허용
    }
    
    // [A-Za-z0-9_-]+
    if !env_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return Some("Environment variable name can only contain letters (A-Z, a-z), digits (0-9), underscores (_), and hyphens (-)".to_string());
    }
    
    None
}

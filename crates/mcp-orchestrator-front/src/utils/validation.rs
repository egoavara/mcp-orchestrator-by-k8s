use std::collections::HashMap;

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
    
    if cpu.ends_with('m') {
        if let Ok(_) = cpu[..cpu.len()-1].parse::<u32>() {
            return None;
        }
    } else if let Ok(_) = cpu.parse::<f64>() {
        return None;
    }
    
    Some("Invalid CPU format (e.g., '2', '500m')".to_string())
}

pub fn validate_memory(memory: &str) -> Option<String> {
    if memory.is_empty() {
        return None;
    }
    
    for suffix in &["Ki", "Mi", "Gi", "Ti"] {
        if memory.ends_with(suffix) {
            if let Ok(_) = memory[..memory.len()-2].parse::<u64>() {
                return None;
            }
        }
    }
    
    Some("Invalid memory format (e.g., '512Mi', '4Gi')".to_string())
}

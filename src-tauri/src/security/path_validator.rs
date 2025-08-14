use std::path::{Path, PathBuf};
use std::fs;

pub struct PathValidator;

impl PathValidator {
    /// 安全地验证和规范化文件路径，防止路径遍历攻击
    pub fn validate_path(path: &str, base_dir: &Path) -> Result<PathBuf, String> {
        // 1. 解析路径
        let input_path = Path::new(path);
        
        // 2. 检查是否包含危险的路径组件
        for component in input_path.components() {
            match component {
                std::path::Component::ParentDir => {
                    return Err("路径不能包含 '..' 组件".to_string());
                }
                std::path::Component::RootDir => {
                    return Err("不允许绝对路径".to_string());
                }
                _ => {}
            }
        }
        
        // 3. 构建完整路径
        let full_path = base_dir.join(input_path);
        
        // 4. 规范化路径
        let canonical_path = match fs::canonicalize(&full_path) {
            Ok(path) => path,
            Err(_) => {
                // 如果文件不存在，尝试规范化父目录
                if let Some(parent) = full_path.parent() {
                    if let Ok(parent_canonical) = fs::canonicalize(parent) {
                        parent_canonical.join(full_path.file_name().unwrap_or_default())
                    } else {
                        return Err("无效的路径".to_string());
                    }
                } else {
                    return Err("无效的路径".to_string());
                }
            }
        };
        
        // 5. 确保规范化后的路径仍在基础目录内
        let canonical_base = fs::canonicalize(base_dir)
            .map_err(|_| "基础目录不存在".to_string())?;
            
        if !canonical_path.starts_with(&canonical_base) {
            return Err("路径超出允许的范围".to_string());
        }
        
        Ok(canonical_path)
    }
    
    /// 验证文件扩展名
    pub fn validate_file_extension(path: &Path, allowed_extensions: &[&str]) -> Result<(), String> {
        if let Some(extension) = path.extension() {
            let ext_str = extension.to_str().unwrap_or("");
            if allowed_extensions.contains(&ext_str.to_lowercase().as_str()) {
                Ok(())
            } else {
                Err(format!("不允许的文件扩展名: {}", ext_str))
            }
        } else {
            Err("文件必须有扩展名".to_string())
        }
    }
    
    /// 验证文件大小
    pub fn validate_file_size(path: &Path, max_size_bytes: u64) -> Result<(), String> {
        match fs::metadata(path) {
            Ok(metadata) => {
                if metadata.len() > max_size_bytes {
                    Err(format!("文件大小超过限制: {} bytes", max_size_bytes))
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(format!("无法读取文件信息: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_path_traversal_prevention() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();
        
        // 测试路径遍历攻击
        assert!(PathValidator::validate_path("../etc/passwd", base_path).is_err());
        assert!(PathValidator::validate_path("..\\..\\windows\\system32", base_path).is_err());
        assert!(PathValidator::validate_path("/etc/passwd", base_path).is_err());
        
        // 测试合法路径
        assert!(PathValidator::validate_path("test.txt", base_path).is_ok());
        assert!(PathValidator::validate_path("subfolder/test.txt", base_path).is_ok());
    }
    
    #[test]
    fn test_file_extension_validation() {
        let path = Path::new("test.wav");
        assert!(PathValidator::validate_file_extension(path, &["wav", "mp3", "m4a"]).is_ok());
        
        let path = Path::new("test.exe");
        assert!(PathValidator::validate_file_extension(path, &["wav", "mp3", "m4a"]).is_err());
    }
}
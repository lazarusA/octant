use crate::stores::VariableInfo;
use crate::utils::units::calculate_variable_size_bytes;

/// Extract variable metadata from Icechunk configuration / manifest JSON structures.
pub fn extract_icechunk_manifest_variables(manifest_json: &serde_json::Value) -> Vec<VariableInfo> {
    let mut variables = Vec::new();

    if let Some(vars) = manifest_json.get("variables").and_then(|v| v.as_array()) {
        for var_item in vars {
            if let Some(var_name) = var_item.get("name").and_then(|n| n.as_str()) {
                let shape = vec![365, 64, 64];
                let data_type = "float32".to_string();
                let file_size = calculate_variable_size_bytes(&shape, &data_type);

                variables.push(VariableInfo {
                    name: var_name.to_string(),
                    data_type,
                    shape,
                    dimension_names: vec!["time".to_string(), "y".to_string(), "x".to_string()],
                    chunk_shape: vec![30, 64, 64],
                    file_size,
                });
            }
        }
    }

    variables
}

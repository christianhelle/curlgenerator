use crate::error::CurlGeneratorError;
use anyhow::Result;
use openapiv3::OpenAPI;
use sw4rm_rs::Spec;

pub fn load_document(path: &str) -> Result<OpenAPI> {
    let content = if is_http_url(path) {
        load_from_url(path)?
    } else {
        std::fs::read_to_string(path)?
    };

    parse_openapi(&content)
}

fn load_from_url(url: &str) -> Result<String> {
    let response = reqwest::blocking::get(url)?;
    let text = response.text()?;
    Ok(text)
}

fn parse_openapi(content: &str) -> Result<OpenAPI> {
    // Try parsing as OpenAPI v3.x directly first
    if let Ok(spec) = serde_json::from_str::<OpenAPI>(content) {
        return Ok(spec);
    }
    if let Ok(spec) = serde_yaml::from_str::<OpenAPI>(content) {
        return Ok(spec);
    }

    // Try to detect if this is a Swagger 2.0 spec
    let json_value: serde_json::Value = if let Ok(v) = serde_json::from_str(content) {
        v
    } else if let Ok(v) = serde_yaml::from_str(content) {
        v
    } else {
        return Err(CurlGeneratorError::OpenApiParseError(
            "Failed to parse as JSON or YAML".to_string(),
        )
        .into());
    };

    // Check if it's Swagger 2.0
    if let Some(swagger) = json_value.get("swagger").and_then(|v| v.as_str()) {
        if swagger.starts_with("2.") {
            // Try local conversion first for speed, fallback to API if it fails
            return match convert_swagger_v2_manual(&json_value) {
                Ok(spec) => Ok(spec),
                Err(_) => convert_swagger_v2_using_api(&json_value)
            };
        }
    }

    // Try parsing with sw4rm-rs as fallback
    let sw4rm_spec = if let Ok(spec) = serde_json::from_value::<Spec>(json_value.clone()) {
        Some(spec)
    } else {
        None
    };

    if let Some(spec) = sw4rm_spec {
        eprintln!("DEBUG: Parsed with sw4rm-rs, converting...");
        return convert_spec_to_openapi_v3(&spec);
    }

    eprintln!("DEBUG: Failed to parse with any method");
    Err(CurlGeneratorError::OpenApiParseError(
        "Failed to parse as OpenAPI v2.0 or v3.x (JSON or YAML)".to_string(),
    )
    .into())
}

fn convert_swagger_v2_to_v3(spec: &serde_json::Value) -> Result<OpenAPI> {
    // Convert Swagger v2.0 to OpenAPI v3 format
    let mut json_obj = if let serde_json::Value::Object(obj) = spec {
        obj.clone()
    } else {
        return Err(CurlGeneratorError::OpenApiParseError(
            "Invalid Swagger specification structure".to_string(),
        )
        .into());
    };

    // Remove swagger field and add openapi field
    json_obj.remove("swagger");
    json_obj.insert("openapi".to_string(), serde_json::json!("3.0.0"));

    // Convert host, basePath, schemes to servers
    let host = json_obj.remove("host");
    let base_path = json_obj.remove("basePath");
    let schemes = json_obj.remove("schemes");

    if host.is_some() || base_path.is_some() || schemes.is_some() {
        let mut server_url = String::new();

        // Build server URL from v2 fields
        if let Some(schemes_arr) = schemes {
            if let Some(scheme) = schemes_arr.as_array().and_then(|a| a.first()) {
                server_url.push_str(scheme.as_str().unwrap_or("http"));
            } else {
                server_url.push_str("http");
            }
        } else {
            server_url.push_str("http");
        }

        server_url.push_str("://");

        if let Some(host_val) = host {
            server_url.push_str(host_val.as_str().unwrap_or("localhost"));
        } else {
            server_url.push_str("localhost");
        }

        if let Some(base_path_val) = base_path {
            let path = base_path_val.as_str().unwrap_or("");
            if !path.is_empty() && !path.starts_with('/') {
                server_url.push('/');
            }
            server_url.push_str(path);
        }

        json_obj.insert(
            "servers".to_string(),
            serde_json::json!([{"url": server_url}]),
        );
    }

    // Convert paths parameters from v2 to v3
    if let Some(paths) = json_obj.get_mut("paths") {
        if let Some(paths_obj) = paths.as_object_mut() {
            for (_path, path_item) in paths_obj.iter_mut() {
                if let Some(path_obj) = path_item.as_object_mut() {
                    for (_method, operation) in path_obj.iter_mut() {
                        if let Some(op_obj) = operation.as_object_mut() {
                            convert_operation_parameters_v2_to_v3(op_obj);
                        }
                    }
                }
            }
        }
    }

    // Convert definitions to components/schemas
    if let Some(definitions) = json_obj.remove("definitions") {
        let mut components = json_obj
            .remove("components")
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default();
        components.insert("schemas".to_string(), definitions);
        json_obj.insert("components".to_string(), serde_json::Value::Object(components));
    }

    // Update all $ref paths from #/definitions/ to #/components/schemas/
    let mut value = serde_json::Value::Object(json_obj);
    value = update_refs(&mut value)?;
    json_obj = if let serde_json::Value::Object(obj) = value {
        obj
    } else {
        return Err(CurlGeneratorError::OpenApiParseError(
            "Failed to convert references".to_string(),
        )
        .into());
    };

    // Remove Swagger 2.0 specific fields
    json_obj.remove("securityDefinitions");
    json_obj.remove("consumes");
    json_obj.remove("produces");

    // Debug: write converted spec to file
    if std::env::var("DEBUG_OPENAPI").is_ok() {
        let debug_path = "converted_v3_debug.json";
        if let Ok(file) = std::fs::File::create(debug_path) {
            let _ = serde_json::to_writer_pretty(file, &json_obj);
            eprintln!("DEBUG: Wrote converted spec to {}", debug_path);
        }
    }

    // Try to deserialize to OpenAPI v3
    let openapi_spec: OpenAPI = serde_json::from_value(serde_json::Value::Object(json_obj))?;
    Ok(openapi_spec)
}

fn convert_operation_parameters_v2_to_v3(op_obj: &mut serde_json::Map<String, serde_json::Value>) {
    let mut request_body_to_add = None;

    if let Some(params) = op_obj.get_mut("parameters") {
        if let Some(params_arr) = params.as_array_mut() {
            let mut body_param = None;
            let mut form_params = Vec::new();
            let mut non_body_params = Vec::new();

            // Separate body and non-body parameters
            for param in params_arr.drain(..) {
                if let Some(param_obj) = param.as_object() {
                    if let Some(in_val) = param_obj.get("in").and_then(|v| v.as_str()) {
                        match in_val {
                            "body" => body_param = Some(param),
                            "formData" => form_params.push(param),
                            _ => {
                                let mut converted_param = param;
                                convert_parameter_v2_to_v3(&mut converted_param);
                                non_body_params.push(converted_param);
                            }
                        }
                    } else {
                        non_body_params.push(param);
                    }
                }
            }

            // Update parameters array with non-body parameters
            *params_arr = non_body_params;

            // Build requestBody for body parameter
            if let Some(body) = body_param {
                if let Some(body_obj) = body.as_object() {
                    let mut request_body = serde_json::Map::new();

                    if let Some(desc) = body_obj.get("description") {
                        request_body.insert("description".to_string(), desc.clone());
                    }
                    if let Some(required) = body_obj.get("required") {
                        request_body.insert("required".to_string(), required.clone());
                    }

                    if let Some(schema) = body_obj.get("schema") {
                        let mut content = serde_json::Map::new();
                        let mut media_type = serde_json::Map::new();
                        media_type.insert("schema".to_string(), schema.clone());
                        content.insert(
                            "application/json".to_string(),
                            serde_json::Value::Object(media_type),
                        );
                        request_body.insert("content".to_string(), serde_json::Value::Object(content));
                    }

                    request_body_to_add = Some(serde_json::Value::Object(request_body));
                }
            }

            // Build requestBody for formData parameters
            if !form_params.is_empty() {
                let mut request_body = serde_json::Map::new();
                let mut content = serde_json::Map::new();
                let mut media_type = serde_json::Map::new();
                let mut schema = serde_json::Map::new();
                let mut properties = serde_json::Map::new();
                let mut required = Vec::new();

                for form_param in form_params {
                    if let Some(param_obj) = form_param.as_object() {
                        if let Some(name) = param_obj.get("name").and_then(|v| v.as_str()) {
                            let mut prop = serde_json::Map::new();
                            if let Some(t) = param_obj.get("type") {
                                prop.insert("type".to_string(), t.clone());
                            }
                            if let Some(f) = param_obj.get("format") {
                                prop.insert("format".to_string(), f.clone());
                            }
                            if let Some(d) = param_obj.get("description") {
                                prop.insert("description".to_string(), d.clone());
                            }
                            properties.insert(name.to_string(), serde_json::Value::Object(prop));

                            if param_obj
                                .get("required")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false)
                            {
                                required.push(serde_json::Value::String(name.to_string()));
                            }
                        }
                    }
                }

                schema.insert("type".to_string(), serde_json::json!("object"));
                schema.insert("properties".to_string(), serde_json::Value::Object(properties));
                if !required.is_empty() {
                    schema.insert("required".to_string(), serde_json::Value::Array(required));
                }

                media_type.insert("schema".to_string(), serde_json::Value::Object(schema));
                content.insert(
                    "application/x-www-form-urlencoded".to_string(),
                    serde_json::Value::Object(media_type),
                );
                request_body.insert("content".to_string(), serde_json::Value::Object(content));

                request_body_to_add = Some(serde_json::Value::Object(request_body));
            }
        }
    }

    // Add requestBody to operation if we created one
    if let Some(request_body) = request_body_to_add {
        op_obj.insert("requestBody".to_string(), request_body);
    }

    // Remove empty parameters array if no parameters left
    if let Some(params) = op_obj.get("parameters") {
        if let Some(params_arr) = params.as_array() {
            if params_arr.is_empty() {
                op_obj.remove("parameters");
            }
        }
    }
}

fn convert_spec_to_openapi_v3(spec: &Spec) -> Result<OpenAPI> {
    // Serialize sw4rm Spec to JSON and then deserialize to openapiv3::OpenAPI
    // This works because sw4rm-rs already handles v2 to v3 conversion internally
    let json_value = serde_json::to_value(spec)?;
    
    // Convert Swagger v2.0 fields to OpenAPI v3 if necessary
    let json_obj = if let serde_json::Value::Object(mut obj) = json_value {
        // Convert swagger field to openapi
        let version = obj.remove("swagger").or_else(|| obj.remove("specVersion"));
        if let Some(swagger_version) = version {
            if swagger_version.as_str() == Some("2.0") {
                obj.insert("openapi".to_string(), serde_json::json!("3.0.0"));
                
                // Convert host, basePath, schemes to servers
                let host = obj.remove("host");
                let base_path = obj.remove("basePath");
                let schemes = obj.remove("schemes");
                
                if host.is_some() || base_path.is_some() || schemes.is_some() {
                    let mut server_url = String::new();
                    
                    // Build server URL from v2 fields
                    if let Some(schemes_arr) = schemes {
                        if let Some(scheme) = schemes_arr.as_array().and_then(|a| a.first()) {
                            server_url.push_str(scheme.as_str().unwrap_or("http"));
                        } else {
                            server_url.push_str("http");
                        }
                    } else {
                        server_url.push_str("http");
                    }
                    
                    server_url.push_str("://");
                    
                    if let Some(host_val) = host {
                        server_url.push_str(host_val.as_str().unwrap_or("localhost"));
                    } else {
                        server_url.push_str("localhost");
                    }
                    
                    if let Some(base_path_val) = base_path {
                        let path = base_path_val.as_str().unwrap_or("");
                        if !path.is_empty() && !path.starts_with('/') {
                            server_url.push('/');
                        }
                        server_url.push_str(path);
                    }
                    
                    obj.insert("servers".to_string(), serde_json::json!([{"url": server_url}]));
                }
                
                // Convert paths parameters from v2 to v3
                if let Some(paths) = obj.get_mut("paths") {
                    if let Some(paths_obj) = paths.as_object_mut() {
                        for (_path, path_item) in paths_obj.iter_mut() {
                            if let Some(path_obj) = path_item.as_object_mut() {
                                for (_method, operation) in path_obj.iter_mut() {
                                    if let Some(op_obj) = operation.as_object_mut() {
                                        let mut request_body_to_add = None;
                                        
                                        if let Some(params) = op_obj.get_mut("parameters") {
                                            if let Some(params_arr) = params.as_array_mut() {
                                                let mut body_param = None;
                                                let mut form_params = Vec::new();
                                                let mut non_body_params = Vec::new();
                                                
                                                // Separate body and non-body parameters
                                                for param in params_arr.drain(..) {
                                                    if let Some(param_obj) = param.as_object() {
                                                        if let Some(in_val) = param_obj.get("in").and_then(|v| v.as_str()) {
                                                            match in_val {
                                                                "body" => body_param = Some(param),
                                                                "formData" => form_params.push(param),
                                                                _ => {
                                                                    let mut converted_param = param;
                                                                    convert_parameter_v2_to_v3(&mut converted_param);
                                                                    non_body_params.push(converted_param);
                                                                }
                                                            }
                                                        } else {
                                                            non_body_params.push(param);
                                                        }
                                                    }
                                                }
                                                
                                                // Update parameters array with non-body parameters
                                                *params_arr = non_body_params;
                                                
                                                // Build requestBody for body parameter
                                                if let Some(body) = body_param {
                                                    if let Some(body_obj) = body.as_object() {
                                                        let mut request_body = serde_json::Map::new();
                                                        
                                                        if let Some(desc) = body_obj.get("description") {
                                                            request_body.insert("description".to_string(), desc.clone());
                                                        }
                                                        if let Some(required) = body_obj.get("required") {
                                                            request_body.insert("required".to_string(), required.clone());
                                                        }
                                                        
                                                        if let Some(schema) = body_obj.get("schema") {
                                                            let mut content = serde_json::Map::new();
                                                            let mut media_type = serde_json::Map::new();
                                                            media_type.insert("schema".to_string(), schema.clone());
                                                            content.insert("application/json".to_string(), serde_json::Value::Object(media_type));
                                                            request_body.insert("content".to_string(), serde_json::Value::Object(content));
                                                        }
                                                        
                                                        request_body_to_add = Some(serde_json::Value::Object(request_body));
                                                    }
                                                }
                                                
                                                // Build requestBody for formData parameters
                                                if !form_params.is_empty() {
                                                    let mut request_body = serde_json::Map::new();
                                                    let mut content = serde_json::Map::new();
                                                    let mut media_type = serde_json::Map::new();
                                                    let mut schema = serde_json::Map::new();
                                                    let mut properties = serde_json::Map::new();
                                                    let mut required = Vec::new();
                                                    
                                                    for form_param in form_params {
                                                        if let Some(param_obj) = form_param.as_object() {
                                                            if let Some(name) = param_obj.get("name").and_then(|v| v.as_str()) {
                                                                let mut prop = serde_json::Map::new();
                                                                if let Some(t) = param_obj.get("type") {
                                                                    prop.insert("type".to_string(), t.clone());
                                                                }
                                                                if let Some(f) = param_obj.get("format") {
                                                                    prop.insert("format".to_string(), f.clone());
                                                                }
                                                                if let Some(d) = param_obj.get("description") {
                                                                    prop.insert("description".to_string(), d.clone());
                                                                }
                                                                properties.insert(name.to_string(), serde_json::Value::Object(prop));
                                                                
                                                                if param_obj.get("required").and_then(|v| v.as_bool()).unwrap_or(false) {
                                                                    required.push(serde_json::Value::String(name.to_string()));
                                                                }
                                                            }
                                                        }
                                                    }
                                                    
                                                    schema.insert("type".to_string(), serde_json::json!("object"));
                                                    schema.insert("properties".to_string(), serde_json::Value::Object(properties));
                                                    if !required.is_empty() {
                                                        schema.insert("required".to_string(), serde_json::Value::Array(required));
                                                    }
                                                    
                                                    media_type.insert("schema".to_string(), serde_json::Value::Object(schema));
                                                    content.insert("application/x-www-form-urlencoded".to_string(), serde_json::Value::Object(media_type));
                                                    request_body.insert("content".to_string(), serde_json::Value::Object(content));
                                                    
                                                    request_body_to_add = Some(serde_json::Value::Object(request_body));
                                                }
                                                
                                                // Remove parameters array if empty
                                                if params_arr.is_empty() {
                                                    op_obj.remove("parameters");
                                                }
                                            }
                                        }
                                        
                                        // Add requestBody after we're done with parameters
                                        if let Some(request_body) = request_body_to_add {
                                            op_obj.insert("requestBody".to_string(), request_body);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Convert definitions to components/schemas
                if let Some(definitions) = obj.remove("definitions") {
                    let components = obj.entry("components".to_string())
                        .or_insert_with(|| serde_json::json!({}));
                    if let Some(comp_obj) = components.as_object_mut() {
                        comp_obj.insert("schemas".to_string(), definitions);
                    }
                }
                
                // Convert parameters to components/parameters
                if let Some(parameters) = obj.remove("parameters") {
                    let components = obj.entry("components".to_string())
                        .or_insert_with(|| serde_json::json!({}));
                    if let Some(comp_obj) = components.as_object_mut() {
                        comp_obj.insert("parameters".to_string(), parameters);
                    }
                }
                
                // Convert responses to components/responses
                if let Some(responses) = obj.remove("responses") {
                    let components = obj.entry("components".to_string())
                        .or_insert_with(|| serde_json::json!({}));
                    if let Some(comp_obj) = components.as_object_mut() {
                        comp_obj.insert("responses".to_string(), responses);
                    }
                }
                
                // Convert securityDefinitions to components/securitySchemes
                if let Some(security_defs) = obj.remove("securityDefinitions") {
                    let components = obj.entry("components".to_string())
                        .or_insert_with(|| serde_json::json!({}));
                    if let Some(comp_obj) = components.as_object_mut() {
                        comp_obj.insert("securitySchemes".to_string(), security_defs);
                    }
                }
                
                // Remove v2-specific fields
                obj.remove("consumes");
                obj.remove("produces");
            }
        }
        
        serde_json::Value::Object(obj)
    } else {
        json_value
    };
    
    // Parse as OpenAPI v3
    let openapi: OpenAPI = serde_json::from_value(json_obj)?;
    Ok(openapi)
}

fn convert_parameter_v2_to_v3(param: &mut serde_json::Value) {
    if let Some(param_obj) = param.as_object_mut() {
        // Skip if already has schema or content (already v3 format)
        if param_obj.contains_key("schema") || param_obj.contains_key("content") {
            return;
        }

        // Check if this is a v2-style parameter (has type but not schema)
        if param_obj.contains_key("type") {
            let param_type = param_obj.remove("type");
            let format = param_obj.remove("format");
            let items = param_obj.remove("items");
            let _collection_format = param_obj.remove("collectionFormat");
            let default = param_obj.remove("default");
            let enum_vals = param_obj.remove("enum");
            let min = param_obj.remove("minimum");
            let max = param_obj.remove("maximum");
            let min_length = param_obj.remove("minLength");
            let max_length = param_obj.remove("maxLength");
            let pattern = param_obj.remove("pattern");
            
            // Build schema object
            let mut schema = serde_json::Map::new();
            
            if let Some(t) = param_type {
                schema.insert("type".to_string(), t);
            }
            if let Some(f) = format {
                schema.insert("format".to_string(), f);
            }
            if let Some(i) = items {
                schema.insert("items".to_string(), i);
            }
            if let Some(d) = default {
                schema.insert("default".to_string(), d);
            }
            if let Some(e) = enum_vals {
                schema.insert("enum".to_string(), e);
            }
            if let Some(m) = min {
                schema.insert("minimum".to_string(), m);
            }
            if let Some(m) = max {
                schema.insert("maximum".to_string(), m);
            }
            if let Some(m) = min_length {
                schema.insert("minLength".to_string(), m);
            }
            if let Some(m) = max_length {
                schema.insert("maxLength".to_string(), m);
            }
            if let Some(p) = pattern {
                schema.insert("pattern".to_string(), p);
            }
            
            // Only add schema if it's not empty
            if !schema.is_empty() {
                param_obj.insert("schema".to_string(), serde_json::Value::Object(schema));
            } else {
                // Default schema for parameters without type
                param_obj.insert("schema".to_string(), serde_json::json!({"type": "string"}));
            }
            
            // Remove v2-specific fields
            param_obj.remove("collectionFormat");
        } else {
            // Parameter without type - add default schema
            param_obj.insert("schema".to_string(), serde_json::json!({"type": "string"}));
        }
    }
}

fn update_refs(value: &mut serde_json::Value) -> Result<serde_json::Value> {
    match value {
        serde_json::Value::Object(obj) => {
            let mut new_obj = serde_json::Map::new();
            for (key, val) in obj.iter_mut() {
                if key == "$ref" {
                    if let Some(ref_str) = val.as_str() {
                        if ref_str.starts_with("#/definitions/") {
                            let new_ref = ref_str.replace("#/definitions/", "#/components/schemas/");
                            new_obj.insert(key.clone(), serde_json::Value::String(new_ref));
                            continue;
                        }
                    }
                }
                new_obj.insert(key.clone(), update_refs(val)?);
            }
            Ok(serde_json::Value::Object(new_obj))
        }
        serde_json::Value::Array(arr) => {
            let mut new_arr = Vec::new();
            for item in arr.iter_mut() {
                new_arr.push(update_refs(item)?);
            }
            Ok(serde_json::Value::Array(new_arr))
        }
        _ => Ok(value.clone()),
    }
}

fn convert_swagger_v2_manual(spec: &serde_json::Value) -> Result<OpenAPI> {
    // Convert Swagger v2.0 to OpenAPI v3 format manually
    let mut json_obj = if let serde_json::Value::Object(obj) = spec {
        obj.clone()
    } else {
        return Err(CurlGeneratorError::OpenApiParseError(
            "Invalid Swagger specification structure".to_string(),
        )
        .into());
    };

    // Remove swagger field and add openapi field
    json_obj.remove("swagger");
    json_obj.insert("openapi".to_string(), serde_json::json!("3.0.0"));

    // Convert host, basePath, schemes to servers
    let host = json_obj.remove("host");
    let base_path = json_obj.remove("basePath");
    let schemes = json_obj.remove("schemes");

    if host.is_some() || base_path.is_some() || schemes.is_some() {
        let mut server_url = String::new();

        // Build server URL from v2 fields
        if let Some(schemes_arr) = schemes {
            if let Some(scheme) = schemes_arr.as_array().and_then(|a| a.first()) {
                server_url.push_str(scheme.as_str().unwrap_or("http"));
            } else {
                server_url.push_str("http");
            }
        } else {
            server_url.push_str("http");
        }

        server_url.push_str("://");

        if let Some(host_val) = host {
            server_url.push_str(host_val.as_str().unwrap_or("localhost"));
        } else {
            server_url.push_str("localhost");
        }

        if let Some(base_path_val) = base_path {
            let path = base_path_val.as_str().unwrap_or("");
            if !path.is_empty() && !path.starts_with('/') {
                server_url.push('/');
            }
            server_url.push_str(path);
        }

        json_obj.insert(
            "servers".to_string(),
            serde_json::json!([{"url": server_url}]),
        );
    }

    // Convert paths parameters from v2 to v3
    if let Some(paths) = json_obj.get_mut("paths") {
        if let Some(paths_obj) = paths.as_object_mut() {
            for (_path, path_item) in paths_obj.iter_mut() {
                if let Some(path_obj) = path_item.as_object_mut() {
                    // Convert path-level parameters first
                    if let Some(path_params) = path_obj.get_mut("parameters") {
                        if let Some(params_arr) = path_params.as_array_mut() {
                            for param in params_arr.iter_mut() {
                                convert_parameter_v2_to_v3(param);
                            }
                        }
                    }
                    
                    // Convert operation-level parameters
                    for (_method, operation) in path_obj.iter_mut() {
                        if let Some(op_obj) = operation.as_object_mut() {
                            convert_operation_parameters_v2_to_v3(op_obj);
                        }
                    }
                }
            }
        }
    }

    // Convert definitions to components/schemas
    if let Some(definitions) = json_obj.remove("definitions") {
        let mut components = json_obj
            .remove("components")
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default();
        components.insert("schemas".to_string(), definitions);
        json_obj.insert("components".to_string(), serde_json::Value::Object(components));
    }

    // Update all $ref paths from #/definitions/ to #/components/schemas/
    let mut value = serde_json::Value::Object(json_obj);
    value = update_refs(&mut value)?;
    json_obj = if let serde_json::Value::Object(obj) = value {
        obj
    } else {
        return Err(CurlGeneratorError::OpenApiParseError(
            "Failed to convert references".to_string(),
        )
        .into());
    };

    // Remove Swagger 2.0 specific fields
    json_obj.remove("securityDefinitions");
    json_obj.remove("consumes");
    json_obj.remove("produces");

    // Try to deserialize to OpenAPI v3
    let openapi_spec: OpenAPI = serde_json::from_value(serde_json::Value::Object(json_obj))?;
    Ok(openapi_spec)
}

fn convert_swagger_v2_locally(content: &str) -> Result<OpenAPI> {
    // Parse using openapi crate (Swagger 2.0)
    let v2_spec: openapi::Spec = if content.trim().starts_with('{') {
        serde_json::from_str(content).map_err(|e| {
            CurlGeneratorError::OpenApiParseError(
                format!("Failed to parse Swagger 2.0 JSON: {}", e)
            )
        })?
    } else {
        serde_yaml::from_str(content).map_err(|e| {
            CurlGeneratorError::OpenApiParseError(
                format!("Failed to parse Swagger 2.0 YAML: {}", e)
            )
        })?
    };

    // Convert to OpenAPI v3
    convert_v2_spec_to_v3(&v2_spec)
}

fn convert_v2_spec_to_v3(v2: &openapi::Spec) -> Result<OpenAPI> {
    // Build server URL from v2 fields
    let mut server_url = String::new();
    
    if let Some(ref schemes) = v2.schemes {
        if let Some(scheme) = schemes.first() {
            server_url.push_str(scheme);
        } else {
            server_url.push_str("http");
        }
    } else {
        server_url.push_str("http");
    }
    
    server_url.push_str("://");
    
    if let Some(ref host) = v2.host {
        server_url.push_str(host);
    } else {
        server_url.push_str("localhost");
    }
    
    if let Some(ref base_path) = v2.base_path {
        if !base_path.is_empty() && !base_path.starts_with('/') {
            server_url.push('/');
        }
        server_url.push_str(base_path);
    }

    // Create OpenAPI v3 structure
    let mut v3_json = serde_json::json!({
        "openapi": "3.0.0",
        "info": {
            "title": v2.info.title,
            "version": v2.info.version,
        },
        "servers": [{"url": server_url}],
        "paths": {},
        "components": {
            "schemas": {}
        }
    });

    // Add terms of service if present
    if let Some(ref tos) = v2.info.terms_of_service {
        v3_json["info"]["termsOfService"] = serde_json::json!(tos);
    }

    // Convert definitions to components/schemas
    for (name, schema) in &v2.definitions {
        v3_json["components"]["schemas"][name] = convert_v2_schema_to_v3(schema)?;
    }

    // Convert paths
    for (path, operations) in &v2.paths {
        let mut path_item = serde_json::Map::new();
        
        if let Some(ref op) = operations.get {
            path_item.insert("get".to_string(), convert_v2_operation_to_v3(op)?);
        }
        if let Some(ref op) = operations.post {
            path_item.insert("post".to_string(), convert_v2_operation_to_v3(op)?);
        }
        if let Some(ref op) = operations.put {
            path_item.insert("put".to_string(), convert_v2_operation_to_v3(op)?);
        }
        if let Some(ref op) = operations.patch {
            path_item.insert("patch".to_string(), convert_v2_operation_to_v3(op)?);
        }
        if let Some(ref op) = operations.delete {
            path_item.insert("delete".to_string(), convert_v2_operation_to_v3(op)?);
        }
        
        v3_json["paths"][path] = serde_json::Value::Object(path_item);
    }

    // Parse as OpenAPI v3
    let openapi_spec: OpenAPI = serde_json::from_value(v3_json)?;
    Ok(openapi_spec)
}

fn convert_v2_schema_to_v3(schema: &openapi::Schema) -> Result<serde_json::Value> {
    let mut v3_schema = serde_json::Map::new();
    
    if let Some(ref ref_path) = schema.ref_path {
        // Convert $ref from #/definitions/ to #/components/schemas/
        let new_ref = ref_path.replace("#/definitions/", "#/components/schemas/");
        v3_schema.insert("$ref".to_string(), serde_json::json!(new_ref));
    } else {
        if let Some(ref schema_type) = schema.schema_type {
            v3_schema.insert("type".to_string(), serde_json::json!(schema_type));
        }
        if let Some(ref format) = schema.format {
            v3_schema.insert("format".to_string(), serde_json::json!(format));
        }
        if let Some(ref description) = schema.description {
            v3_schema.insert("description".to_string(), serde_json::json!(description));
        }
        if let Some(ref enum_values) = schema.enum_values {
            v3_schema.insert("enum".to_string(), serde_json::json!(enum_values));
        }
    }
    
    Ok(serde_json::Value::Object(v3_schema))
}

fn convert_v2_operation_to_v3(op: &openapi::Operation) -> Result<serde_json::Value> {
    let mut v3_op = serde_json::Map::new();
    
    if let Some(ref summary) = op.summary {
        v3_op.insert("summary".to_string(), serde_json::json!(summary));
    }
    if let Some(ref description) = op.description {
        v3_op.insert("description".to_string(), serde_json::json!(description));
    }
    if let Some(ref operation_id) = op.operation_id {
        v3_op.insert("operationId".to_string(), serde_json::json!(operation_id));
    }
    if let Some(ref tags) = op.tags {
        v3_op.insert("tags".to_string(), serde_json::json!(tags));
    }
    
    // Convert parameters
    if let Some(ref params) = op.parameters {
        let mut v3_params = Vec::new();
        let mut body_param = None;
        
        for param in params {
            if param.location == "body" {
                body_param = Some(param);
            } else {
                v3_params.push(convert_v2_parameter_to_v3(param)?);
            }
        }
        
        if !v3_params.is_empty() {
            v3_op.insert("parameters".to_string(), serde_json::json!(v3_params));
        }
        
        // Convert body parameter to requestBody
        if let Some(body) = body_param {
            let mut request_body = serde_json::Map::new();
            if let Some(ref schema) = body.schema {
                let mut content = serde_json::Map::new();
                let mut media_type = serde_json::Map::new();
                media_type.insert("schema".to_string(), convert_v2_schema_to_v3(schema)?);
                content.insert("application/json".to_string(), serde_json::Value::Object(media_type));
                request_body.insert("content".to_string(), serde_json::Value::Object(content));
            }
            if body.required.unwrap_or(false) {
                request_body.insert("required".to_string(), serde_json::json!(true));
            }
            v3_op.insert("requestBody".to_string(), serde_json::Value::Object(request_body));
        }
    }
    
    // Convert responses
    let mut v3_responses = serde_json::Map::new();
    for (code, response) in &op.responses {
        let mut v3_response = serde_json::Map::new();
        v3_response.insert("description".to_string(), serde_json::json!(response.description));
        
        if let Some(ref schema) = response.schema {
            let mut content = serde_json::Map::new();
            let mut media_type = serde_json::Map::new();
            media_type.insert("schema".to_string(), convert_v2_schema_to_v3(schema)?);
            content.insert("application/json".to_string(), serde_json::Value::Object(media_type));
            v3_response.insert("content".to_string(), serde_json::Value::Object(content));
        }
        
        v3_responses.insert(code.clone(), serde_json::Value::Object(v3_response));
    }
    v3_op.insert("responses".to_string(), serde_json::Value::Object(v3_responses));
    
    Ok(serde_json::Value::Object(v3_op))
}

fn convert_v2_parameter_to_v3(param: &openapi::Parameter) -> Result<serde_json::Value> {
    let mut v3_param = serde_json::Map::new();
    
    v3_param.insert("name".to_string(), serde_json::json!(param.name));
    v3_param.insert("in".to_string(), serde_json::json!(param.location));
    
    if param.required.unwrap_or(false) {
        v3_param.insert("required".to_string(), serde_json::json!(true));
    }
    
    // Build schema
    let mut schema = serde_json::Map::new();
    if let Some(ref param_type) = param.param_type {
        schema.insert("type".to_string(), serde_json::json!(param_type));
    }
    if let Some(ref format) = param.format {
        schema.insert("format".to_string(), serde_json::json!(format));
    }
    
    if !schema.is_empty() {
        v3_param.insert("schema".to_string(), serde_json::Value::Object(schema));
    }
    
    Ok(serde_json::Value::Object(v3_param))
}

fn convert_swagger_v2_using_api(spec: &serde_json::Value) -> Result<OpenAPI> {
    // Use the Swagger Converter API
    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://converter.swagger.io/api/convert")
        .json(spec)
        .send()?;

    if !response.status().is_success() {
        return Err(CurlGeneratorError::OpenApiParseError(
            format!("Converter API failed with status: {}", response.status()),
        )
        .into());
    }

    let converted_text = response.text()?;
    
    // Try to parse as OpenAPI v3
    if let Ok(openapi) = serde_json::from_str::<OpenAPI>(&converted_text) {
        return Ok(openapi);
    }
    if let Ok(openapi) = serde_yaml::from_str::<OpenAPI>(&converted_text) {
        return Ok(openapi);
    }

    Err(CurlGeneratorError::OpenApiParseError(
        "Failed to parse converted OpenAPI v3 specification".to_string(),
    )
    .into())
}

fn is_http_url(path: &str) -> bool {
    path.starts_with("http://") || path.starts_with("https://")
}

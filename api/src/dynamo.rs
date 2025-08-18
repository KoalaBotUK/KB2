use std::collections::HashMap;
use aws_sdk_dynamodb::types::AttributeValue;

pub fn as_string(val: Option<&AttributeValue>, default: &String) -> String {
    if let Some(v) = val {
        if let Ok(s) = v.as_s() {
            return s.to_owned();
        }
    }
    default.to_owned()
}

pub fn _as_i32(val: Option<&AttributeValue>, default: i32) -> i32 {
    if let Some(v) = val {
        if let Ok(n) = v.as_n() {
            if let Ok(n) = n.parse::<i32>() {
                return n;
            }
        }
    }
    default
}

pub fn as_u64(val: Option<&AttributeValue>, default: u64) -> u64 {
    if let Some(v) = val {
        if let Ok(n) = v.as_n() {
            if let Ok(n) = n.parse::<u64>() {
                return n;
            }
        }
    }
    default
}

pub fn as_string_vec(val: Option<&AttributeValue>) -> Vec<String> {
    if let Some(val) = val {
        if let Ok(val) = val.as_l() {
            return val
                .iter()
                .map(|v| as_string(Some(v), &"".to_string()))
                .collect();
        }
    }
    // val
    //         .map(|v| v.as_l())
    //         .unwrap_or_else(|| Ok(&Vec::<AttributeValue>::new()))
    //         .unwrap_or_else(|_| &Vec::<AttributeValue>::new())
    //         .iter()
    //         .map(|v| as_string(Some(v), &"".to_string()))
    //         .collect();
    vec![]
}

pub fn as_map_vec(val: Option<&AttributeValue>) -> Vec<&HashMap<String, AttributeValue>> {
    const MAX_LIST_SIZE: usize = 1000; // Prevent stack overflow from huge lists
    if let Some(val) = val {
        if let Ok(val) = val.as_l() {
            if val.len() > MAX_LIST_SIZE {
                lambda_http::tracing::error!("DynamoDB list too large: {} items", val.len());
                return vec![];
            }
            return val
                .iter()
                .filter_map(|v| v.as_m().ok())
                .collect();
        }
    }
    vec![]
}


pub fn as_bool(val: Option<&AttributeValue>, default: bool) -> bool {
    if let Some(v) = val {
        if let Ok(b) = v.as_bool() {
            return *b;
        }
    }
    default
}

pub fn as_map(val: Option<&AttributeValue>) -> Option<&HashMap<String, AttributeValue>> {
    if let Some(val) = val {
        if let Ok(val) = val.as_m() {
            return Some(val);
        }
    }
    None
}
//! Output formatting

use serde_json::{json, Value};
use std::collections::HashMap;

/// Output builder for formatted CLI output
pub struct Output {
    json_mode: bool,
    fields: HashMap<String, Value>,
    message: Option<String>,
}

impl Output {
    /// Create a new output builder
    pub fn new(json_mode: bool) -> Self {
        Self {
            json_mode,
            fields: HashMap::new(),
            message: None,
        }
    }

    /// Add a string field to the output
    pub fn field(mut self, key: &str, value: &str) -> Self {
        self.fields.insert(key.to_string(), Value::String(value.to_string()));
        self
    }

    /// Add a u64 field to the output
    pub fn field_u64(mut self, key: &str, value: u64) -> Self {
        self.fields.insert(key.to_string(), Value::Number(value.into()));
        self
    }

    /// Add a u128 field to the output (stored as string to avoid overflow)
    pub fn field_u128(mut self, key: &str, value: u128) -> Self {
        self.fields.insert(key.to_string(), Value::String(value.to_string()));
        self
    }

    /// Add a JSON value field to the output
    pub fn field_value(mut self, key: &str, value: Value) -> Self {
        self.fields.insert(key.to_string(), value);
        self
    }

    /// Set the human-readable message
    pub fn message(mut self, msg: &str) -> Self {
        self.message = Some(msg.to_string());
        self
    }

    /// Print the output
    pub fn print(self) {
        if self.json_mode {
            let json = json!(self.fields);
            println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
        } else if let Some(msg) = self.message {
            println!("{}", msg);
        }
    }
}

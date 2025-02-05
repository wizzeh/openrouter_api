/*
   src/models/tool.rs

   This module defines the data types required for tool calling functionality.
   It includes types for function descriptions, tools representing callable functions,
   choices for tool selection, and the tool call details returned in API responses.
*/

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents a description for a callable function (tool).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDescription {
    /// The name of the function.
    pub name: String,
    /// An optional description of what the function does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// A JSON Schema object representing the function parameters.
    /// This should be a valid JSON object describing the expected arguments.
    pub parameters: Value,
}

/// Encapsulates a tool that the model can call.
/// Currently, we only support function‑type tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Tool {
    /// A function call tool with a detailed function description.
    Function {
        #[serde(rename = "function")]
        function: FunctionDescription,
    },
}

/// Represents the specific function call requested by the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// The name of the function to call.
    pub name: String,
    /// A JSON string representing the arguments for the function call.
    /// This string should be parseable into a structured object.
    pub arguments: String,
}

/// Represents the tool call details in an API response.
/// This is used when the model indicates that a tool should be invoked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// A unique identifier for the tool call.
    pub id: String,
    /// The type of call. Currently, this must be "function".
    #[serde(rename = "type")]
    pub kind: String,
    /// The details of the function call, including its name and arguments.
    #[serde(rename = "function")]
    pub function_call: FunctionCall,
}

/// Represents a tool selection option for when the model must choose among available tools.
/// This can be either no tool selected ("none"), an automatic selection ("auto"),
/// or a specific function choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// No tool is selected.
    None(String),
    /// The model automatically selects a tool.
    Auto(String),
    /// A specific function is selected. The `type` field is fixed to "function".
    FunctionChoice {
        #[serde(rename = "type")]
        kind: String,
        function: FunctionName,
    },
}

/// A simple struct to represent a function name for tool selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionName {
    /// The name of the function.
    pub name: String,
}

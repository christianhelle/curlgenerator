//! The version independent document model the script generator renders from.
//!
//! Swagger 2.0, OpenAPI 3.0, and OpenAPI 3.1 documents are all normalized into these types so the
//! generator only has to understand a single shape.

use serde_json::Value;

/// A normalized OpenAPI document.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Document {
    /// Server URLs declared by the document, in the order they appear.
    pub servers: Vec<String>,
    /// Path items in document order.
    pub paths: Vec<PathItem>,
}

/// A single path and the operations declared on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PathItem {
    /// The templated path, for example `/pet/{petId}`.
    pub path: String,
    /// Operations in document order.
    pub operations: Vec<Operation>,
}

/// A single operation on a path.
#[derive(Debug, Clone, PartialEq)]
pub struct Operation {
    /// The upper case HTTP method, for example `GET`.
    pub method: String,
    /// The operation identifier, when the document declares one.
    pub operation_id: Option<String>,
    /// The short operation summary.
    pub summary: Option<String>,
    /// The long operation description.
    pub description: Option<String>,
    /// Operation level parameters. `None` when the operation declares no `parameters` member,
    /// which the generator renders differently from an empty list.
    pub parameters: Option<Vec<Parameter>>,
    /// The request body, when the operation accepts one.
    pub request_body: Option<RequestBody>,
}

/// Where a parameter is transmitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterLocation {
    /// A templated path segment.
    Path,
    /// A query string entry.
    Query,
    /// A request header.
    Header,
    /// A cookie value.
    Cookie,
}

/// A single operation parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    /// The parameter name as declared by the document.
    pub name: String,
    /// Where the parameter is transmitted.
    pub location: ParameterLocation,
    /// The parameter description, when the document declares one.
    pub description: Option<String>,
}

/// A request body and the media types it accepts.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RequestBody {
    /// Media types in document order.
    pub content: Vec<MediaType>,
}

impl RequestBody {
    /// Returns the schema declared for `content_type`, when present.
    pub fn schema_for(&self, content_type: &str) -> Option<&Schema> {
        self.content
            .iter()
            .find(|media_type| media_type.content_type == content_type)
            .and_then(|media_type| media_type.schema.as_ref())
    }

    /// Returns the first declared content type.
    pub fn first_content_type(&self) -> Option<&str> {
        self.content
            .first()
            .map(|media_type| media_type.content_type.as_str())
    }
}

/// A single media type entry of a request body.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaType {
    /// The media type, for example `application/json`.
    pub content_type: String,
    /// The schema describing the payload.
    pub schema: Option<Schema>,
}

/// The JSON type a schema describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaType {
    /// A JSON object.
    Object,
    /// A JSON array.
    Array,
    /// A JSON string.
    String,
    /// A JSON integer.
    Integer,
    /// A JSON number.
    Number,
    /// A JSON boolean.
    Boolean,
}

impl SchemaType {
    /// Parses a schema type name as declared by a specification.
    ///
    /// # Examples
    ///
    /// ```
    /// use curlgenerator::normalized::SchemaType;
    ///
    /// assert_eq!(SchemaType::parse("integer"), Some(SchemaType::Integer));
    /// assert_eq!(SchemaType::parse("null"), None);
    /// ```
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "object" => Some(Self::Object),
            "array" => Some(Self::Array),
            "string" => Some(Self::String),
            "integer" => Some(Self::Integer),
            "number" => Some(Self::Number),
            "boolean" => Some(Self::Boolean),
            _ => None,
        }
    }
}

/// A normalized schema, with references already resolved.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Schema {
    /// The declared type, when the document declares one.
    pub schema_type: Option<SchemaType>,
    /// The declared string format, for example `date-time`.
    pub format: Option<String>,
    /// The declared example value.
    pub example: Option<Value>,
    /// Object properties in document order.
    pub properties: Vec<(String, Schema)>,
    /// The item schema of an array.
    pub items: Option<Box<Schema>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_type_parses_known_names() {
        assert_eq!(SchemaType::parse("object"), Some(SchemaType::Object));
        assert_eq!(SchemaType::parse("array"), Some(SchemaType::Array));
        assert_eq!(SchemaType::parse("string"), Some(SchemaType::String));
        assert_eq!(SchemaType::parse("integer"), Some(SchemaType::Integer));
        assert_eq!(SchemaType::parse("number"), Some(SchemaType::Number));
        assert_eq!(SchemaType::parse("boolean"), Some(SchemaType::Boolean));
        assert_eq!(SchemaType::parse("anything-else"), None);
    }

    #[test]
    fn request_body_looks_up_schemas_by_content_type() {
        let body = RequestBody {
            content: vec![
                MediaType {
                    content_type: "application/json".to_string(),
                    schema: Some(Schema {
                        schema_type: Some(SchemaType::Object),
                        ..Schema::default()
                    }),
                },
                MediaType {
                    content_type: "application/xml".to_string(),
                    schema: None,
                },
            ],
        };

        assert_eq!(body.first_content_type(), Some("application/json"));
        assert!(body.schema_for("application/json").is_some());
        assert!(body.schema_for("application/xml").is_none());
        assert!(body.schema_for("text/plain").is_none());
    }
}

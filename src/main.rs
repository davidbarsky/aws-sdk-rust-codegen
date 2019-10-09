use http::{method::Method, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, convert::TryFrom};

type Err = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug, PartialEq, Deserialize)]
pub struct BotocoreModel {
    pub version: String,
    pub metadata: Metadata,
    pub operations: HashMap<String, Operation>,
    pub shapes: HashMap<String, Shape>,
    pub documentation: Markdown,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    api_version: String,
    endpoint_prefix: String,
    // Specifies the service protocol. Should be one of "json", "rest-json", "rest-xml", or "query".
    protocol: Protocol,
    // A shorter name for the service. Often used to generate class names for SDKs. For example: "Amazon S3", which in the Ruby SDK would generate a client class in the `Aws::S3` namespace rather than `Aws::SimpleStorage`.
    service_full_name: String,
    service_id: String,
    signature_version: Signature,
}
#[derive(Debug, PartialEq, Deserialize)]
pub enum Protocol {
    #[serde(rename = "rest-json")]
    RestJson,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "rest-xml")]
    RestXml,
    #[serde(rename = "query")]
    Query,
}

#[derive(Debug, PartialEq, Deserialize)]
pub enum Signature {
    #[serde(rename = "v4")]
    V4,
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct Operation {
    pub name: String,
    pub http: HttpBindings,
    pub input: ShapeReference,
    pub output: Option<ShapeReference>,
    pub errors: Vec<ShapeReference>,
    pub documentation: Markdown,
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(from = "String")]
pub struct Markdown(String);

impl From<String> for Markdown {
    fn from(s: String) -> Self {
        Markdown(html2md::parse_html(&s))
    }
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Shape {
    #[serde(rename = "structure")]
    Structure {
        members: HashMap<String, ShapeMember>,
        documentation: Option<String>,
        required: Option<Vec<String>>,
    },
    #[serde(rename = "string")]
    String {
        #[serde(flatten)]
        contents: HashMap<String, Value>,
    },
    #[serde(rename = "map")]
    Map {
        key: ShapeReference,
        value: ShapeReference,
    },
    #[serde(rename = "list")]
    List { member: ShapeReference },
    // i32
    #[serde(rename = "integer")]
    Integer(Value),
    // i64
    #[serde(rename = "long")]
    Long(Value),
    #[serde(rename = "double")]
    Double(Value),
    #[serde(rename = "blob")]
    Blob(Value),
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "timestamp")]
    Timestamp,
}
#[derive(Debug, PartialEq, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShapeMember {
    #[serde(flatten)]
    shape: ShapeReference,
    documentation: Option<Markdown>,
    #[serde(flatten)]
    location: Option<Location>,
    streaming: Option<bool>,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    location: String,
    location_name: String,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Clone)]
pub struct ShapeReference {
    shape: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Member {
    pub target: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpBindingsTemp {
    method: String,
    request_uri: String,
    response_code: Option<u16>,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(try_from = "HttpBindingsTemp")]
pub struct HttpBindings {
    method: Method,
    request_uri: String,
    response_code: Option<StatusCode>,
}

impl TryFrom<HttpBindingsTemp> for HttpBindings {
    type Error = Err;
    fn try_from(value: HttpBindingsTemp) -> Result<Self, Self::Error> {
        let code: Option<StatusCode> = match value.response_code {
            Some(v) => Some(StatusCode::from_u16(v)?),
            None => None,
        };

        let method = Method::from_bytes(&value.method.as_bytes())?;
        let http = HttpBindings {
            method,
            request_uri: value.request_uri,
            response_code: code,
        };

        Ok(http)
    }
}

#[derive(Debug, PartialEq)]
struct ResolvedOperation {
    pub name: String,
    pub http: HttpBindings,
    pub input: Shape,
    pub output: Option<Shape>,
    pub errors: Vec<Shape>,
    pub documentation: Markdown,
}

fn resolve(op: Operation, shapes: &HashMap<String, Shape>) -> ResolvedOperation {
    ResolvedOperation {
        name: op.name,
        http: op.http,
        input: shapes[&op.input.shape].clone(),
        output: op.output.map(|o| shapes[&o.shape].clone()),
        errors: op
            .errors
            .into_iter()
            .map(|o| shapes[&o.shape].clone())
            .collect::<Vec<Shape>>(),
        documentation: op.documentation,
    }
}

#[test]
fn it_works() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let def = std::fs::read_to_string("test-data/lambda.json")?;
    let def = serde_json::from_str::<BotocoreModel>(&def)?;
    let create_alias_request = def.operations["CreateFunction"].clone();
    let resolved = resolve(create_alias_request, &def.shapes);
    dbg!(&resolved);

    Ok(())
}

fn main() {}

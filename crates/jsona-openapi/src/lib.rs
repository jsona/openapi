mod openapi;

use std::{
    cell::RefCell, collections::HashSet, convert::TryFrom, fmt::Display, rc::Rc, str::FromStr,
};

use indexmap::IndexMap;
use jsona::dom::{self, Key, KeyOrIndex, Keys, Node, Object};
use jsona_schema::{SchemaError, SchemaParser};
use serde_json::Value;
use thiserror::Error;

pub use jsona_schema::Schema;
pub use openapi::*;

#[derive(Clone, Debug, Error)]
pub enum Error {
    #[error("invalid jsona")]
    InvalidJsona(#[from] dom::ParseError),
    #[error("invalid openapi")]
    InvalidOpenapi { errors: Vec<OpenapiError> },
}

#[derive(Clone, Debug)]
pub struct OpenapiError {
    pub keys: Keys,
    pub message: String,
}

impl OpenapiError {
    pub fn new<T: ToString>(keys: Keys, message: T) -> Self {
        Self {
            keys,
            message: message.to_string(),
        }
    }
}

impl From<SchemaError> for OpenapiError {
    fn from(err: SchemaError) -> Self {
        Self {
            keys: err.keys().clone(),
            message: err.to_string(),
        }
    }
}

impl Display for OpenapiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.keys.is_empty() {
            write!(f, "{}", self.message)
        } else {
            write!(f, "{} at {}", self.message, self.keys)
        }
    }
}

type OpenapiResult<T> = std::result::Result<T, OpenapiError>;

impl FromStr for Openapi {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let node: Node = s.parse()?;
        OpenapiParser::parse(&node).map_err(|errors| Error::InvalidOpenapi { errors })
    }
}

impl TryFrom<&Node> for Openapi {
    type Error = Vec<OpenapiError>;

    fn try_from(value: &Node) -> Result<Self, Self::Error> {
        OpenapiParser::parse(value)
    }
}

struct OpenapiParser {
    openapi: Openapi,
    routes: HashSet<String>,
    errors: Vec<OpenapiError>,
    defs: Rc<RefCell<IndexMap<String, Schema>>>,
}

impl OpenapiParser {
    fn parse(node: &Node) -> Result<Openapi, Vec<OpenapiError>> {
        let mut errors: Vec<OpenapiError> = vec![];
        let routes: HashSet<String> = HashSet::default();
        let mut openapi = Self::parse_openapi(&mut errors, node);
        let schemas = get_components_mut(&mut openapi)
            .schemas
            .take()
            .unwrap_or_default();
        let mut parser = OpenapiParser {
            openapi,
            routes,
            errors,
            defs: Rc::new(RefCell::new(schemas)),
        };
        parser.parse_paths(node);
        let OpenapiParser {
            mut openapi,
            errors,
            defs,
            ..
        } = parser;
        if errors.is_empty() {
            if !defs.borrow().is_empty() {
                get_components_mut(&mut openapi).schemas = Some(defs.take());
            }
            Ok(openapi)
        } else {
            Err(errors)
        }
    }

    fn parse_openapi(errors: &mut Vec<OpenapiError>, value: &Node) -> Openapi {
        let mut spec = Openapi {
            openapi: "3.0".into(),
            info: Info {
                title: "openapi".into(),
                version: "0.1.0".into(),
                ..Default::default()
            },
            ..Default::default()
        };
        match value.get_as_object("@openapi") {
            Some((key, Some(value))) => {
                let keys = Keys::single(key);
                let mut value = Node::from(value).to_plain_json();
                if let Value::Object(ref mut obj) = value {
                    if obj.get("info").is_none() {
                        obj.insert(
                            "info".into(),
                            serde_json::to_value(spec.info.clone()).unwrap(),
                        );
                    }
                    if obj.get("openapi").is_none() {
                        obj.insert(
                            "openapi".into(),
                            serde_json::to_value(spec.openapi.clone()).unwrap(),
                        );
                    }
                }
                if let Value::Object(ref mut v) = value {
                    v.insert("paths".into(), Value::Object(serde_json::Map::new()));
                }
                match serde_json::from_value(value) {
                    Ok(v) => spec = v,
                    Err(error) => errors.push(OpenapiError::new(
                        keys,
                        format!("invalid spec value, {error}"),
                    )),
                }
            }
            Some((key, None)) => {
                errors.push(OpenapiError::new(Keys::single(key), "must be object"))
            }
            None => {}
        }
        spec
    }

    fn parse_paths(&mut self, node: &Node) {
        if let Some(object) = node.as_object() {
            for (key, value) in object.value().read().iter() {
                if let Err(error) = self.parse_endpoint(key, value) {
                    self.errors.push(error);
                }
            }
        } else {
            self.errors
                .push(OpenapiError::new(Keys::default(), "must be object"))
        }
    }

    fn parse_endpoint(&mut self, key: &Key, value: &Node) -> OpenapiResult<()> {
        let operation_id = key.value();
        let keys = Keys::single(key.clone());
        if !value.is_object() {
            return Err(OpenapiError::new(keys, "must be object"));
        }
        let mut operation = self.parse_endpoint_annotation(&keys, value)?;
        operation.operation_id = Some(operation_id.into());
        let (method, path_parts) = self.parse_route(&keys, value)?;
        let pathname = self.parse_req(&mut operation, &keys, value, &path_parts)?;
        self.parse_res(&mut operation, &keys, value)?;
        let path_item = self
            .openapi
            .paths
            .entry(pathname)
            .or_insert(Default::default());
        method.add_operation(path_item, operation);
        Ok(())
    }

    fn parse_endpoint_annotation(&mut self, keys: &Keys, value: &Node) -> OpenapiResult<Operation> {
        match value.get_as_object("@endpoint") {
            Some((key, Some(value))) => {
                let mut value = Node::from(value).to_plain_json();
                value
                    .as_object_mut()
                    .unwrap()
                    .insert("responses".into(), Value::Object(Default::default()));
                serde_json::from_value(value).map_err(|error| {
                    OpenapiError::new(keys.join(key), format!("invalid endpoint value, {error}"))
                })
            }
            Some((key, None)) => Err(OpenapiError::new(keys.join(key), "must be object")),
            None => Ok(Operation::default()),
        }
    }

    fn parse_route(
        &mut self,
        keys: &Keys,
        value: &Node,
    ) -> OpenapiResult<(MethodKind, Vec<String>)> {
        match value.get_as_string("route") {
            Some((key, Some(value))) => {
                let keys = keys.join(key);
                let splitted_route: Vec<&str> = value.value().split(' ').collect();
                let err = || OpenapiError::new(keys.clone(), "is invalid");
                if splitted_route.len() != 2 {
                    return Err(err());
                }
                let method = MethodKind::from_str(splitted_route[0]).ok_or_else(err)?;
                let path = splitted_route[1].trim();
                let path_parts: Vec<String> = path.split('/').map(|v| v.to_string()).collect();
                let canonical_route = format!("{} {}", method, path);
                if !self.routes.insert(canonical_route) {
                    return Err(OpenapiError::new(keys, "is conflict"));
                }
                Ok((method, path_parts))
            }
            Some((key, None)) => Err(OpenapiError::new(keys.join(key), "must be string")),
            None => Err(OpenapiError::new(keys.clone(), "miss route")),
        }
    }

    fn parse_req(
        &mut self,
        operation: &mut Operation,
        keys: &Keys,
        value: &Node,
        path_parts: &[String],
    ) -> OpenapiResult<String> {
        match value.get_as_object("req") {
            Some((key, Some(value))) => {
                let keys = keys.join(key);
                let pathname = self.parse_req_params(operation, &keys, &value, path_parts)?;
                for (key, value) in value.value().read().iter() {
                    match key.value() {
                        "query" | "headers" => {
                            self.parse_req_parameters(
                                operation,
                                key.value(),
                                &keys.join(key.clone()),
                                value,
                            )?;
                        }
                        "body" => self.parse_req_body(operation, &keys.join(key.clone()), value)?,
                        _ => {}
                    }
                }
                Ok(pathname)
            }
            Some((key, None)) => Err(OpenapiError::new(keys.join(key), "must be object")),
            None => {
                if path_parts.iter().any(|v| v.as_str() == "{}") {
                    return Err(OpenapiError::new(keys.clone(), "req.params is required"));
                }
                Ok(path_parts.join("/"))
            }
        }
    }

    fn parse_req_params(
        &mut self,
        operation: &mut Operation,
        keys: &Keys,
        value: &Object,
        path_parts: &[String],
    ) -> OpenapiResult<String> {
        match Node::from(value.clone()).get_as_object("params") {
            Some((key, Some(value))) => {
                let keys = keys.join(key);
                let num_params = path_parts.iter().filter(|v| v.as_str() == "{}").count();
                let map = value.value().read();
                if num_params != map.len() {
                    return Err(OpenapiError::new(keys, "does not match route"));
                }
                let mut new_path_parts: Vec<String> = vec![];
                let mut idx = 0;
                let names: Vec<&str> = map.iter().map(|(k, _)| k.value()).collect();
                for part in path_parts {
                    if *part == "{}" {
                        new_path_parts.push(format!("{{{}}}", names[idx]));
                        idx += 1;
                    } else {
                        new_path_parts.push(part.to_string())
                    }
                }
                self.parse_req_parameters(operation, "path", &keys, &value.into())?;
                Ok(new_path_parts.join("/"))
            }
            Some((key, None)) => Err(OpenapiError::new(keys.join(key), "must be object")),
            None => {
                if path_parts.iter().any(|v| v.as_str() == "{}") {
                    return Err(OpenapiError::new(keys.clone(), "params is required"));
                }
                Ok(path_parts.join("/"))
            }
        }
    }

    fn parse_req_parameters(
        &mut self,
        operation: &mut Operation,
        location: &str,
        keys: &Keys,
        value: &Node,
    ) -> OpenapiResult<()> {
        match value.as_object() {
            Some(object) => {
                let mut parameters = vec![];
                for (key, value) in object.value().read().iter() {
                    let parameter = Parameter {
                        name: key.value().to_string(),
                        location: location.into(),
                        ..Default::default()
                    };
                    parameters.push(self.parse_parameter(
                        parameter,
                        &keys.join(key.clone()),
                        value,
                    )?);
                }
                if let Some(v) = operation.parameters.as_mut() {
                    v.extend(parameters)
                } else {
                    operation.parameters = Some(parameters);
                }
                Ok(())
            }
            None => Err(OpenapiError::new(keys.clone(), "must be object")),
        }
    }

    fn parse_req_body(
        &mut self,
        operation: &mut Operation,
        keys: &Keys,
        value: &Node,
    ) -> OpenapiResult<()> {
        let content_type = parse_string_annotation(keys, value, "@contentType")?
            .unwrap_or_else(|| "application/json".into());
        let schema = self.parse_schema(keys, value)?;
        let media_type = MediaType {
            schema: Some(schema),
            examples: if exist_annotation(value, "@example") {
                Some(OneOrMultiExample::Example {
                    example: value.to_plain_json(),
                })
            } else {
                None
            },
            ..Default::default()
        };
        let mut content = IndexMap::default();
        content.insert(content_type, media_type);
        let request_body = RequestBody {
            description: parse_string_annotation(keys, value, "@describe")?,
            required: Some(true),
            content,
        };
        operation.request_body = Some(ObjectOrReference::Object(request_body));
        Ok(())
    }

    fn parse_res(
        &mut self,
        operation: &mut Operation,
        keys: &Keys,
        value: &Node,
    ) -> OpenapiResult<()> {
        match value.get_as_object("res") {
            Some((key, Some(value))) => {
                let keys = keys.join(key);
                for (key, value) in value.value().read().iter() {
                    let keys = keys.join(key.clone());
                    let status = key
                        .value()
                        .parse::<u32>()
                        .map_err(|_| OpenapiError::new(keys.clone(), "should be status code"))?;
                    if !(100..=599).contains(&status) {
                        return Err(OpenapiError::new(keys, "must be integer in [100, 600)"));
                    }
                    let description =
                        parse_string_annotation(&keys, value, "@describe")?.unwrap_or_default();
                    let mut response = Response {
                        description,
                        ..Default::default()
                    };

                    let with_header = exist_annotation(value, "@withHeader");

                    if with_header {
                        match value.as_object() {
                            Some(object) => {
                                for (key, value) in object.value().read().iter() {
                                    match key.value() {
                                        "headers" => self.parse_res_header(
                                            &mut response,
                                            &keys.join(key.clone()),
                                            value,
                                        )?,
                                        "body" => self.parse_res_body(
                                            &mut response,
                                            &keys.join(key.clone()),
                                            value,
                                        )?,
                                        _ => {}
                                    }
                                }
                            }
                            None => {
                                return Err(OpenapiError::new(keys, "must be object"));
                            }
                        }
                    } else {
                        self.parse_res_body(&mut response, &keys, value)?;
                    }

                    operation.responses.insert(status.to_string(), response);
                }
                Ok(())
            }
            Some((key, None)) => Err(OpenapiError::new(keys.join(key), "must be object")),
            None => {
                let default_response = Response {
                    description: Default::default(),
                    ..Default::default()
                };
                operation.responses.insert("200".into(), default_response);
                Ok(())
            }
        }
    }

    fn parse_parameter(
        &mut self,
        mut parameter: Parameter,
        keys: &Keys,
        value: &Node,
    ) -> OpenapiResult<ObjectOrReference<Parameter>> {
        if let Some(ref_val) = parse_ref_annotation(keys, value, "#/components/parameters/")? {
            return Ok(ref_val);
        }
        parameter.description = parse_string_annotation(keys, value, "@describe")?;
        parameter.required = Some(!exist_annotation(value, "@optional"));
        parameter.schema = Some(self.parse_schema(keys, value)?);
        parameter.examples = if exist_annotation(value, "@example") {
            Some(OneOrMultiExample::Example {
                example: value.to_plain_json(),
            })
        } else {
            None
        };

        let parameter_object = ObjectOrReference::Object(parameter);

        if let Some(name) = parse_string_annotation(keys, value, "@def")? {
            return self.def_parameters(name, parameter_object);
        }
        Ok(parameter_object)
    }

    fn parse_res_header(
        &mut self,
        response: &mut Response,
        keys: &Keys,
        value: &Node,
    ) -> OpenapiResult<()> {
        match value.as_object() {
            Some(value) => {
                for (key, value) in value.value().read().iter() {
                    let keys = keys.join(key.clone());
                    let header = Header {
                        description: parse_string_annotation(&keys, value, "@describe")?,
                        required: Some(!exist_annotation(value, "@optional")),
                        schema: Some(self.parse_schema(&keys, value)?),
                        ..Default::default()
                    };
                    let header_object = ObjectOrReference::Object(header);
                    response
                        .headers
                        .get_or_insert(Default::default())
                        .insert(key.value().to_string(), header_object);
                }
                Ok(())
            }
            None => Err(OpenapiError::new(keys.clone(), "must be object")),
        }
    }

    fn parse_res_body(
        &mut self,
        response: &mut Response,
        keys: &Keys,
        value: &Node,
    ) -> OpenapiResult<()> {
        let content_type = parse_string_annotation(keys, value, "@contentType")?
            .unwrap_or_else(|| "application/json".into());
        let schema = self.parse_schema(keys, value)?;
        let media_type = MediaType {
            schema: Some(schema),
            examples: if exist_annotation(value, "@example") {
                Some(OneOrMultiExample::Example {
                    example: value.to_plain_json(),
                })
            } else {
                None
            },
            ..Default::default()
        };
        response
            .content
            .get_or_insert(Default::default())
            .insert(content_type, media_type);
        Ok(())
    }

    fn parse_schema(&mut self, keys: &Keys, value: &Node) -> OpenapiResult<Schema> {
        let scope = SchemaParser {
            keys: keys.clone(),
            node: value.clone(),
            defs: self.defs.clone(),
            ref_prefix: Rc::new("#/components/schemas/".to_string()),
            prefer_optional: false,
        };
        let mut schema = scope.parse()?;
        schema.description = None;
        Ok(schema)
    }

    fn def_parameters(
        &mut self,
        name: String,
        value: ObjectOrReference<Parameter>,
    ) -> OpenapiResult<ObjectOrReference<Parameter>> {
        let components = get_components_mut(&mut self.openapi);
        if components.parameters.is_none() {
            components.parameters = Some(Default::default());
        }
        components
            .parameters
            .as_mut()
            .unwrap()
            .insert(name.clone(), value);
        return Ok(ObjectOrReference::Ref {
            ref_path: format!("#/components/parameters/{}", name),
        });
    }
}

fn get_components_mut(spec: &mut Openapi) -> &mut Components {
    if spec.components.is_none() {
        spec.components = Some(Default::default());
    }
    spec.components.as_mut().unwrap()
}

fn exist_annotation(value: &Node, name: &str) -> bool {
    value.get(&KeyOrIndex::annotation(name)).is_some()
}

fn parse_string_annotation(keys: &Keys, value: &Node, name: &str) -> OpenapiResult<Option<String>> {
    match value.get_as_string(name) {
        Some((_, Some(value))) => Ok(Some(value.value().to_string())),
        Some((key, None)) => Err(OpenapiError::new(keys.join(key), "must be string")),
        None => Ok(None),
    }
}

fn parse_ref_annotation<T>(
    keys: &Keys,
    value: &Node,
    ref_prefix: &str,
) -> OpenapiResult<Option<ObjectOrReference<T>>> {
    match parse_string_annotation(keys, value, "@ref")? {
        Some(ref_value) => Ok(Some(ObjectOrReference::Ref {
            ref_path: format!("{}{}", ref_prefix, ref_value),
        })),
        None => Ok(None),
    }
}

enum MethodKind {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl MethodKind {
    pub fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "get" => Some(MethodKind::Get),
            "post" => Some(MethodKind::Post),
            "put" => Some(MethodKind::Put),
            "delete" => Some(MethodKind::Delete),
            "patch" => Some(MethodKind::Patch),
            _ => None,
        }
    }
    pub fn add_operation(&self, path_item: &mut PathItem, operation: Operation) {
        match self {
            MethodKind::Get => path_item.get = Some(operation),
            MethodKind::Post => path_item.post = Some(operation),
            MethodKind::Put => path_item.put = Some(operation),
            MethodKind::Delete => path_item.delete = Some(operation),
            MethodKind::Patch => path_item.patch = Some(operation),
        };
    }
}
impl Display for MethodKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MethodKind::Get => write!(f, "get"),
            MethodKind::Post => write!(f, "post"),
            MethodKind::Put => write!(f, "put"),
            MethodKind::Delete => write!(f, "delete"),
            MethodKind::Patch => write!(f, "patch"),
        }
    }
}

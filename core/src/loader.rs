use crate::error::{Error, Result};
use jsona::ast::{self, Annotation, Ast};
use jsona::Loader as JLoader;
use jsona::Position;
use jsona_openapi_spec::*;
use serde_json::{Map, Value};

pub fn parse(input: &str) -> Result<Spec> {
    Loader::load_from_str(input)
}
pub struct Loader {
    spec: Spec,
}

impl Loader {
    pub fn load_from_str(input: &str) -> Result<Spec> {
        let ast = JLoader::load_from_str(input)?;
        Self::load_from_ast(ast)
    }
    pub fn load_from_ast(ast: Ast) -> Result<Spec> {
        if let Some(openapi) = ast
            .get_annotations()
            .iter()
            .find(|annotation| annotation.name == "openapi")
        {
            let spec = Self::load_openapi(&openapi.value, openapi.position)?;
            let mut loader = Loader { spec };
            if let Ast::Object(ast::Object { properties, .. }) = &ast {
                properties
                    .iter()
                    .map(|prop| loader.parse_endpoint(prop))
                    .collect::<Result<_>>()?;
                Ok(loader.spec)
            } else {
                Err(Error::invalid_ast(
                    "must be object",
                    &vec![],
                    openapi.position,
                ))
            }
        } else {
            Err(Error::invalid_ast(
                "miss @openapi",
                &vec![],
                Default::default(),
            ))
        }
    }
    fn load_openapi(value: &Value, position: Position) -> Result<Spec> {
        if !value.is_object() {
            return Err(Error::invalid_annotation(
                "must be object",
                "openapi",
                &[],
                position,
            ));
        }
        let mut value = value.clone();
        if let Value::Object(ref mut v) = value {
            v.insert("paths".into(), Value::Object(Map::new()));
        }
        serde_json::from_value(value).map_err(|_| {
            Error::invalid_annotation(
                "is invalid",
                "openapi",
                &[],
                position,
            )
        })
    }
    fn parse_endpoint(&mut self, prop: &ast::Property) -> Result<()> {
        let operation_id = prop.key.as_str();
        let scope = vec![operation_id];
        if !prop.value.is_object() {
            return Err(Error::invalid_ast("must be object", &scope, prop.position));
        }
        let mut operation = self.parse_endpoint_annotation(&prop.value, &scope)?;
        operation.operation_id = Some(operation_id.into());
        let (method, path_parts) = self.parse_route(&prop.value, &enter_scope(&scope, "route"))?;
        let req_scope = enter_scope(&scope, "req");
        let pathname: String;
        if let Some(req) = prop.value.retrive(&["req"]) {
            pathname = self.parse_req(&mut operation, req, &path_parts, &req_scope)?;
        } else {
            if path_parts.iter().find(|v| **v == "{}").is_some() {
                return Err(Error::invalid_ast(
                    "must have params",
                    &req_scope,
                    prop.value.get_position().clone(),
                ));
            }
            pathname = path_parts.join("/");
        }
        let res_path = vec!["res"];
        if let Some(res) = prop.value.retrive(&res_path) {
            self.parse_res(&mut operation, res, &enter_scope(&scope, "res"))?;
        } else {
            let default_response = Response {
                description: "-".into(),
                ..Default::default()
            };
            operation.responses.insert("200".into(), default_response);
        }
        if self.spec.paths.get(&pathname).is_none() {
            self.spec.paths.insert(pathname.clone(), Default::default());
        }
        let path_item = self.spec.paths.get_mut(&pathname).unwrap();
        method.add_operation(path_item, operation);
        Ok(())
    }
    fn parse_endpoint_annotation(&mut self, value: &Ast, scope: &[&str]) -> Result<Operation> {
        let endpoint_annotations = value.get_annotations();
        if let Some(Annotation {
            position, value, ..
        }) = endpoint_annotations.iter().find(|v| v.name == "endpoint")
        {
            if !value.is_object() {
                return Err(Error::invalid_annotation(
                    "must be object",
                    "endpoint",
                    &scope,
                    position.clone(),
                ));
            }
            let mut value = value.clone();
            value
                .as_object_mut()
                .unwrap()
                .insert("responses".into(), Value::Object(Default::default()));
            serde_json::from_value(value).map_err(|_| {
                Error::invalid_annotation(
                    "is invalid",
                    "endpoint",
                    &scope,
                    position.clone(),
                )
            })
        } else {
            Ok(Operation::default())
        }
    }
    fn parse_route<'a>(
        &mut self,
        value: &'a Ast,
        scope: &[&str],
    ) -> Result<(MethodKind, Vec<&'a str>)> {
        match value.retrive(&["route"]) {
            Some(Ast::String(route)) => {
                let splited_route: Vec<&str> = route.value.split(" ").collect();
                let err = || Error::invalid_ast("is invalid", &scope, route.position.clone());
                if splited_route.len() != 2 {
                    return Err(err());
                }
                let method = MethodKind::from_str(splited_route[0]).ok_or(err())?;
                let path_parts: Vec<&str> = splited_route[1].trim().split("/").collect();
                Ok((method, path_parts))
            }
            Some(route) => Err(Error::invalid_ast(
                "must be string",
                &scope,
                route.get_position().clone(),
            )),
            None => Err(Error::invalid_ast(
                "miss",
                &scope,
                value.get_position().clone(),
            )),
        }
    }
    fn parse_req(
        &mut self,
        operation: &mut Operation,
        req: &Ast,
        path_parts: &[&str],
        scope: &[&str],
    ) -> Result<String> {
        if !req.is_object() {
            return Err(Error::invalid_ast(
                "must be object",
                &scope,
                req.get_position().clone(),
            ));
        }
        let pathname: String;
        let params_scope = enter_scope(scope, "params");
        if let Some(params) = req.retrive(&["params"]) {
            let path = self.parse_req_params(operation, params, &path_parts, &params_scope)?;
            pathname = path;
        } else {
            if path_parts.iter().find(|v| **v == "{}").is_some() {
                return Err(Error::invalid_ast(
                    "miss",
                    &params_scope,
                    req.get_position().clone(),
                ));
            }
            pathname = path_parts.join("/");
        }
        for location in ["query", "header"].iter() {
            if let Some(node) = req.retrive(&[location]) {
                self.parse_req_parameters(
                    operation,
                    location,
                    node,
                    &enter_scope(scope, location),
                )?;
            }
        }
        if let Some(body) = req.retrive(&["body"]) {
            self.parse_req_body(operation, body, &enter_scope(scope, "body"))?;
        }
        Ok(pathname)
    }
    fn parse_req_params(
        &mut self,
        operation: &mut Operation,
        params: &Ast,
        path_parts: &[&str],
        scope: &[&str],
    ) -> Result<String> {
        if !params.is_object() {
            return Err(Error::invalid_ast(
                "must be object",
                &scope,
                params.get_position().clone(),
            ));
        }
        let params_object = params.as_object().unwrap();
        let num_params = path_parts.iter().filter(|v| **v == "{}").count();
        if num_params != params_object.properties.len() {
            return Err(Error::invalid_ast(
                "does not match route",
                &scope,
                params_object.position.clone(),
            ));
        }
        let mut new_path_parts: Vec<String> = vec![];
        let mut idx = 0;
        for part in path_parts {
            if *part == "{}" {
                new_path_parts.push(format!("{{{}}}", params_object.properties[idx].key));
                idx += 1;
            } else {
                new_path_parts.push(part.to_string())
            }
        }

        self.parse_req_parameters(operation, "path", params, scope)?;
        Ok(new_path_parts.join("/"))
    }
    fn parse_req_parameters(
        &mut self,
        operation: &mut Operation,
        location: &str,
        node: &Ast,
        scope: &[&str],
    ) -> Result<()> {
        if !node.is_object() {
            return Err(Error::invalid_ast(
                "must be object",
                &scope,
                node.get_position().clone(),
            ));
        }
        let query = node.as_object().unwrap();
        let mut parameters = vec![];
        for prop in query.properties.iter() {
            let parameter = Parameter {
                name: prop.key.clone(),
                location: location.into(),
                ..Default::default()
            };
            parameters.push(self.parse_parameter(
                &prop.value,
                parameter,
                &enter_scope(scope, prop.key.as_str()),
            )?);
        }
        extend_operation_parameters(operation, parameters);
        Ok(())
    }
    fn parse_req_body(
        &mut self,
        operation: &mut Operation,
        body: &Ast,
        scope: &[&str],
    ) -> Result<()> {
        let annotations = body.get_annotations();
        let content_type = self.parse_content_type_annotation(annotations, scope)?;
        let schema = self.parse_schema(body, false, scope)?;
        let media_type = MediaType {
            schema: Some(schema),
            ..Default::default()
        };
        let mut request_body = RequestBody::default();
        request_body.content.insert(content_type, media_type);
        request_body.required = Some(true);
        operation.request_body = Some(ObjectOrReference::Object(request_body));
        Ok(())
    }
    fn parse_res(&mut self, opration: &mut Operation, res: &Ast, scope: &[&str]) -> Result<()> {
        if !res.is_object() {
            return Err(Error::invalid_ast(
                "must be object",
                &scope,
                res.get_position().clone(),
            ));
        }
        let res = res.as_object().unwrap();
        for prop in res.properties.iter() {
            let prop_scope = enter_scope(scope, prop.key.as_str());
            let prop_annotations = prop.value.get_annotations();
            let status = prop.key.parse::<u32>().map_err(|_| {
                Error::invalid_ast("should be status code", &prop_scope, prop.position.clone())
            })?;
            if status < 100 || status > 599 {
                return Err(Error::invalid_ast(
                    "must be integer in [100, 600)",
                    &prop_scope,
                    prop.position.clone(),
                ));
            }
            let content_type = self.parse_content_type_annotation(prop_annotations, &prop_scope)?;
            let schema = self.parse_schema(&prop.value, false, scope)?;
            let media_type = MediaType {
                schema: Some(schema),
                ..Default::default()
            };
            let description = self
                .parse_description_annnotation(prop_annotations, &prop_scope)?
                .unwrap_or("-".into());
            let mut response = Response {
                description,
                ..Default::default()
            };
            response
                .content
                .get_or_insert(Default::default())
                .insert(content_type, media_type);
            opration.responses.insert(prop.key.clone(), response);
        }
        Ok(())
    }
    fn parse_parameter(
        &mut self,
        value: &Ast,
        mut parameter: Parameter,
        scope: &[&str],
    ) -> Result<ObjectOrReference<Parameter>> {
        let annotations = value.get_annotations();
        if let Some(ref_val) =
            self.parse_use_annotation(annotations, ComponentKind::Parameter, scope)?
        {
            return Ok(ref_val);
        }
        parameter.description = self.parse_description_annnotation(annotations, scope)?;
        parameter.required = Some(!self.parse_optional_annnotation(annotations, scope));
        parameter.schema = Some(self.parse_schema(value, true, scope)?);
        let parameter_object = ObjectOrReference::Object(parameter);
        if let Some(name) = self.parse_save_annotation(annotations, scope)? {
            return self.save_parameters(name, parameter_object);
        }
        Ok(parameter_object)
    }
    fn parse_schema(&mut self, value: &Ast, is_parameter: bool, scope: &[&str]) -> Result<Schema> {
        let annotations = value.get_annotations();
        if !is_parameter {
            if let Some(ObjectOrReference::Ref { ref_path }) = self
                .parse_use_annotation::<ObjectOrReference<Schema>>(
                    annotations,
                    ComponentKind::Schema,
                    scope,
                )?
            {
                return Ok(Schema {
                    ref_path: Some(ref_path),
                    ..Default::default()
                });
            }
        }
        let mut schema = self
            .parse_schema_annotation(annotations, scope)?
            .unwrap_or_default();

        self.parse_description_annnotation(annotations, scope)?
            .map(|v| schema.description = Some(v));
        let mut set_type = |ty: &str| {
            if schema.schema_type.is_none() {
                schema.schema_type = Some(ty.into());
            }
        };

        match value {
            Ast::Null(_) => {
                return Err(Error::invalid_ast(
                    "can not be null",
                    scope,
                    value.get_position().clone(),
                ));
            }
            Ast::Boolean(_) => {
                set_type("boolean");
                schema.example = Some(value.into());
            }
            Ast::Integer(_) => {
                set_type("integer");
                schema.example = Some(value.into());
                if schema.format.is_none() {
                    schema.format = Some("int64".into());
                }
            }
            Ast::Float(_) => {
                set_type("number");
                schema.example = Some(value.into());
            }
            Ast::String(_) => {
                set_type("string");
                schema.example = Some(value.into());
            }
            Ast::Array(ast::Array { elements, .. }) => {
                let combine = self.parse_schema_combine_annotation(annotations, scope)?;
                if combine.is_none() {
                    if elements.len() == 0 {
                        set_type("array");
                    } else {
                        if schema.items.is_none() {
                            let items_schema =
                                self.parse_schema(&elements[0], false, &enter_scope(scope, "0"))?;
                            schema.items = Some(Box::new(items_schema));
                        }
                    }
                } else {
                    let mut elem_schemas: Vec<Schema> = vec![];
                    for (i, elem) in elements.iter().enumerate() {
                        elem_schemas.push(self.parse_schema(
                            elem,
                            is_parameter,
                            &enter_scope(scope, format!("{}", i).as_str()),
                        )?);
                    }
                    match combine.unwrap().as_str() {
                        "allOf" => schema.all_of = Some(elem_schemas),
                        "anyOf" => schema.any_of = Some(elem_schemas),
                        "oneOf" => schema.one_of = Some(elem_schemas),
                        _ => unreachable!(),
                    }
                }
            }
            Ast::Object(ast::Object { properties, .. }) => {
                set_type("object");
                for prop in properties.iter() {
                    let prop_scope = enter_scope(scope, prop.key.as_str());
                    let prop_schema = self.parse_schema(&prop.value, false, &prop_scope)?;
                    schema
                        .properties
                        .get_or_insert(Default::default())
                        .insert(prop.key.clone(), prop_schema);
                    if !self.parse_optional_annnotation(prop.value.get_annotations(), &prop_scope) {
                        schema
                            .required
                            .get_or_insert(Default::default())
                            .push(prop.key.to_owned());
                    }
                }
            }
        }
        if let Some(name) = self.parse_save_annotation(annotations, scope)? {
            return self.save_schemas(name, schema);
        }
        Ok(schema)
    }
    fn parse_use_annotation<T>(
        &mut self,
        annotations: &[Annotation],
        kind: ComponentKind,
        scope: &[&str],
    ) -> Result<Option<ObjectOrReference<T>>> {
        let name = "use";
        match annotations.iter().find(|v| v.name == name) {
            Some(Annotation {
                value: Value::String(value),
                ..
            }) => Ok(Some(ObjectOrReference::Ref {
                ref_path: kind.compute_ref(value.to_owned()),
            })),
            Some(annotation) => Err(Error::invalid_annotation(
                "must be string",
                name,
                scope,
                annotation.position.clone(),
            )),
            None => Ok(None),
        }
    }
    fn parse_save_annotation(
        &mut self,
        annotations: &[Annotation],
        scope: &[&str],
    ) -> Result<Option<String>> {
        let name = "save";
        match annotations.iter().find(|v| v.name == name) {
            Some(Annotation {
                value: Value::String(value),
                ..
            }) => Ok(Some(value.to_owned())),
            Some(annotation) => Err(Error::invalid_annotation(
                "must be string",
                name,
                scope,
                annotation.position.clone(),
            )),
            None => Ok(None),
        }
    }
    fn parse_description_annnotation(
        &mut self,
        annotations: &[Annotation],
        scope: &[&str],
    ) -> Result<Option<String>> {
        let name = "description";
        match annotations.iter().find(|v| v.name == name) {
            Some(Annotation {
                value: Value::String(description),
                ..
            }) => Ok(Some(description.to_owned())),
            Some(annotation) => Err(Error::invalid_annotation(
                "must be string",
                name,
                scope,
                annotation.position.clone(),
            )),
            None => Ok(None),
        }
    }
    fn parse_optional_annnotation(&mut self, annotations: &[Annotation], _scope: &[&str]) -> bool {
        annotations.iter().find(|v| v.name == "optional").is_some()
    }
    fn parse_schema_annotation(
        &mut self,
        annotations: &[Annotation],
        scope: &[&str],
    ) -> Result<Option<Schema>> {
        let name = "schema";
        match annotations.iter().find(|v| v.name == name) {
            Some(Annotation {
                value, position, ..
            }) => {
                if value.is_object() {
                    let schema = serde_json::from_value(value.clone()).map_err(|_| {
                        Error::invalid_annotation(
                            "is invalid",
                            name,
                            &[],
                            position.clone(),
                        )
                    })?;
                    Ok(Some(schema))
                } else {
                    Err(Error::invalid_annotation(
                        "must be object",
                        "schema",
                        scope,
                        position.clone(),
                    ))
                }
            }
            None => Ok(None),
        }
    }
    fn parse_schema_combine_annotation(
        &mut self,
        annotations: &[Annotation],
        scope: &[&str],
    ) -> Result<Option<String>> {
        let name = "schemaCombine";
        match annotations.iter().find(|v| v.name == name) {
            Some(Annotation {
                value: Value::String(value),
                position,
                ..
            }) => match value.as_str() {
                "anyOf" | "oneOf" | "allOf" => Ok(Some(value.clone())),
                _ => Err(Error::invalid_annotation("", name, scope, position.clone())),
            },
            Some(annotation) => Err(Error::invalid_annotation(
                "must be string",
                name,
                scope,
                annotation.position.clone(),
            )),
            None => Ok(None),
        }
    }
    fn parse_content_type_annotation(
        &mut self,
        annotations: &[Annotation],
        scope: &[&str],
    ) -> Result<String> {
        let name = "contentType";
        match annotations.iter().find(|v| v.name == name) {
            Some(Annotation {
                value: Value::String(value),
                ..
            }) => Ok(value.clone()),
            Some(annotation) => Err(Error::invalid_annotation(
                "must be string",
                name,
                scope,
                annotation.position.clone(),
            )),
            None => Ok("application/json".into()),
        }
    }

    fn get_components_mut(&mut self) -> &mut Components {
        if self.spec.components.is_none() {
            self.spec.components = Some(Default::default());
        }
        self.spec.components.as_mut().unwrap()
    }
    fn save_parameters(
        &mut self,
        name: String,
        value: ObjectOrReference<Parameter>,
    ) -> Result<ObjectOrReference<Parameter>> {
        let components = self.get_components_mut();
        if components.parameters.is_none() {
            components.parameters = Some(Default::default());
        }
        components
            .parameters
            .as_mut()
            .unwrap()
            .insert(name.clone(), value);
        return Ok(ObjectOrReference::Ref {
            ref_path: ComponentKind::Parameter.compute_ref(name),
        });
    }
    fn save_schemas(&mut self, name: String, value: Schema) -> Result<Schema> {
        let components = self.get_components_mut();
        if components.schemas.is_none() {
            components.schemas = Some(Default::default());
        }
        components
            .schemas
            .as_mut()
            .unwrap()
            .insert(name.clone(), ObjectOrReference::Object(value));
        return Ok(Schema {
            ref_path: Some(ComponentKind::Schema.compute_ref(name)),
            ..Default::default()
        });
    }
}

fn extend_operation_parameters(
    operation: &mut Operation,
    parameters: Vec<ObjectOrReference<Parameter>>,
) {
    if operation.parameters.is_none() {
        operation.parameters = Some(parameters);
    } else {
        operation.parameters.as_mut().map(|v| v.extend(parameters));
    }
}

fn enter_scope<'a: 'd, 'b: 'd, 'c: 'd, 'd>(scope: &'a [&'b str], current: &'c str) -> Vec<&'d str> {
    [scope, &vec![current]].concat()
}

enum MethodKind {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

impl MethodKind {
    pub fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "get" => Some(MethodKind::GET),
            "post" => Some(MethodKind::POST),
            "put" => Some(MethodKind::PUT),
            "delete" => Some(MethodKind::DELETE),
            "patch" => Some(MethodKind::PATCH),
            _ => None,
        }
    }
    pub fn add_operation(&self, path_item: &mut PathItem, operation: Operation) {
        match self {
            MethodKind::GET => path_item.get = Some(operation),
            MethodKind::POST => path_item.post = Some(operation),
            MethodKind::PUT => path_item.put = Some(operation),
            MethodKind::DELETE => path_item.delete = Some(operation),
            MethodKind::PATCH => path_item.patch = Some(operation),
        };
    }
}

enum ComponentKind {
    Schema,
    Parameter,
}

impl ComponentKind {
    pub fn compute_ref(&self, name: String) -> String {
        match self {
            ComponentKind::Schema => format!("#/components/schemas/{}", name),
            ComponentKind::Parameter => format!("#/components/parameters/{}", name),
        }
    }
}

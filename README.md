# JSONA-OPENAPI 

Jsona openapi is a concise, developer-friendly way to describe your API contract.

Example of an API definition file api.jsona which defines a single POST endpoint to create a user:

```
{ @openapi
  createUser: { @endpoint({summary: "create a user"})
    route: "POST /users",
    req: {
      body: {
        firstName: "foo",
        lastName: "bar",
      }
    },
    res: {
      200: {
        firstName: "foo",
        lastName: "bar",
        role: "user",
      }
    }
  }
}
```

You can view generated openapi doc and swagger ui in [here](https://sigoden.github.io/jsona-openapi/?source=https://raw.githubusercontent.com/sigoden/jsona-openapi/master/core/tests/spec/readme_snippet.jsona)

A [peetstore](https://sigoden.github.io/jsona-openapi/?source=https://raw.githubusercontent.com/sigoden/jsona-openapi/master/core/tests/spec/petstore.jsona) is avaiable.

## Annotation

### @openapi

[OpenapiObject](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.1.0.md#openapi-object) provide the root object of the OpenAPI document. 

```
{ @openapi
  createUser: { @endpoint({summary: "create a user"})
    route: "POST /users",
    req: {
      body: {
        firstName: "foo",
        lastName: "bar",
      }
    },
    res: {
      200: {
        firstName: "foo",
        lastName: "bar",
        role: "user",
      }
    }
  }
}

```

### @schema
[SchemaObject](https://github.com/OAI/OpenAPI-Specification/blob/master/versions/3.1.0.md#schemaObject) allows the definition of input and output data types. it will be merged with generated schema

```
{
  endpoint1: {
    req: {
      body: {
        user: {
          email: "postmaster@example.com", @schema(format:"email")
        }
      }
    }
  }
}
```

### @save/@use

`@save`: save generated schema to components
`@use`: use the schema in components

```
{
  endpoint1: {
    req: {
      body: {
        category: { @save("Category")
          id: "",
          title: "",
        }
      }
    },
    res: {
      200: {
        category: { @use("Category")
        }
      }
    }
  }
}
```

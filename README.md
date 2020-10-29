# JSONA-OPENAPI 

Jsona openapi is a concise, developer-friendly way to describe your API contract.

Example of an API definition file api.jsona which defines a single POST endpoint to create a user:

```
{ @openapi({
    openapi: "3.0.0",
  })
  createUser: { @endpoint({summary: "create a user"})
    route: "POST /users",
    req: {
      body: {
        firstName: "foo",
        lastName: "bar",
      }
    },
    res: {
      body: {
        firstName: "foo",
        lastName: "bar",
        role: "user",
      }
    }
  }
}
```

The api.jsona will generate openapi doc below

```
{
  "openapi": "3.0.0",
  "info": {
    "title": "Sample Api",
    "version": "0.1.0"
  },
  "paths": {
    "/users": {
      "post": {
        "summary": "create a user",
        "operationId": "createUser",
        "requestBody": {
          "content": {
            "application/json": {
              "schema": {
                "type": "object",
                "properties": {
                  "firstName": {
                    "type": "string",
                    "example": "foo"
                  },
                  "lastName": {
                    "type": "string",
                    "example": "bar"
                  }
                },
                "required": [
                  "firstName",
                  "lastName"
                ]
              }
            }
          },
          "required": true
        },
        "responses": {
          "200": {
            "description": "-",
            "content": {
              "application/json": {
                "schema": {
                  "type": "object",
                  "properties": {
                    "firstName": {
                      "type": "string",
                      "example": "foo"
                    },
                    "lastName": {
                      "type": "string",
                      "example": "bar"
                    },
                    "role": {
                      "type": "string",
                      "example": "user"
                    }
                  },
                  "required": [
                    "firstName",
                    "lastName",
                    "role"
                  ]
                }
              }
            }
          }
        }
      }
    }
  }
}
```

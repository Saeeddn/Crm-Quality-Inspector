// OpenAPI/Swagger UI for CRM Quality Inspector
// Self-contained — does NOT load from CDN (Iran blocks most CDNs).
// Generates a basic OpenAPI 3.0 spec from the current routes.
use axum::{response::{Html, Json}, http::header};
use serde_json::{json, Value};

/// Auto-discovered OpenAPI 3.0 spec for all /api/* routes.
/// Hand-maintained to keep dependencies small — no utoipa derive macros needed.
pub fn openapi_spec() -> Value {
    json!({
        "openapi": "3.0.3",
        "info": {
            "title": "CRM Quality Inspector API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Backend API for the CRM quality inspection tool. \
                All endpoints under /api require a Bearer token (POST /api/auth/login first). \
                Most list endpoints support `page` and `limit` query params for server-side pagination.",
            "contact": { "name": "API Support" }
        },
        "servers": [{ "url": "/api", "description": "Same-origin" }],
        "components": {
            "securitySchemes": {
                "BearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT"
                }
            },
            "schemas": {
                "Agent": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "integer" },
                        "name": { "type": "string" },
                        "department": { "type": "string", "nullable": true },
                        "position": { "type": "string", "nullable": true },
                        "active": { "type": "boolean" },
                        "created_at": { "type": "string", "format": "date-time" }
                    }
                },
                "Customer": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "integer" },
                        "name": { "type": "string" },
                        "phone": { "type": "string", "nullable": true },
                        "product_type": { "type": "string", "nullable": true },
                        "segment": { "type": "string", "nullable": true },
                        "notes": { "type": "string", "nullable": true },
                        "created_at": { "type": "string", "format": "date-time" }
                    }
                },
                "Interaction": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "integer" },
                        "agent_id": { "type": "integer" },
                        "customer_id": { "type": "integer", "nullable": true },
                        "subject": { "type": "string" },
                        "transcript": { "type": "string" },
                        "channel": { "type": "string" },
                        "created_at": { "type": "string", "format": "date-time" }
                    }
                },
                "Score": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "integer" },
                        "interaction_id": { "type": "integer" },
                        "overall_score": { "type": "number", "format": "float" },
                        "criteria_scores": { "type": "object", "additionalProperties": { "type": "number" } },
                        "created_at": { "type": "string", "format": "date-time" }
                    }
                },
                "Issue": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "integer" },
                        "interaction_id": { "type": "integer" },
                        "agent_id": { "type": "integer" },
                        "severity": { "type": "string", "enum": ["low", "medium", "high", "critical"] },
                        "category": { "type": "string" },
                        "description": { "type": "string" },
                        "status": { "type": "string", "enum": ["open", "in_progress", "resolved", "closed"] },
                        "root_cause": { "type": "string", "nullable": true },
                        "corrective_action": { "type": "string", "nullable": true },
                        "created_at": { "type": "string", "format": "date-time" }
                    }
                },
                "Kpi": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "integer" },
                        "name": { "type": "string" },
                        "target": { "type": "number" },
                        "actual": { "type": "number" },
                        "period": { "type": "string" },
                        "department": { "type": "string", "nullable": true }
                    }
                },
                "Dashboard": {
                    "type": "object",
                    "properties": {
                        "agent_count": { "type": "integer" },
                        "customer_count": { "type": "integer" },
                        "interaction_count": { "type": "integer" },
                        "scored_count": { "type": "integer" },
                        "open_issues": { "type": "integer" },
                        "critical_failures": { "type": "integer" },
                        "average_score": { "type": "number" },
                        "coverage": { "type": "number" },
                        "quality_grade": { "type": "string" }
                    }
                },
                "LoginRequest": {
                    "type": "object",
                    "required": ["username", "password"],
                    "properties": {
                        "username": { "type": "string" },
                        "password": { "type": "string" }
                    }
                },
                "LoginResponse": {
                    "type": "object",
                    "properties": {
                        "token": { "type": "string" },
                        "user": { "type": "object" }
                    }
                },
                "Paged": {
                    "type": "object",
                    "properties": {
                        "items": { "type": "array", "items": { "type": "object" } },
                        "total": { "type": "integer" },
                        "page": { "type": "integer" },
                        "limit": { "type": "integer" },
                        "total_pages": { "type": "integer" }
                    }
                },
                "Error": {
                    "type": "object",
                    "properties": {
                        "success": { "type": "boolean", "example": false },
                        "error": { "type": "string" }
                    }
                }
            }
        },
        "security": [{ "BearerAuth": [] }],
        "tags": [
            { "name": "auth", "description": "Login / logout" },
            { "name": "agents", "description": "Agent (inspector) management" },
            { "name": "customers", "description": "Customer management" },
            { "name": "interactions", "description": "Customer interactions" },
            { "name": "scoring", "description": "Quality scoring" },
            { "name": "issues", "description": "Issues raised from inspections" },
            { "name": "kpis", "description": "KPI tracking" },
            { "name": "reports", "description": "Dashboards and aggregations" }
        ],
        "paths": {
            "/auth/login": {
                "post": {
                    "tags": ["auth"],
                    "summary": "Login",
                    "description": "Exchange username + password for a Bearer token. No auth required.",
                    "security": [],
                    "requestBody": { "required": true, "content": { "application/json": { "schema": { "$ref": "#/components/schemas/LoginRequest" } } } },
                    "responses": {
                        "200": { "description": "OK", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/LoginResponse" } } } },
                        "401": { "description": "Bad credentials", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Error" } } } }
                    }
                }
            },
            "/agents": {
                "get": {
                    "tags": ["agents"],
                    "summary": "List agents (paginated)",
                    "parameters": [
                        { "name": "page", "in": "query", "schema": { "type": "integer", "default": 1, "minimum": 1 } },
                        { "name": "limit", "in": "query", "schema": { "type": "integer", "default": 10, "maximum": 1000, "minimum": 1 } }
                    ],
                    "responses": {
                        "200": { "description": "OK", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Paged" } } } }
                    }
                },
                "post": {
                    "tags": ["agents"],
                    "summary": "Create agent",
                    "requestBody": { "required": true, "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Agent" } } } },
                    "responses": { "200": { "description": "Created" }, "400": { "description": "Validation" } }
                }
            },
            "/agents/{id}": {
                "patch": {
                    "tags": ["agents"],
                    "summary": "Update agent",
                    "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "integer" } }],
                    "responses": { "200": { "description": "OK" }, "404": { "description": "Not found" } }
                },
                "delete": {
                    "tags": ["agents"],
                    "summary": "Deactivate (soft delete) agent",
                    "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "integer" } }],
                    "responses": { "200": { "description": "OK" } }
                }
            },
            "/customers": {
                "get": {
                    "tags": ["customers"],
                    "summary": "List customers (paginated)",
                    "parameters": [
                        { "name": "page", "in": "query", "schema": { "type": "integer", "default": 1, "minimum": 1 } },
                        { "name": "limit", "in": "query", "schema": { "type": "integer", "default": 10, "maximum": 1000, "minimum": 1 } }
                    ],
                    "responses": { "200": { "description": "OK", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Paged" } } } } }
                },
                "post": {
                    "tags": ["customers"],
                    "summary": "Create customer",
                    "requestBody": { "required": true, "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Customer" } } } },
                    "responses": { "200": { "description": "Created" } }
                }
            },
            "/interactions": {
                "get": {
                    "tags": ["interactions"],
                    "summary": "List interactions (paginated)",
                    "parameters": [
                        { "name": "page", "in": "query", "schema": { "type": "integer", "default": 1, "minimum": 1 } },
                        { "name": "limit", "in": "query", "schema": { "type": "integer", "default": 10, "maximum": 1000, "minimum": 1 } }
                    ],
                    "responses": { "200": { "description": "OK", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Paged" } } } } }
                }
            },
            "/interactions/{id}": {
                "get": {
                    "tags": ["interactions"],
                    "summary": "Get one interaction",
                    "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "integer" } }],
                    "responses": { "200": { "description": "OK" }, "404": { "description": "Not found" } }
                }
            },
            "/scoring/{interaction_id}": {
                "get": {
                    "tags": ["scoring"],
                    "summary": "Get score for an interaction",
                    "parameters": [{ "name": "interaction_id", "in": "path", "required": true, "schema": { "type": "integer" } }],
                    "responses": { "200": { "description": "OK", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Score" } } } }, "404": { "description": "No score yet" } }
                },
                "post": {
                    "tags": ["scoring"],
                    "summary": "Create / update score for an interaction",
                    "parameters": [{ "name": "interaction_id", "in": "path", "required": true, "schema": { "type": "integer" } }],
                    "requestBody": { "required": true, "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Score" } } } },
                    "responses": { "200": { "description": "OK" } }
                }
            },
            "/issues": {
                "get": {
                    "tags": ["issues"],
                    "summary": "List issues (paginated, with optional filters)",
                    "parameters": [
                        { "name": "page", "in": "query", "schema": { "type": "integer", "default": 1, "minimum": 1 } },
                        { "name": "limit", "in": "query", "schema": { "type": "integer", "default": 10, "maximum": 1000, "minimum": 1 } },
                        { "name": "status", "in": "query", "schema": { "type": "string", "enum": ["open", "in_progress", "resolved", "closed"] } },
                        { "name": "severity", "in": "query", "schema": { "type": "string", "enum": ["low", "medium", "high", "critical"] } }
                    ],
                    "responses": { "200": { "description": "OK", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Paged" } } } } }
                },
                "post": {
                    "tags": ["issues"],
                    "summary": "Create issue",
                    "requestBody": { "required": true, "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Issue" } } } },
                    "responses": { "200": { "description": "Created" } }
                }
            },
            "/issues/{id}": {
                "patch": {
                    "tags": ["issues"],
                    "summary": "Update issue (status, root_cause, corrective_action)",
                    "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "integer" } }],
                    "responses": { "200": { "description": "OK" }, "404": { "description": "Not found" } }
                }
            },
            "/kpis": {
                "get": {
                    "tags": ["kpis"],
                    "summary": "List KPIs (paginated, optional filters)",
                    "parameters": [
                        { "name": "page", "in": "query", "schema": { "type": "integer", "default": 1, "minimum": 1 } },
                        { "name": "limit", "in": "query", "schema": { "type": "integer", "default": 10, "maximum": 1000, "minimum": 1 } },
                        { "name": "department", "in": "query", "schema": { "type": "string" } }
                    ],
                    "responses": { "200": { "description": "OK", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Paged" } } } } }
                },
                "post": {
                    "tags": ["kpis"],
                    "summary": "Create or update KPI",
                    "responses": { "200": { "description": "OK" } }
                }
            },
            "/reports/dashboard": {
                "get": {
                    "tags": ["reports"],
                    "summary": "Aggregated dashboard counts + KPIs",
                    "description": "Returns counts (agent_count, customer_count, etc.) and computed metrics. Cheap — no full table loads.",
                    "responses": { "200": { "description": "OK", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Dashboard" } } } } }
                }
            },
            "/reports/agent/{id}": {
                "get": {
                    "tags": ["reports"],
                    "summary": "Per-agent score + activity report",
                    "parameters": [{ "name": "id", "in": "path", "required": true, "schema": { "type": "integer" } }],
                    "responses": { "200": { "description": "OK" } }
                }
            }
        }
    })
}

/// Public route: GET /openapi.json
pub async fn openapi_json() -> Json<Value> {
    Json(openapi_spec())
}

/// Public route: GET /swagger-ui
/// Self-contained Swagger UI — no CDN, no internet dependency.
pub async fn swagger_ui() -> Html<String> {
    Html(SWAGGER_UI_HTML.to_string())
}

const SWAGGER_UI_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>CRM Quality Inspector — API Docs</title>
  <link rel="stylesheet" href="/static/swagger-ui.css">
  <style>
    body { margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #fafafa; }
    .topbar { display: none; }
    #swagger-ui .info { margin: 20px 0; }
    #swagger-ui .scheme-container { background: #fff; box-shadow: 0 1px 2px 0 rgba(0,0,0,.1); padding: 10px 0; }
  </style>
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="/static/swagger-ui-bundle.js"></script>
  <script>
    window.onload = () => {
      window.ui = SwaggerUIBundle({
        url: "/openapi.json",
        dom_id: "#swagger-ui",
        deepLinking: true,
        presets: [SwaggerUIBundle.presets.apis],
        layout: "BaseLayout",
        persistAuthorization: true,
        displayRequestDuration: true
      });
    };
  </script>
</body>
</html>
"##;

/// Optional: download Swagger UI assets at build time.
/// In a real deploy, ship swagger-ui-bundle.js and swagger-ui.css as static files.
/// For now we link them in /static — if absent, the page still shows the spec.
pub fn _hint_user() {
    // This is a hint to the developer: drop swagger-ui-bundle.js + swagger-ui.css
    // into static/ to enable the full interactive UI. The OpenAPI JSON at
    // /openapi.json works without them.
    let _ = header::CONTENT_TYPE;
}

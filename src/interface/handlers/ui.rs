use axum::{
    response::{Html, IntoResponse, Redirect},
    extract::{State, Form},
    http::{StatusCode, header},
};
use std::sync::Arc;
use tera::{Context, Tera}; // <--- CORRECCIÓN: AÑADIDO 'Tera' AQUÍ
use serde::Deserialize;
use crate::interface::handlers::admin::AppState;

// Credentials for deployment
const USERNAME: &str = "propileno";
const PASSWORD: &str = "propileno24";
const SESSION_COOKIE: &str = "lamuralla_auth";

#[derive(Deserialize)]
pub struct AuthPayload {
    username: String,
    password: String,
}

pub async fn render_login() -> impl IntoResponse {
    // La instancia de Tera se crea aquí temporalmente para renderizar el login
    // ya que no requiere el estado de la aplicación.
    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => return Html(format!("<h1>Error loading templates: {}</h1>", e)).into_response(),
    };

    match tera.render("login.html", &Context::new()) {
        Ok(html) => Html(html).into_response(),
        Err(err) => Html(format!("<h1>Error rendering template</h1><p>{}</p>", err)).into_response(),
    }
}

pub async fn authenticate(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<AuthPayload>,
) -> impl IntoResponse {
    
    if payload.username == USERNAME && payload.password == PASSWORD {
        // En un entorno de producción, esto debería ser un token JWT o una cookie con sesión segura.
        // Aquí usamos una cookie simple como "sesión" para el ejercicio.
        let cookie_value = format!("{}=valid; Path=/; Max-Age={}; HttpOnly; SameSite=Strict", SESSION_COOKIE, 3600); // 1 hora
        
        let mut response = Redirect::to("/dashboard").into_response();
        response.headers_mut().insert(header::SET_COOKIE, header::HeaderValue::from_str(&cookie_value).unwrap());
        response
    } else {
        // Renderizar página de login con mensaje de error
        let mut ctx = Context::new();
        ctx.insert("error", &true);
        match state.tera.render("login.html", &ctx) {
             Ok(html) => (StatusCode::UNAUTHORIZED, Html(html)).into_response(),
             Err(err) => Html(format!("<h1>Error rendering template</h1><p>{}</p>", err)).into_response(),
        }
    }
}

pub async fn auth_guard(headers: header::HeaderMap) -> Result<(), StatusCode> {
    // Comprueba la existencia de la cookie de autenticación
    let cookie_header = headers.get(header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    
    if cookie_header.contains(&format!("{}={}", SESSION_COOKIE, "valid")) {
        Ok(())
    } else {
        // Si no está autenticado, redirige al login
        Err(StatusCode::UNAUTHORIZED)
    }
}

// Envuelve el render_dashboard original con el guard
pub async fn render_dashboard_guarded(
    headers: header::HeaderMap, 
    State(state): State<Arc<AppState>>
) -> impl IntoResponse {
    // 1. Ejecutar el guard de autenticación
    if let Err(_) = auth_guard(headers).await {
        return Redirect::to("/").into_response();
    }
    
    // 2. Si pasa, renderiza el dashboard
    let _ai_guard = state.ai_service.read().await;
    let mut ctx = Context::new();
    ctx.insert("config", &serde_json::json!({
        "model_name": "gpt-4o",
        "embedding_dim": 1536
    }));

    match state.tera.render("dashboard.html", &ctx) {
        Ok(html) => Html(html).into_response(),
        Err(err) => Html(format!("<h1>Error rendering template</h1><p>{}</p>", err)).into_response(),
    }
}
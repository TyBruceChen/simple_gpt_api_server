use axum::{Json, Router, extract::{State}, response::Html, routing::{get,post}};
use reqwest;
use tokio::fs;
use std::{net::SocketAddr, path::PathBuf, str, sync::Arc, vec};
use serde::{Deserialize, Serialize};
use axum_server::tls_rustls::RustlsConfig;
use tower_http::services::ServeDir;


const HTML_PATH: &str  = "assets/index.html";
const API_KEY_PATH: &str = "credentials/openai_api_key.txt";
const SERVER_MODE: &str = "HTTPS";
const SSL_PRI_PATH: &str = "credentials/ssl/priv.pem";
const SSL_PUB_PATH: &str = "credentials/ssl/cer.pem";

#[derive(Deserialize)]
struct UserQuery{
    model_type: String,
    question: String,
}

#[derive(Serialize)]
struct ServerResponse{
    success: bool,
    answer: String,
}

#[derive(Serialize)]
struct OpenAIRequest{ //with temperature configuration
    model: String,
    messages: Vec<OpenAIMessage>,   // For upload format, it's messages (with 's')
    temperature: Option<f32>,
}

#[derive(Serialize, Deserialize)]
struct OpenAIMessage{
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse{
    choices: Vec<Choice>
}

#[derive(Deserialize)]
struct Choice {
    message: OpenAIMessage // For the receive format, it's message (no 's')
}

struct Appstates{
    client: reqwest::Client,
    openai_key: String,
}

#[tokio::main] // This macro sets up the multi-threaded Async engine
async fn main() {
    let api_key = fs::read_to_string(API_KEY_PATH).await.unwrap();
    let request_state = Arc::new(Appstates{
        client: reqwest::Client::new(),
        openai_key: api_key
    });
    let static_files_services = ServeDir::new("assets");

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/submit", post(handle_submit))
        .with_state(request_state)
        .nest_service("/assets", static_files_services);


    let addr = SocketAddr::from(([127, 0, 0, 1], 5000));
    println!("🚀 Server running at https://{}", addr);

    if SERVER_MODE == "HTTP"{
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }else {
        let config = RustlsConfig::from_pem_file(
            PathBuf::from(SSL_PUB_PATH),
            PathBuf::from(SSL_PRI_PATH)).await.unwrap();
        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service())
            .await.unwrap();
    }
}


async fn serve_index(State(_rs_state): State<Arc<Appstates>>) -> Html<String> {
    match fs::read_to_string(HTML_PATH).await {
        Ok(content) => Html(content),
        Err(e) => Html(format!("<h1>500 Internal Server Error: {}</h1>", e).to_string()),
    }
}

async fn handle_submit(State(rs_state):State<Arc<Appstates>>, 
    Json(params): Json<UserQuery>) -> Json<ServerResponse>{
    let openai_req;
    if params.model_type == "gpt-4o-mini"{
        openai_req = OpenAIRequest{
            model: params.model_type,
            messages: vec![OpenAIMessage{
                role: "user".to_string(),
                content: params.question, 
            }],
            temperature: Some(0.0),
        };
    } else {
        openai_req = OpenAIRequest{
            model: params.model_type,
            messages: vec![OpenAIMessage{
                role: "user".to_string(),
                content: params.question, 
            }],
            temperature: None,
    }
    }
     
    let rs = rs_state.client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&rs_state.openai_key)
            .json(&openai_req)
            .send().await;
    
    // Test script:
    //let body_text = rs.unwrap().text().await.unwrap();
    //println!("{}", body_text);
    //Json(ServerResponse{success: false, answer: "Test Mode".to_string()})

    let response: ServerResponse = match rs {
        Ok(raw_response) => {
            let openai_data: OpenAIResponse = raw_response.json().await.unwrap();
            let ai_answer = openai_data.choices[0].message.content.clone(); // Why clone?
            ServerResponse{answer: ai_answer, success: true}
        },
        Err(e) => ServerResponse{answer: format!("Connection Error {}", e), success: false}
    };
    Json(response)
}
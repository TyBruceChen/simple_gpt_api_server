use axum::{
    Form, Json, Router, body, extract::Query, http::{Response, response}, response::Html, routing::{get,post}
};
use tokio::fs;
use std::{fmt::format, net::SocketAddr, str, sync::Arc, vec};
use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Debug)]
struct UserInput{
    input: String,
}

const HTML_PATH: &str  = "static/index.html";
const API_KEY_PATH: &str = "static/openai_api_key.txt";

#[derive(Deserialize)]
struct UserQuery{
    question: String,
}

#[derive(Serialize)]
struct ServerResponse{
    answer: String,
}

#[derive(Serialize)]
struct OpenAIRequest{
    model: String,
    messages: Vec<OpenAIMessage>,   // For upload format, it's messages (with 's')
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

#[tokio::main] // This macro sets up the multi-threaded Async engine
async fn main() {
    //let shared_api_key = Arc::new(api_key); 
    let api_key = fs::read_to_string(API_KEY_PATH).await.unwrap();
    print!("API: {}", api_key);

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/submit", post(handle_submit));


    let addr = SocketAddr::from(([127, 0, 0, 1], 5000));
    println!("🚀 Server running at http://{}", addr);

    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


async fn serve_index() -> Html<String> {
    // NOTE: This String is stored on the HEAP (because it's dynamic data).
    match fs::read_to_string(HTML_PATH).await {
        Ok(content) => Html(content),
        Err(e) => Html(format!("<h1>500 Internal Server Error: {}</h1>", e).to_string()),
    }
}

const PREFIX: &str = r#"<div id="response" style="display: none;">LLM_RESPONSE"#;
const PREFIX_: &str = r#"<div id="response" style="display: block;">"#;

async fn handle_submit(Form(params): Form<UserInput>) -> Html<String>{
    // API read
    let api_key = fs::read_to_string(API_KEY_PATH).await.unwrap();
    let content = fs::read_to_string(HTML_PATH).await.unwrap();
    let client = reqwest::Client::new();
    let openai_req = OpenAIRequest{
        model: "gpt-4o-mini".to_string(),
        messages: vec![OpenAIMessage{
            role: "user".to_string(),
            content: params.input,
        }],
    };
    let rs = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(api_key)
            .json(&openai_req)
            .send().await;
    
    // Test script:
    //let body_text = rs.unwrap().text().await.unwrap();
    //println!("{}", body_text);
    //Html("No".to_string())

    let response: ServerResponse = match rs {
        Ok(response) => {
            let openai_data: OpenAIResponse = response.json().await.unwrap();
            let ai_answer = openai_data.choices[0].message.content.clone(); // Why clone?
            ServerResponse{answer: ai_answer}
        },
        Err(_) => ServerResponse{answer: "Connection Error".to_string()}
    };
    //let response: String = format!("Receive: {}", params.input);
    let response = content.replace(PREFIX, format!("{}{}", PREFIX_, &response.answer).as_str());
    Html(response)
}
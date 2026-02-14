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
    model_type: String,
    question: String,
}

#[derive(Serialize)]
struct ServerResponse{
    success: bool,
    answer: String,
}

#[derive(Serialize)]
struct OpenAIRequest_T{ //with temperature configuration
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

#[tokio::main] // This macro sets up the multi-threaded Async engine
async fn main() {
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

async fn handle_submit(Json(params): Json<UserQuery>) -> Json<ServerResponse>{
    // API read
    let api_key = fs::read_to_string(API_KEY_PATH).await.unwrap();
    //let content = fs::read_to_string(HTML_PATH).await.unwrap();
    let client = reqwest::Client::new();
    let openai_req;
    if params.model_type == "gpt-4o-mini"{
        openai_req = OpenAIRequest_T{
            model: params.model_type,
            messages: vec![OpenAIMessage{
                role: "user".to_string(),
                content: params.question, 
            }],
            temperature: Some(0.0),
        };
    } else {
        openai_req = OpenAIRequest_T{
            model: params.model_type,
            messages: vec![OpenAIMessage{
                role: "user".to_string(),
                content: params.question, 
            }],
            temperature: None,
    }
    }
     
    let rs = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(api_key)
            .json(&openai_req)
            .send().await;
    
    // Test script:
    //let body_text = rs.unwrap().text().await.unwrap();
    //println!("{}", body_text);
    //Json(ServerResponse{success: false, answer: "Test Mode".to_string()})

    let response: ServerResponse = match rs {
        Ok(response) => {
            let openai_data: OpenAIResponse = response.json().await.unwrap();
            let ai_answer = openai_data.choices[0].message.content.clone(); // Why clone?
            ServerResponse{answer: ai_answer, success: true}
        },
        Err(e) => ServerResponse{answer: format!("Connection Error {}", e), success: false}
    };
    Json(response)
}
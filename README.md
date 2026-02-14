# Introduction:
A RUST built LLM server for one-time question answering by OpenAI-API

# Config:
All reside in the beginning of `src/main.rs` file.

# Design:
```
Request: HTML form component -> JS transform into JSON request -> Server -> Response
Response: Response JSON{"answer":..., "sucess":...} -> JS handler -> fill html <pre> text content
```
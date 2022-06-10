use crate::context::Context;
use lsp_types::CompletionParams;
use lsp_server::Request;

pub fn on_completion_request(context: &Context, request: &Request) {
    eprintln!("handling completion request");
    let parameters = serde_json::from_value::<CompletionParams>(request.params.clone())
        .expect("could not deserialize completion request");

    let path = parameters.text_document_position.text_document.uri.path();
    let buffer = context.files.get(path);
    if buffer.is_none() {
        eprintln!("Could not read '{}' when handling completion request", path);
    }
}

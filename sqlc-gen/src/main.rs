use prost::Message;
use quote::ToTokens;
use sqlc_sqlc_community_neoeinstein_prost::plugin;
use std::io;
use std::io::prelude::*;

pub mod codegen;

pub fn deserialize_codegen_request(
    buf: &[u8],
) -> Result<plugin::GenerateRequest, prost::DecodeError> {
    plugin::GenerateRequest::decode(buf)
}

pub fn serialize_codegen_response(resp: &plugin::GenerateResponse) -> Vec<u8> {
    resp.encode_to_vec()
}

pub fn create_codegen_response(content: &str) -> plugin::GenerateResponse {
    let mut file = plugin::File::default();
    file.name = "gen.rs".to_string();
    file.contents = content.as_bytes().to_vec().into();

    let mut resp = plugin::GenerateResponse::default();
    resp.files.push(file);
    resp
}

pub fn generate_rust_code(req: plugin::GenerateRequest) -> String {
    let code_partials: codegen::CodePartials = req.into();
    let tokens = code_partials.to_token_stream();
    let syntax_tree = syn::parse_file(tokens.to_string().as_str()).unwrap();
    return prettyplease::unparse(&syntax_tree);
    // return tokens.to_string();
}

fn main() -> Result<(), prost::DecodeError> {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut buffer: Vec<u8> = Vec::new();
    stdin.read_to_end(&mut buffer).unwrap();

    let req = deserialize_codegen_request(&buffer)?;

    let out = generate_rust_code(req);

    let resp = create_codegen_response(&out);
    let out = serialize_codegen_response(&resp);

    let _result = io::stdout()
        .write_all(&out)
        .expect("failed to write generated code");

    Ok(())
}

use codegen::plugin;
use prost::Message;
use std::io;
use std::io::prelude::*;
use std::io::Cursor;

pub mod codegen;

pub fn deserialize_codegen_request(
    buf: &[u8],
) -> Result<plugin::CodeGenRequest, prost::DecodeError> {
    plugin::CodeGenRequest::decode(&mut Cursor::new(buf))
}

pub fn serialize_codegen_response(resp: &plugin::CodeGenResponse) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.reserve(resp.encoded_len());

    resp.encode(&mut buf).unwrap();
    buf
}

pub fn create_codegen_response(content: &str) -> plugin::CodeGenResponse {
    let mut file = plugin::File::default();
    file.name = "gen.rs".to_string();
    file.contents = content.as_bytes().to_vec();

    let mut resp = plugin::CodeGenResponse::default();
    resp.files.push(file);
    resp
}

pub fn generate_rust_code(req: plugin::CodeGenRequest) -> String {
    let tokens = codegen::CodeBuilder::new(req).generate_code();
    let syntax_tree = syn::parse_file(tokens.to_string().as_str()).unwrap();
    return prettyplease::unparse(&syntax_tree);
    // return tokens.to_string();
}

fn main() -> Result<(), prost::DecodeError> {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let buffer = stdin.fill_buf().unwrap();

    let req = match deserialize_codegen_request(&buffer) {
        Ok(request_deserialized_result) => request_deserialized_result,
        Err(_e) => std::process::exit(1),
    };

    let out = generate_rust_code(req);

    let resp = create_codegen_response(&out);
    let out = serialize_codegen_response(&resp);

    let _ = match io::stdout().write_all(&out) {
        Ok(result) => result,
        Err(_e) => std::process::exit(1),
    };

    Ok(())
}

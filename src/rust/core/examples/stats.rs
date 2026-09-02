//! Prints the OpenAPI statistics for a specification, for parity checking.

fn main() {
    let path = std::env::args().nth(1).expect("usage: stats <spec>");
    let document = curlgenerator_core::openapi::load_document(&path).expect("load");
    let stats = curlgenerator_core::openapi::inspect(&document);

    println!(
        "{} {} {} {} {} {} {} {}",
        stats.path_item_count,
        stats.operation_count,
        stats.parameter_count,
        stats.request_body_count,
        stats.response_count,
        stats.link_count,
        stats.callback_count,
        stats.schema_count
    );
}

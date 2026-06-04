use openapiv3::{OpenAPI, Schema, SchemaKind};

#[derive(Debug, Default)]
pub struct OpenApiStats {
    pub path_item_count: usize,
    pub operation_count: usize,
    pub parameter_count: usize,
    pub request_body_count: usize,
    pub response_count: usize,
    pub link_count: usize,
    pub callback_count: usize,
    pub schema_count: usize,
}

pub fn compute_stats(doc: &OpenAPI) -> OpenApiStats {
    let mut stats = OpenApiStats::default();

    stats.path_item_count = doc.paths.paths.len();

    for path_item_or_ref in doc.paths.paths.values() {
        if let openapiv3::ReferenceOr::Item(ref path_item) = path_item_or_ref {
            let ops = [
                path_item.get.as_ref(),
                path_item.put.as_ref(),
                path_item.post.as_ref(),
                path_item.delete.as_ref(),
                path_item.options.as_ref(),
                path_item.head.as_ref(),
                path_item.patch.as_ref(),
                path_item.trace.as_ref(),
            ];

            for op in ops.into_iter().flatten() {
                stats.operation_count += 1;
                collect_from_operation(op, &mut stats);
            }
        }
    }

    if let Some(ref components) = doc.components {
        stats.schema_count += components.schemas.len();
        for schema in components.schemas.values() {
            if let openapiv3::ReferenceOr::Item(ref s) = schema {
                count_nested_schemas(s, &mut stats);
            }
        }
    }

    stats
}

fn collect_from_operation(
    operation: &openapiv3::Operation,
    stats: &mut OpenApiStats,
) {
    stats.parameter_count += operation.parameters.len();

    if let Some(openapiv3::ReferenceOr::Item(ref request_body)) = operation.request_body {
        stats.request_body_count += 1;
        for media_type in request_body.content.values() {
            if let Some(openapiv3::ReferenceOr::Item(ref schema)) = &media_type.schema {
                count_nested_schemas(schema, stats);
            }
        }
    }

    stats.response_count += operation.responses.responses.len();
    for response in operation.responses.responses.values() {
        if let openapiv3::ReferenceOr::Item(ref resp) = response {
            for media_type in resp.content.values() {
                if let Some(openapiv3::ReferenceOr::Item(ref schema)) = &media_type.schema {
                    count_nested_schemas(schema, stats);
                }
            }
        }
    }
}

fn count_nested_schemas(schema: &Schema, stats: &mut OpenApiStats) {
    match &schema.schema_kind {
        SchemaKind::AllOf { all_of, .. } => {
            for s in all_of {
                if let openapiv3::ReferenceOr::Item(ref item) = s {
                    count_nested_schemas(item, stats);
                }
            }
        }
        SchemaKind::AnyOf { any_of, .. } => {
            for s in any_of {
                if let openapiv3::ReferenceOr::Item(ref item) = s {
                    count_nested_schemas(item, stats);
                }
            }
        }
        SchemaKind::OneOf { one_of, .. } => {
            for s in one_of {
                if let openapiv3::ReferenceOr::Item(ref item) = s {
                    count_nested_schemas(item, stats);
                }
            }
        }
        SchemaKind::Not { not: not_box } => {
            if let openapiv3::ReferenceOr::Item(ref item) = **not_box {
                count_nested_schemas(item, stats);
            }
        }
        SchemaKind::Type(openapiv3::Type::Object(_)) => {
            stats.schema_count += 1;
        }
        _ => {}
    }
}

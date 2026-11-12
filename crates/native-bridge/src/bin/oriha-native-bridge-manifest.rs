//! Native bridge schema manifest emitted as stable JSON.
#[path = "manifest/enums.rs"]
mod enum_values;
#[path = "manifest/functions.rs"]
mod function_specs;
#[path = "manifest/records.rs"]
mod record_specs;
use enum_values::*;
use function_specs::functions;
use record_specs::records;

use oriha_i18n::SUPPORTED_LOCALES;
use serde::Serialize;
use std::io;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BridgeManifest {
    schema_version: u32,
    enums: Vec<EnumSpec>,
    string_unions: Vec<StringUnionSpec>,
    records: Vec<RecordSpec>,
    functions: Vec<FunctionSpec>,
    catalogs: Vec<CatalogSpec>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnumSpec {
    name: &'static str,
    variants: Vec<EnumVariant>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnumVariant {
    name: &'static str,
    value: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StringUnionSpec {
    name: &'static str,
    values: Vec<&'static str>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RecordSpec {
    name: &'static str,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    generic_params: Vec<&'static str>,
    fields: Vec<FieldSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    index_signature: Option<IndexSignatureSpec>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct IndexSignatureSpec {
    key_name: &'static str,
    key_type: &'static str,
    value_type: TypeRef,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FieldSpec {
    name: &'static str,
    type_ref: TypeRef,
    optional: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TypeRef {
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    item: Option<Box<TypeRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    items: Option<Vec<TypeRef>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FunctionSpec {
    name: &'static str,
    params: Vec<FunctionParamSpec>,
    returns: TypeRef,
    #[serde(default, skip_serializing_if = "is_false")]
    async_result: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FunctionParamSpec {
    name: &'static str,
    type_ref: TypeRef,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CatalogSpec {
    name: &'static str,
    option_type: &'static str,
    enum_name: &'static str,
    entries: Vec<OptionEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OptionEntry {
    enum_name: &'static str,
    variant_name: &'static str,
    label: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    family: Option<&'static str>,
    #[serde(default, skip_serializing_if = "is_false")]
    recommended: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    font_family: Option<FontSpec>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FontSpec {
    family: &'static str,
    fallbacks: Vec<&'static str>,
}

fn main() -> serde_json::Result<()> {
    serde_json::to_writer_pretty(io::stdout().lock(), &bridge_manifest())?;
    println!();
    Ok(())
}

fn bridge_manifest() -> BridgeManifest {
    BridgeManifest {
        schema_version: 1,
        enums: vec![
            enum_spec("RenderMode", RENDER_MODE),
            enum_spec("AppearanceStyle", APPEARANCE_STYLE),
            enum_spec("TableBorderStyle", TABLE_BORDER_STYLE),
            enum_spec("RenderDiagnosticSeverity", RENDER_DIAGNOSTIC_SEVERITY),
            enum_spec("RenderDiagnosticEngine", RENDER_DIAGNOSTIC_ENGINE),
            enum_spec("RenderAssetKind", RENDER_ASSET_KIND),
            enum_spec("RenderPatchStrategy", RENDER_PATCH_STRATEGY),
            enum_spec("RenderPatchFallbackReason", RENDER_PATCH_FALLBACK_REASON),
            enum_spec("BridgeFeature", BRIDGE_FEATURE),
            enum_spec(
                "UsageAnalyticsOutboxCommandKind",
                USAGE_ANALYTICS_OUTBOX_COMMAND_KIND,
            ),
            enum_spec("Plan", PLAN),
            enum_spec("ColorScheme", COLOR_SCHEME),
            enum_spec("SyntaxTheme", SYNTAX_THEME),
            enum_spec("HeadingFont", HEADING_FONT),
            enum_spec("BodyFont", BODY_FONT),
            enum_spec("CodeFont", CODE_FONT),
        ],
        string_unions: vec![
            string_union_spec(
                "OrihaBuiltinLocale",
                SUPPORTED_LOCALES
                    .iter()
                    .map(|locale| locale.code())
                    .collect(),
            ),
            string_union_spec(
                "RenderPatchOperationKind",
                vec!["replaceAll", "replaceBlocks"],
            ),
        ],
        records: records(),
        functions: functions(),
        catalogs: Vec::new(),
    }
}

fn enum_spec(name: &'static str, variants: &[(&'static str, &'static str)]) -> EnumSpec {
    EnumSpec {
        name,
        variants: variants
            .iter()
            .map(|(name, value)| EnumVariant { name, value })
            .collect(),
    }
}

fn string_union_spec(name: &'static str, values: Vec<&'static str>) -> StringUnionSpec {
    StringUnionSpec { name, values }
}

fn record(
    name: &'static str,
    generic_params: &[&'static str],
    fields: Vec<FieldSpec>,
) -> RecordSpec {
    RecordSpec {
        name,
        generic_params: generic_params.to_vec(),
        fields,
        index_signature: None,
    }
}

fn record_with_index_signature(
    name: &'static str,
    generic_params: &[&'static str],
    fields: Vec<FieldSpec>,
    index_signature: IndexSignatureSpec,
) -> RecordSpec {
    RecordSpec {
        name,
        generic_params: generic_params.to_vec(),
        fields,
        index_signature: Some(index_signature),
    }
}

fn field(name: &'static str, type_ref: TypeRef, optional: bool) -> FieldSpec {
    FieldSpec {
        name,
        type_ref,
        optional,
    }
}

fn function(
    name: &'static str,
    params: Vec<FunctionParamSpec>,
    returns: TypeRef,
    async_result: bool,
) -> FunctionSpec {
    FunctionSpec {
        name,
        params,
        returns,
        async_result,
    }
}

fn param(name: &'static str, type_ref: TypeRef) -> FunctionParamSpec {
    FunctionParamSpec { name, type_ref }
}

fn primitive(name: &'static str) -> TypeRef {
    TypeRef {
        kind: "primitive",
        name: Some(name),
        item: None,
        items: None,
    }
}

fn named(name: &'static str) -> TypeRef {
    TypeRef {
        kind: "named",
        name: Some(name),
        item: None,
        items: None,
    }
}

fn nullable(item: TypeRef) -> TypeRef {
    TypeRef {
        kind: "nullable",
        name: None,
        item: Some(Box::new(item)),
        items: None,
    }
}

fn array(item: TypeRef) -> TypeRef {
    TypeRef {
        kind: "array",
        name: None,
        item: Some(Box::new(item)),
        items: None,
    }
}

fn map(item: TypeRef) -> TypeRef {
    TypeRef {
        kind: "map",
        name: Some("string"),
        item: Some(Box::new(item)),
        items: None,
    }
}

fn union(items: Vec<TypeRef>) -> TypeRef {
    TypeRef {
        kind: "union",
        name: None,
        item: None,
        items: Some(items),
    }
}

fn undefined() -> TypeRef {
    TypeRef {
        kind: "undefined",
        name: None,
        item: None,
        items: None,
    }
}

fn index_signature(
    key_name: &'static str,
    key_type: &'static str,
    value_type: TypeRef,
) -> IndexSignatureSpec {
    IndexSignatureSpec {
        key_name,
        key_type,
        value_type,
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

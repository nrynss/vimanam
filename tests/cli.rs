use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;

const OAS3: &str = "tests/fixtures/petstore_oas3.json";
const OAS2: &str = "tests/fixtures/petstore_oas2.json";
const OAS3_SCHEMA_REFS: &str = "tests/fixtures/schema_refs_oas3.json";
const OAS3_MULTI_AUTH: &str = "tests/fixtures/multi_auth_oas3.json";
const OAS2_MULTI_AUTH: &str = "tests/fixtures/multi_auth_oas2.json";
const OAS3_EXAMPLES: &str = "tests/fixtures/examples_oas3.json";
const OAS3_REF_BODY: &str = "tests/fixtures/ref_request_body_oas3.json";

// YAML twin of petstore_oas3.json — must parse to identical documentation (#4).
const OAS3_YAML: &str = "tests/fixtures/petstore_oas3.yaml";
const OAS2_YAML: &str = "tests/fixtures/petstore_oas2.yaml";
const OAS3_SCHEMA_REFS_YAML: &str = "tests/fixtures/schema_refs_oas3.yaml";

// Parse-layer correctness cluster (issues #48, #50, #51, #54, #56, #60).
const MULTI_TAG: &str = "tests/fixtures/multi_tag_oas3.json";
const OAS2_HTTP_SCHEME: &str = "tests/fixtures/http_scheme_oas2.json";
const REF_PARAMETER: &str = "tests/fixtures/ref_parameter.json";
const REF_PATH_ITEM: &str = "tests/fixtures/ref_path_item.json";
const TYPE_ARRAY: &str = "tests/fixtures/type_array_nullable.json";
const MISSING_RESPONSES: &str = "tests/fixtures/missing_responses.json";
const OVERRIDE_PARAM: &str = "tests/fixtures/override_param.json";
const UNKNOWN_TAG: &str = "tests/fixtures/unknown_tag.json";

// --stats alignment with a service name wider than the SERVICE header (#42).
const STATS_LONG_SERVICE: &str = "tests/fixtures/stats_long_service_oas3.json";

fn vimanam() -> Command {
    Command::cargo_bin("vimanam").unwrap()
}

#[test]
fn version_flag_reports_crate_version() {
    vimanam()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn summary_lists_services_and_operations() {
    vimanam()
        .arg(OAS3)
        .assert()
        .success()
        .stdout(predicate::str::contains("# Petstore API"))
        .stdout(predicate::str::contains("- Pets"))
        .stdout(predicate::str::contains("- Store"))
        // Service prefix is stripped from operation IDs in the summary view
        .stdout(predicate::str::contains("* ListPets"));
}

#[test]
fn basic_detail_writes_endpoint_sections() {
    vimanam()
        .arg(OAS3)
        .args(["--detail", "basic"])
        .assert()
        .success()
        .stdout(predicate::str::contains("### Pets_ListPets"))
        .stdout(predicate::str::contains("**Operation:** GET /pets"))
        .stdout(predicate::str::contains("**Operation:** POST /pets"));
}

// Regression test: optional request bodies (no `required: true`) used to be
// dropped from the parameter table entirely.
#[test]
fn optional_request_body_is_documented() {
    vimanam()
        .arg(OAS3)
        .args(["--detail", "standard"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "| `requestBody` | body | No | Pet to add |",
        ));
}

// `--required-only` drops parameters that are not required (explicit
// `required: false` or unspecified), keeping required ones.
#[test]
fn required_only_excludes_non_required_parameters() {
    vimanam()
        .arg(OAS3)
        .args(["--detail", "standard", "--required-only"])
        .assert()
        .success()
        // Required path parameter is kept.
        .stdout(predicate::str::contains("| `petId` | path | Yes |"))
        // Optional query parameter is dropped.
        .stdout(predicate::str::contains("| `limit` |").not());
}

#[test]
fn required_path_param_is_documented() {
    vimanam()
        .arg(OAS3)
        .args(["--detail", "standard"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "| `petId` | path | Yes | ID of the pet |",
        ));
}

#[test]
fn exclude_deprecated_hides_endpoint() {
    vimanam()
        .arg(OAS3)
        .args(["--detail", "basic"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Store_ListOrders"));

    vimanam()
        .arg(OAS3)
        .args(["--detail", "basic", "--exclude-deprecated"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Store_ListOrders").not());
}

#[test]
fn method_filter_excludes_other_methods() {
    vimanam()
        .arg(OAS3)
        .args(["--detail", "basic", "--method-filter", "GET"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pets_ListPets"))
        .stdout(predicate::str::contains("Pets_CreatePet").not());
}

// Regression test for #13: methods are stored uppercase, so a lowercase
// `--method-filter` value used to match nothing and silently empty the output.
#[test]
fn method_filter_is_case_insensitive() {
    vimanam()
        .arg(OAS3)
        .args(["--detail", "basic", "--method-filter", "get"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pets_ListPets"))
        .stdout(predicate::str::contains("Pets_CreatePet").not());
}

// Regression test for #19: a case-mismatched `--service-filter` used to
// silently omit all endpoints.
#[test]
fn service_filter_is_case_insensitive() {
    vimanam()
        .arg(OAS3)
        .args(["--detail", "basic", "--service-filter", "pets"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pets_ListPets"))
        .stdout(predicate::str::contains("Store_ListOrders").not());
}

#[test]
fn path_filter_excludes_other_paths() {
    vimanam()
        .arg(OAS3)
        .args(["--detail", "basic", "--path-filter", "/store"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Store_ListOrders"))
        .stdout(predicate::str::contains("Pets_ListPets").not());
}

#[test]
fn include_auth_shows_servers_and_schemes() {
    vimanam()
        .arg(OAS3)
        .arg("--include-auth")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "https://api.petstore.example.com/v1",
        ))
        .stdout(predicate::str::contains("apiKeyAuth"));
}

#[test]
fn flat_grouping_lists_all_endpoints() {
    vimanam()
        .arg(OAS3)
        .args(["--detail", "basic", "--flat"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Endpoints"))
        .stdout(predicate::str::contains("### Pets_ListPets"))
        .stdout(predicate::str::contains("### Store_ListOrders"));
}

#[test]
fn oas2_spec_is_supported() {
    vimanam()
        .arg(OAS2)
        .args(["--detail", "standard", "--include-auth"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Petstore Legacy API"))
        // host + basePath are combined into a server URL
        .stdout(predicate::str::contains(
            "https://legacy.petstore.example.com/v2",
        ))
        .stdout(predicate::str::contains("Pets_CreatePet"))
        // OpenAPI 2.0 body responses infer application/json
        .stdout(predicate::str::contains(
            "| 200 | application/json | Created |",
        ));
}

// The OAS2 `schemes` field names the transfer protocol; a plain-HTTP spec must
// not be rendered with an assumed `https://` prefix. (Specs without `schemes`
// still default to https — covered by `oas2_spec_is_supported` above.)
#[test]
fn oas2_http_scheme_is_respected() {
    vimanam()
        .arg(OAS2_HTTP_SCHEME)
        .arg("--include-auth")
        .assert()
        .success()
        .stdout(predicate::str::contains("http://internal.example.com/v1"))
        .stdout(predicate::str::contains("https://internal.example.com").not());
}

#[test]
fn output_flag_writes_file() {
    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("out.md");

    vimanam()
        .arg(OAS3)
        .args(["-o", out_path.to_str().unwrap()])
        .assert()
        .success();

    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("# Petstore API"));
}

// --- YAML input support (#4) ---

// A YAML OpenAPI 3 spec parses just like its JSON counterpart.
#[test]
fn yaml_spec_is_parsed() {
    vimanam()
        .arg(OAS3_YAML)
        .args(["--detail", "basic"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# Petstore API"))
        .stdout(predicate::str::contains("### Pets_ListPets"))
        .stdout(predicate::str::contains("**Operation:** GET /pets"));
}

// The YAML twin and the JSON fixture must produce byte-identical documentation:
// format is an input detail, not a semantic one. Also guards key-order determinism
// (IndexMap) across the YAML deserializer.
#[test]
fn yaml_and_json_produce_identical_output() {
    let render = |spec: &str| {
        vimanam()
            .arg(spec)
            .args(["--detail", "full", "--include-schemas", "--include-auth"])
            .output()
            .unwrap()
            .stdout
    };
    assert_eq!(
        render(OAS3),
        render(OAS3_YAML),
        "YAML and JSON inputs produced different output"
    );
}

// The Swagger 2.0 (OAS2) YAML twin and its JSON counterpart must produce byte-identical documentation.
#[test]
fn yaml_and_json_produce_identical_output_oas2() {
    let render = |spec: &str| {
        vimanam()
            .arg(spec)
            .args(["--detail", "full", "--include-schemas", "--include-auth"])
            .output()
            .unwrap()
            .stdout
    };
    assert_eq!(
        render(OAS2),
        render(OAS2_YAML),
        "OAS2 YAML and JSON inputs produced different output"
    );
}

// The $ref-heavy YAML twin of schema_refs_oas3.json must produce identical output.
#[test]
fn yaml_and_json_produce_identical_output_schema_refs() {
    let render = |spec: &str| {
        vimanam()
            .arg(spec)
            .args(["--detail", "full", "--include-schemas"])
            .output()
            .unwrap()
            .stdout
    };
    assert_eq!(
        render(OAS3_SCHEMA_REFS),
        render(OAS3_SCHEMA_REFS_YAML),
        "schema_refs YAML and JSON inputs produced different output"
    );
}

// Extension detection is case-insensitive (`.YAML` routes to the YAML parser).
#[test]
fn yaml_extension_is_case_insensitive() {
    let yaml = std::fs::read_to_string(OAS3_YAML).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("spec.YAML");
    std::fs::write(&path, yaml).unwrap();

    vimanam()
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("# Petstore API"));
}

// A YAML spec with a non-YAML/JSON extension still parses: the JSON-first path
// falls back to the YAML parser.
#[test]
fn yaml_content_with_unknown_extension_falls_back() {
    let yaml = std::fs::read_to_string(OAS3_YAML).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("spec.txt");
    std::fs::write(&path, yaml).unwrap();

    vimanam()
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("# Petstore API"));
}

// Malformed YAML fails with an error rather than panicking.
#[test]
fn invalid_yaml_fails() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("broken.yaml");
    std::fs::write(&path, "openapi: \"3.0.0\"\n  bad: : indentation:").unwrap();

    vimanam()
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error:"));
}

// A structurally-valid YAML document that isn't an OpenAPI spec reports the
// targeted missing-field error (and only that — no doubled fallback noise).
#[test]
fn yaml_without_openapi_fields_fails() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notspec.yaml");
    std::fs::write(&path, "hello: world\n").unwrap();

    vimanam()
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Missing 'swagger' or 'openapi' field",
        ));
}

#[test]
fn invalid_json_fails() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    write!(file, "this is not json").unwrap();

    vimanam()
        .arg(file.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error:"));
}

#[test]
fn json_without_openapi_fields_fails() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    write!(file, "{{\"hello\": \"world\"}}").unwrap();

    vimanam()
        .arg(file.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error:"));
}

// Output must be byte-identical across runs, even with sorting disabled.
// Guards the IndexMap-based ordering of paths, responses, and content types.
#[test]
fn output_is_deterministic() {
    let run = || {
        vimanam()
            .arg(OAS3)
            .args([
                "--detail",
                "full",
                "--include-schemas",
                "--include-auth",
                "--sort",
                "none",
            ])
            .output()
            .unwrap()
            .stdout
    };

    let first = run();
    for _ in 0..4 {
        assert_eq!(first, run(), "output differed between identical runs");
    }
}

// By default (#58) component schemas are linked from their use site and expanded
// once in a trailing "Schema Definitions" section, rather than re-inlined.
#[test]
fn full_detail_links_schema_refs_to_definitions() {
    vimanam()
        .arg(OAS3_SCHEMA_REFS)
        .args(["--detail", "full", "--include-schemas"])
        .assert()
        .success()
        // The use site is a single linked row, not a re-inlined subtree.
        .stdout(predicate::str::contains(
            "| `request` | [CreatePetRequest](#schema-createpetrequest) | - | - |",
        ))
        .stdout(predicate::str::contains(
            "| `response` | [Pet](#schema-pet) | - | - |",
        ))
        // The shared schemas are expanded once in the definitions section.
        .stdout(predicate::str::contains("## Schema Definitions"))
        .stdout(predicate::str::contains(
            "### CreatePetRequest {#schema-createpetrequest}",
        ))
        .stdout(predicate::str::contains(
            "| `CreatePetRequest.name` | string | Yes | Pet name |",
        ))
        .stdout(predicate::str::contains(
            "| `CreatePetRequest.category` | [Category](#schema-category) | Yes |",
        ))
        .stdout(predicate::str::contains(
            "| `Category.id` | string | Yes | Category identifier |",
        ))
        .stdout(predicate::str::contains(
            "| `Pet.allOf[1].id` | string | Yes | Pet identifier |",
        ));
}

// CreatePetRequest is referenced from the /pets request body and again from
// Pet's `allOf`; it must be expanded exactly once (the #58 win).
#[test]
fn shared_schema_is_expanded_once() {
    let output = String::from_utf8(
        vimanam()
            .arg(OAS3_SCHEMA_REFS)
            .args(["--detail", "full", "--include-schemas"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let definitions = output
        .matches("### CreatePetRequest {#schema-createpetrequest}")
        .count();
    assert_eq!(
        definitions, 1,
        "CreatePetRequest expanded {definitions} times"
    );
}

// A self-referential schema (Node.next -> Node) renders once and links back to
// itself instead of looping or printing a "cycle detected" row.
#[test]
fn linked_mode_handles_self_reference_with_a_link() {
    vimanam()
        .arg(OAS3_SCHEMA_REFS)
        .args(["--detail", "full", "--include-schemas"])
        .assert()
        .success()
        .stdout(predicate::str::contains("### Node {#schema-node}"))
        .stdout(predicate::str::contains(
            "| `Node.next` | [Node](#schema-node) | No |",
        ))
        .stdout(predicate::str::contains("Cycle detected").not());
}

// `--inline-schemas` restores the fully self-contained output: every `$ref` is
// expanded inline at each use site, with no shared definitions section.
#[test]
fn inline_schemas_expands_refs_at_each_use_site() {
    vimanam()
        .arg(OAS3_SCHEMA_REFS)
        .args(["--detail", "full", "--include-schemas", "--inline-schemas"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "| `request.name` | string | Yes | Pet name |",
        ))
        .stdout(predicate::str::contains(
            "| `request.category.id` | string | Yes | Category identifier |",
        ))
        .stdout(predicate::str::contains(
            "| `response.allOf[1].id` | string | Yes | Pet identifier |",
        ))
        .stdout(predicate::str::contains("request.variant.oneOf[0]"))
        .stdout(predicate::str::contains("## Schema Definitions").not());
}

// #69 follow-up: the "no effect" warning reports the current detail level in the
// same lowercase spelling the user types (`standard`), not the Debug-derived
// `Standard`.
#[test]
fn include_schemas_warning_uses_lowercase_detail_name() {
    vimanam()
        .arg(OAS3)
        .args(["--detail", "standard", "--include-schemas"])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "--include-schemas has no effect at --detail standard; use --detail full.",
        ));
}

// At `--detail full` the flag takes effect, so no warning is emitted.
#[test]
fn include_schemas_at_full_detail_emits_no_warning() {
    vimanam()
        .arg(OAS3)
        .args(["--detail", "full", "--include-schemas"])
        .assert()
        .success()
        .stderr(predicate::str::contains("no effect").not());
}

// `--inline-schemas` only changes how schemas render, so it warns when used
// without `--include-schemas`.
#[test]
fn inline_schemas_without_include_schemas_warns() {
    vimanam()
        .arg(OAS3)
        .args(["--detail", "full", "--inline-schemas"])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "--inline-schemas has no effect without --include-schemas.",
        ));
}

// `--required-only` only filters the parameters table, which basic/summary
// detail never renders — so it warns there like the other no-effect flags.
#[test]
fn required_only_at_basic_detail_warns() {
    vimanam()
        .arg(OAS3)
        .args(["--detail", "basic", "--required-only"])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "--required-only has no effect at --detail basic; use --detail standard or full.",
        ));
}

// At `--detail standard` the flag takes effect, so no warning is emitted.
#[test]
fn required_only_at_standard_detail_emits_no_warning() {
    vimanam()
        .arg(OAS3)
        .args(["--detail", "standard", "--required-only"])
        .assert()
        .success()
        .stderr(predicate::str::contains("no effect").not());
}

// `--toc` is the explicit opposite of `--no-toc`; the TOC is on by default, and
// when both flags are given the later one wins.
#[test]
fn toc_flag_is_accepted_and_last_one_wins() {
    vimanam()
        .arg(OAS3)
        .args(["--detail", "basic", "--toc"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Services"));

    vimanam()
        .arg(OAS3)
        .args(["--detail", "basic", "--no-toc", "--toc"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Services"));

    vimanam()
        .arg(OAS3)
        .args(["--detail", "basic", "--toc", "--no-toc"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Services").not());
}

// #70 follow-up: an operation carrying multiple tags is rendered under each
// service section, so its heading anchor must be scoped per service to stay
// unique — and each TOC link must point at the matching copy.
#[test]
fn multi_tag_endpoint_gets_unique_anchors_per_service() {
    let output = String::from_utf8(
        vimanam()
            .arg(MULTI_TAG)
            .args(["--detail", "basic"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    // Distinct, service-scoped heading anchors (no duplicate `#delete-pets-petid`).
    assert!(
        output.contains("### DeletePet {#pets-delete-pets-petid}"),
        "missing Pets-scoped anchor:\n{output}"
    );
    assert!(
        output.contains("### DeletePet {#admin-delete-pets-petid}"),
        "missing Admin-scoped anchor:\n{output}"
    );

    // Each TOC entry links to the copy under its own service.
    assert!(
        output.contains("* [DeletePet](#pets-delete-pets-petid)")
            && output.contains("* [DeletePet](#admin-delete-pets-petid)"),
        "TOC links do not match per-service anchors:\n{output}"
    );
}

// Regression test for #16: the Authentication section is emitted in spec
// (file) order, not the random order of a HashMap, and is stable across runs.
#[test]
fn multiple_security_schemes_preserve_spec_order() {
    let run = || {
        String::from_utf8(
            vimanam()
                .arg(OAS3_MULTI_AUTH)
                .arg("--include-auth")
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
    };

    let output = run();

    let zebra = output.find("zebraAuth").expect("zebraAuth missing");
    let api_key = output.find("apiKeyAuth").expect("apiKeyAuth missing");
    let middle = output.find("middleAuth").expect("middleAuth missing");

    // Schemes appear in the order they are declared in the spec file.
    assert!(
        zebra < api_key && api_key < middle,
        "security schemes not in spec order: {output}"
    );

    // And that order is deterministic across runs.
    for _ in 0..4 {
        assert_eq!(output, run(), "authentication order differed between runs");
    }
}

// Companion to #16 for OpenAPI 2.0: `securityDefinitions` are read through the
// extensions map, so they only preserve spec order with serde_json's
// `preserve_order` feature (otherwise they sort alphabetically). The schemes
// are declared zebra/apiKey/middle, which is not alphabetical.
#[test]
fn oas2_security_schemes_preserve_spec_order() {
    let output = String::from_utf8(
        vimanam()
            .arg(OAS2_MULTI_AUTH)
            .arg("--include-auth")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let zebra = output.find("zebraAuth").expect("zebraAuth missing");
    let api_key = output.find("apiKey").expect("apiKey missing");
    let middle = output.find("middleAuth").expect("middleAuth missing");

    assert!(
        zebra < api_key && api_key < middle,
        "OAS2 security schemes not in spec order: {output}"
    );
}

// Regression test for #20: `--group-by method` must behave like `--method`,
// producing HTTP-method sections rather than service sections.
#[test]
fn group_by_method_groups_by_http_method() {
    vimanam()
        .arg(OAS3)
        .args(["--detail", "basic", "--group-by", "method"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## GET"))
        .stdout(predicate::str::contains("## POST"));
}

// Regression test for #18: under alphabetical sort the TOC operation links must
// appear in the same order as the endpoint sections in the body.
#[test]
fn toc_order_matches_body_order() {
    let output = String::from_utf8(
        vimanam()
            .arg(OAS3)
            .args(["--detail", "basic", "--sort", "alpha"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    // The Pets service has CreatePet (POST /pets) and ListPets (GET /pets);
    // sorted by path then method, GET sorts before POST, so ListPets precedes
    // CreatePet in both the TOC and the body.
    let toc_list = output
        .find("[Pets_ListPets]")
        .expect("ListPets TOC link missing");
    let toc_create = output
        .find("[Pets_CreatePet]")
        .expect("CreatePet TOC link missing");
    let body_list = output
        .find("### Pets_ListPets")
        .expect("ListPets section missing");
    let body_create = output
        .find("### Pets_CreatePet")
        .expect("CreatePet section missing");

    assert!(toc_list < toc_create, "TOC order unexpected: {output}");
    assert!(body_list < body_create, "body order unexpected: {output}");
}

// #6: `--include-examples` at `--detail full` renders the request body's inline
// example and the response example resolved from a `$ref` into
// `components/examples`.
#[test]
fn include_examples_renders_request_and_response() {
    vimanam()
        .arg(OAS3_EXAMPLES)
        .args(["--detail", "full", "--include-examples"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#### Examples"))
        // Inline request body example.
        .stdout(predicate::str::contains("**Request**"))
        .stdout(predicate::str::contains("\"name\": \"Fluffy\""))
        // Response example resolved through #/components/examples/CreatedPet.
        .stdout(predicate::str::contains("Response `201`"))
        .stdout(predicate::str::contains("\"id\": 7"));
}

// Examples only render at `--detail full`, matching `--include-schemas`.
#[test]
fn include_examples_only_at_full_detail() {
    vimanam()
        .arg(OAS3_EXAMPLES)
        .args(["--detail", "standard", "--include-examples"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#### Examples").not());
}

// A `requestBody` given as a `$ref` into `components/requestBodies` is resolved
// during parsing: its description/required surface in the parameter table, and
// at `--detail full` its referenced schema expands. Before resolution such a
// spec failed to parse at all.
#[test]
fn ref_request_body_is_resolved() {
    vimanam()
        .arg(OAS3_REF_BODY)
        .args(["--detail", "standard"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "| `requestBody` | body | Yes | Pet to add |",
        ));

    vimanam()
        .arg(OAS3_REF_BODY)
        .args(["--detail", "full", "--include-schemas"])
        .assert()
        .success()
        // The resolved body schema is linked and expanded in the definitions
        // section.
        .stdout(predicate::str::contains("## Schema Definitions"))
        .stdout(predicate::str::contains(
            "| `Pet.name` | string | Yes | Pet name |",
        ));
}

// #8: `--group-by path` produces one section per path with its operations
// underneath.
#[test]
fn group_by_path_groups_by_path() {
    vimanam()
        .arg(OAS3)
        .args(["--detail", "basic", "--group-by", "path"])
        .assert()
        .success()
        .stdout(predicate::str::contains("## Paths"))
        .stdout(predicate::str::contains("## /pets/{petId}"))
        .stdout(predicate::str::contains("## /store/orders"))
        .stdout(predicate::str::contains("### Pets_ListPets"))
        .stdout(predicate::str::contains("### Pets_CreatePet"));
}

// #7: a tiny `--max-tokens` budget forces a full-detail request down to a lower
// detail level and reports the reduction on stderr.
#[test]
fn max_tokens_steps_down_detail_level() {
    vimanam()
        .arg(OAS3)
        .args([
            "--detail",
            "full",
            "--include-schemas",
            "--max-tokens",
            "40",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("token budget"))
        .stderr(predicate::str::contains("--detail summary"));
}

// A generous `--max-tokens` budget leaves the requested detail untouched and
// emits no stderr note.
#[test]
fn max_tokens_keeps_detail_when_it_fits() {
    vimanam()
        .arg(OAS3)
        .args(["--detail", "basic", "--max-tokens", "100000"])
        .assert()
        .success()
        .stdout(predicate::str::contains("### Pets_ListPets"))
        .stderr(predicate::str::is_empty());
}

// Under `--inline-schemas` the recursive expansion still guards against `$ref`
// cycles, breaking the chain with a "cycle detected" row.
#[test]
fn inline_schema_expansion_detects_ref_cycles() {
    vimanam()
        .arg(OAS3_SCHEMA_REFS)
        .args(["--detail", "full", "--include-schemas", "--inline-schemas"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Cycle detected while expanding schema reference",
        ));
}

// #48: a parameter declared as a component `$ref` is resolved instead of
// failing the whole parse (a bare `$ref` param used to crash on `missing field name`).
#[test]
fn ref_parameter_is_resolved() {
    vimanam()
        .arg(REF_PARAMETER)
        .args(["--detail", "standard"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "| `limit` | query | No | Max results |",
        ));
}

// #50: a path item declared as a `$ref` yields its operation instead of being
// silently dropped.
#[test]
fn path_item_ref_yields_operation() {
    vimanam()
        .arg(REF_PATH_ITEM)
        .args(["--detail", "basic"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Things_ListThings"))
        .stdout(predicate::str::contains("**Operation:** GET /things"));
}

// #51: OpenAPI 3.1 `type` arrays (e.g. ["string","null"]) parse instead of
// failing on "invalid type: sequence".
#[test]
fn type_array_parameter_parses() {
    vimanam()
        .arg(TYPE_ARRAY)
        .args(["--detail", "standard"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "| `q` | query | No | Search term |",
        ));
}

// #56: an operation missing its `responses` block no longer fails the whole
// document; both operations are still rendered.
#[test]
fn operation_missing_responses_still_parses() {
    vimanam()
        .arg(MISSING_RESPONSES)
        .args(["--detail", "basic"])
        .assert()
        .success()
        .stdout(predicate::str::contains("A_NoResponses"))
        .stdout(predicate::str::contains("B_HasResponses"));
}

// #54: an operation-level parameter overrides a path-level one of the same
// (name, in) — it should appear exactly once.
#[test]
fn duplicate_parameter_is_deduplicated() {
    let output = vimanam()
        .arg(OVERRIDE_PARAM)
        .args(["--detail", "standard"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).unwrap();
    let id_rows = text.matches("| `id` | path |").count();
    assert_eq!(id_rows, 1, "expected a single `id` row, got {id_rows}");
    // The operation-level definition wins.
    assert!(text.contains("Operation-level id wins"));
}

// #60: an operation tagged with a value not in the declared `tags` list gets its
// own service section instead of being silently reassigned to the first service.
#[test]
fn unknown_operation_tag_keeps_its_own_service() {
    vimanam()
        .arg(UNKNOWN_TAG)
        .args(["--detail", "basic"])
        .assert()
        .success()
        // The undeclared tag Gamma becomes its own service section (under the
        // bug it would not exist — the endpoint was dumped under Alpha)...
        .stdout(predicate::str::contains("## Gamma"))
        .stdout(predicate::str::contains("W_Get"))
        // ...and the first declared service ends up with no endpoints.
        .stdout(predicate::str::contains(
            "## Alpha {#alpha}\n\nNo endpoints found for this service.",
        ));
}

// Shell completions (#41): `vimanam completions <SHELL>` prints a completion
// script for every shell clap_complete supports, without needing a spec file.
#[test]
fn completions_are_generated_for_each_supported_shell() {
    // Each shell is paired with a marker unique to its script format, so the
    // test fails if every shell were to emit the same (e.g. bash) script.
    let shells = [
        ("bash", "complete -F _vimanam"),
        ("zsh", "#compdef vimanam"),
        ("fish", "complete -c vimanam"),
        ("powershell", "Register-ArgumentCompleter"),
        ("elvish", "edit:completion:arg-completer[vimanam]"),
    ];
    for (shell, marker) in shells {
        vimanam()
            .args(["completions", shell])
            .assert()
            .success()
            .stdout(predicate::str::contains(marker))
            .stderr(predicate::str::is_empty());
    }

    // The script must actually cover the CLI's options.
    vimanam()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--group-by"));
}

#[test]
fn completions_rejects_unsupported_shell() {
    vimanam()
        .args(["completions", "nushell"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("invalid value 'nushell'"));
}

#[test]
fn completions_requires_a_shell() {
    vimanam()
        .arg("completions")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("<SHELL>"));
}

// The subcommand must not loosen the normal path: with no subcommand the spec
// file is still required.
#[test]
fn no_arguments_still_requires_input_file() {
    vimanam()
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("<FILE>"));
}

// Conversion flags and the spec file are meaningless alongside a subcommand,
// so mixing them is an error rather than being silently ignored.
#[test]
fn completions_conflicts_with_conversion_arguments() {
    vimanam()
        .args([OAS3, "completions", "bash"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

// --- Spec hygiene report (#44) ---

// A spec that trips every hygiene check at least once.
const HYGIENE: &str = "tests/fixtures/hygiene_oas3.json";
// One operation whose undescribed requestBody offers two media types.
const HYGIENE_MULTI_BODY: &str = "tests/fixtures/hygiene_multi_body_oas3.json";

// The report is appended after the body by default, separated by a rule.
#[test]
fn hygiene_report_is_appended_by_default() {
    vimanam()
        .arg(OAS3)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\n---\n\n## Spec Hygiene Report\n",
        ))
        .stdout(predicate::str::contains(
            "**4 endpoints** across **2 services**",
        ))
        .stdout(predicate::str::contains("| Deprecated | 1 |"))
        .stdout(predicate::str::contains(
            "### Deprecated (1)\n- `GET /store/orders`\n",
        ));
}

#[test]
fn no_report_suppresses_hygiene_report() {
    vimanam()
        .arg(OAS3)
        .arg("--no-report")
        .assert()
        .success()
        .stdout(predicate::str::contains("# Petstore API"))
        .stdout(predicate::str::contains("Spec Hygiene Report").not())
        .stdout(predicate::str::contains("\n---\n").not());
}

// Every check fires on the hygiene fixture; the whole report is asserted
// byte-for-byte so its shape (table order, list order, list item format) is
// pinned down.
#[test]
fn hygiene_report_lists_every_check() {
    let output = String::from_utf8(
        vimanam()
            .arg(HYGIENE)
            .args(["--detail", "standard"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    // A plain multi-line literal (no `\` continuations) so the two-space
    // indentation of the nested duplicate-operationId items is preserved.
    let expected = "
---

## Spec Hygiene Report

**6 endpoints** across **2 services**

| Check | Count |
|-------|------:|
| Missing description | 1 |
| Missing operationId | 1 |
| No responses documented | 1 |
| Deprecated | 2 |
| Untagged (no service tag) | 1 |
| Duplicate operationIds | 1 |
| Parameters without description | 3 |

### Missing description (1)
- `GET /health`

### Missing operationId (1)
- `GET /health`

### No responses documented (1)
- `POST /users`

### Deprecated (2)
- `GET /ping`
- `DELETE /users/{id}`

### Untagged (no service tag) (1)
- `GET /health`

### Duplicate operationIds (1)
- `getUser`
  - `DELETE /users/{id}`
  - `GET /users/{id}`

### Parameters without description (3)
- `GET /users` — `limit`
- `POST /users` — `requestBody`
- `DELETE /users/{id}` — `id`
";

    assert!(
        output.ends_with(expected),
        "report did not match.\nexpected tail:\n{expected}\nactual output:\n{output}"
    );
    // The body precedes the report.
    assert!(output.starts_with("# Hygiene API\n"), "{output}");
}

// The report analyzes the same endpoint set the body rendered: a service
// filter narrows both the counts and the service total.
#[test]
fn hygiene_report_respects_service_filter() {
    vimanam()
        .arg(HYGIENE)
        .args(["--service-filter", "Health"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "**1 endpoint** across **1 service**",
        ))
        .stdout(predicate::str::contains("| Deprecated | 1 |"))
        .stdout(predicate::str::contains(
            "| Untagged (no service tag) | 0 |",
        ))
        .stdout(predicate::str::contains("| Duplicate operationIds | 0 |"))
        .stdout(predicate::str::contains(
            "### Deprecated (1)\n- `GET /ping`\n",
        ))
        .stdout(predicate::str::contains("GET /health").not());
}

// `--exclude-deprecated` removes the deprecated DELETE, which also dissolves
// the duplicate operationId pair and drops its undescribed parameter; the
// deprecated GET /ping was the only Health endpoint, so one service remains.
#[test]
fn hygiene_report_respects_exclude_deprecated() {
    vimanam()
        .arg(HYGIENE)
        .arg("--exclude-deprecated")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "**4 endpoints** across **1 service**",
        ))
        .stdout(predicate::str::contains("| Deprecated | 0 |"))
        .stdout(predicate::str::contains("| Duplicate operationIds | 0 |"))
        .stdout(predicate::str::contains(
            "| Parameters without description | 2 |",
        ))
        .stdout(predicate::str::contains("### Deprecated").not())
        .stdout(predicate::str::contains("### Duplicate operationIds").not());
}

// `--max-tokens` budgets the body only; the report is still appended.
#[test]
fn hygiene_report_is_appended_under_max_tokens() {
    vimanam()
        .arg(OAS3)
        .args(["--detail", "full", "--max-tokens", "40"])
        .assert()
        .success()
        .stderr(predicate::str::contains("token budget"))
        .stdout(predicate::str::contains("## Spec Hygiene Report"));
}

// The file output path appends the report just like stdout does.
#[test]
fn hygiene_report_is_written_to_output_file() {
    let dir = tempfile::tempdir().unwrap();
    let out_path = dir.path().join("out.md");

    vimanam()
        .arg(HYGIENE)
        .args(["-o", out_path.to_str().unwrap()])
        .assert()
        .success();

    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(content.starts_with("# Hygiene API\n"), "{content}");
    assert!(
        content.contains("\n---\n\n## Spec Hygiene Report\n"),
        "{content}"
    );
    assert!(
        content.contains("| Duplicate operationIds | 1 |"),
        "{content}"
    );
}

// The report is deterministic even with `--sort none` (spec order), where
// nothing but stable collections keeps the lists in a fixed order.
#[test]
fn hygiene_report_is_deterministic() {
    let run = || {
        vimanam()
            .arg(HYGIENE)
            .args(["--detail", "full", "--sort", "none"])
            .output()
            .unwrap()
            .stdout
    };

    let first = run();
    for _ in 0..4 {
        assert_eq!(first, run(), "report differed between identical runs");
    }
}

// The parser emits one synthetic body parameter per requestBody media type,
// all sharing the request body's description; an undescribed body is reported
// once per endpoint, not once per media type.
#[test]
fn hygiene_report_counts_multi_media_type_body_once() {
    let output = String::from_utf8(
        vimanam()
            .arg(HYGIENE_MULTI_BODY)
            .args(["--detail", "standard"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    // The body still documents both media types...
    assert!(
        output.contains("`requestBody (application/json)` | body |"),
        "{output}"
    );
    assert!(
        output.contains("`requestBody (application/xml)` | body |"),
        "{output}"
    );
    // ...but the report flags the body once.
    assert!(
        output.contains("| Parameters without description | 1 |"),
        "{output}"
    );
    assert!(
        output.ends_with(
            "### Parameters without description (1)\n- `POST /items` — `requestBody (application/json)`\n"
        ),
        "{output}"
    );
    assert_eq!(
        output.matches("- `POST /items` — `requestBody").count(),
        1,
        "{output}"
    );
}

// Exactly one blank line separates the body from the report at every detail
// level, even though the views themselves end with differing trailing
// whitespace (one newline at summary, a blank line otherwise).
#[test]
fn hygiene_report_is_separated_from_body_by_one_blank_line() {
    for detail in ["summary", "basic", "standard", "full"] {
        vimanam()
            .arg(OAS3)
            .args(["--detail", detail])
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "\n\n---\n\n## Spec Hygiene Report",
            ))
            .stdout(predicate::str::contains("\n\n\n---").not());
    }
}

// A multi-tag operation is rendered under each of its services but analyzed
// once; the service count still reflects every service it appears in.
#[test]
fn hygiene_report_counts_multi_tag_endpoint_once() {
    vimanam()
        .arg(MULTI_TAG)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "**1 endpoint** across **2 services**",
        ));
}

// Filters that leave nothing visible produce an all-zero report with no
// detail sections, and no services (only services with visible endpoints
// are counted).
#[test]
fn hygiene_report_on_empty_filtered_set_is_all_zero() {
    let output = String::from_utf8(
        vimanam()
            .arg(OAS3)
            .args(["--path-filter", "/nope"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    let expected = "\
## Spec Hygiene Report

**0 endpoints** across **0 services**

| Check | Count |
|-------|------:|
| Missing description | 0 |
| Missing operationId | 0 |
| No responses documented | 0 |
| Deprecated | 0 |
| Untagged (no service tag) | 0 |
| Duplicate operationIds | 0 |
| Parameters without description | 0 |
";
    assert!(output.ends_with(expected), "{output}");
    assert!(!output.contains("\n### "), "{output}");
}

// --- --stats token-budget dry-run (#42) ---

fn stats_output(args: &[&str]) -> String {
    let output = vimanam().args(args).output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(
        output.stderr.is_empty(),
        "stats wrote to stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

// Splits a stats table into (service, endpoints, ~tokens) rows, header excluded.
fn stats_rows(table: &str) -> Vec<(String, usize, usize)> {
    let mut lines = table.lines();
    assert_eq!(
        lines
            .next()
            .map(|line| line.split_whitespace().collect::<Vec<_>>()),
        Some(vec!["SERVICE", "ENDPOINTS", "~TOKENS"]),
        "{table}"
    );
    lines
        .map(|line| {
            let mut fields = line.split_whitespace().rev();
            let tokens: usize = fields.next().unwrap().parse().unwrap();
            let endpoints: usize = fields.next().unwrap().parse().unwrap();
            let name: Vec<&str> = fields.rev().collect();
            (name.join(" "), endpoints, tokens)
        })
        .collect()
}

// Petstore declares Pets (GET /pets, POST /pets, GET /pets/{petId}) and Store
// (GET /store/orders): one row each in declared order, then a TOTAL of 4.
#[test]
fn stats_lists_each_service_with_endpoint_counts_and_total() {
    let table = stats_output(&[OAS3, "--stats"]);
    let rows = stats_rows(&table);

    assert_eq!(rows.len(), 3, "{table}");
    assert_eq!((rows[0].0.as_str(), rows[0].1), ("Pets", 3));
    assert_eq!((rows[1].0.as_str(), rows[1].1), ("Store", 1));
    assert_eq!((rows[2].0.as_str(), rows[2].1), ("TOTAL", 4));
    for (name, _, tokens) in &rows {
        assert!(*tokens > 0, "{name} has no token estimate: {table}");
    }

    // Plain text only: no blank lines, no Markdown, one trailing newline.
    assert!(table.ends_with('\n') && !table.ends_with("\n\n"), "{table}");
    assert!(!table.contains("\n\n"), "{table}");
    assert!(!table.contains('#') && !table.contains('|'), "{table}");
}

// The exact layout: SERVICE padded to the widest name (the header here),
// numeric columns right-aligned under their headers, three spaces between.
#[test]
fn stats_columns_are_aligned() {
    let table = stats_output(&[OAS3, "--stats"]);
    let lines: Vec<&str> = table.lines().collect();

    assert_eq!(lines[0], "SERVICE   ENDPOINTS   ~TOKENS");
    assert!(lines[1].starts_with("Pets              3   "), "{table}");
    assert!(lines[2].starts_with("Store             1   "), "{table}");
    assert!(lines[3].starts_with("TOTAL             4   "), "{table}");
    let width = lines[0].len();
    assert!(lines.iter().all(|line| line.len() == width), "{table}");
}

#[test]
fn stats_is_deterministic() {
    let first = stats_output(&[OAS3, "--stats", "--detail", "full", "--include-schemas"]);
    let second = stats_output(&[OAS3, "--stats", "--detail", "full", "--include-schemas"]);
    assert_eq!(first, second);
}

// More detail renders more text, so the estimate for the same service grows.
#[test]
fn stats_tokens_grow_with_detail_level() {
    let summary = stats_rows(&stats_output(&[OAS3, "--stats", "--detail", "summary"]));
    let full = stats_rows(&stats_output(&[
        OAS3,
        "--stats",
        "--detail",
        "full",
        "--include-schemas",
    ]));

    for (row, summary_row) in full.iter().zip(&summary) {
        assert_eq!(row.0, summary_row.0);
        assert!(
            row.2 > summary_row.2,
            "{}: full {} <= summary {}",
            row.0,
            row.2,
            summary_row.2
        );
    }
}

#[test]
fn stats_respects_service_filter() {
    let rows = stats_rows(&stats_output(&[
        OAS3,
        "--stats",
        "--service-filter",
        "store",
    ]));

    assert_eq!(rows.len(), 2);
    assert_eq!((rows[0].0.as_str(), rows[0].1), ("Store", 1));
    assert_eq!((rows[1].0.as_str(), rows[1].1), ("TOTAL", 1));
}

// A multi-tag operation is counted in each of its service rows but once in
// the TOTAL, so the TOTAL is not the sum of the rows.
#[test]
fn stats_counts_multi_tag_endpoint_in_each_service_but_once_in_total() {
    let rows = stats_rows(&stats_output(&[MULTI_TAG, "--stats"]));

    assert_eq!(rows.len(), 3);
    assert_eq!((rows[0].0.as_str(), rows[0].1), ("Pets", 1));
    assert_eq!((rows[1].0.as_str(), rows[1].1), ("Admin", 1));
    assert_eq!((rows[2].0.as_str(), rows[2].1), ("TOTAL", 1));
}

// The hygiene fixture's Users service has 5 endpoints (one deprecated) and
// Health has only a deprecated one; excluding deprecated drops Users to 4 and
// removes the Health row entirely.
#[test]
fn stats_respects_exclude_deprecated() {
    let all = stats_rows(&stats_output(&[HYGIENE, "--stats"]));
    assert_eq!(all.len(), 3);
    assert_eq!((all[0].0.as_str(), all[0].1), ("Users", 5));
    assert_eq!((all[1].0.as_str(), all[1].1), ("Health", 1));
    assert_eq!((all[2].0.as_str(), all[2].1), ("TOTAL", 6));

    let live = stats_rows(&stats_output(&[HYGIENE, "--stats", "--exclude-deprecated"]));
    assert_eq!(live.len(), 2);
    assert_eq!((live[0].0.as_str(), live[0].1), ("Users", 4));
    assert_eq!((live[1].0.as_str(), live[1].1), ("TOTAL", 4));
}

// Filters that leave nothing visible still print the header and a zero TOTAL.
#[test]
fn stats_on_empty_filtered_set_is_header_and_zero_total() {
    let table = stats_output(&[OAS3, "--stats", "--path-filter", "/nope"]);
    assert_eq!(
        table,
        "\
SERVICE   ENDPOINTS   ~TOKENS
TOTAL             0         0
"
    );
}

// The hygiene report is never part of stats output, whatever --no-report says.
#[test]
fn stats_never_includes_hygiene_report() {
    let table = stats_output(&[OAS3, "--stats"]);
    assert!(!table.contains("Spec Hygiene Report"), "{table}");
    assert!(!table.contains("---"), "{table}");

    let with_flag = stats_output(&[OAS3, "--stats", "--no-report"]);
    assert_eq!(table, with_flag);
}

// Renders the document with `args` and returns the chars/4 token estimate of
// the resulting Markdown, exactly as `--max-tokens` and `--stats` compute it.
fn rendered_tokens(args: &[&str]) -> usize {
    let output = vimanam().args(args).arg("--no-report").output().unwrap();
    assert!(output.status.success(), "{output:?}");
    String::from_utf8(output.stdout)
        .unwrap()
        .chars()
        .count()
        .div_ceil(4)
}

// Each row's estimate must equal a real render of that service alone under the
// same flags, and TOTAL a real render of the whole document, so the table can
// be trusted as a menu for `--service-filter`.
fn assert_stats_match_real_render(render_args: &[&str]) {
    let mut stats_args = vec![OAS3, "--stats"];
    stats_args.extend_from_slice(render_args);
    let table = stats_output(&stats_args);
    let rows = stats_rows(&table);
    assert!(rows.len() > 1, "{table}");

    for (name, _, tokens) in &rows {
        let mut args = vec![OAS3];
        args.extend_from_slice(render_args);
        if name != "TOTAL" {
            args.extend_from_slice(&["--service-filter", name]);
        }
        assert_eq!(
            *tokens,
            rendered_tokens(&args),
            "{name} estimate disagrees with a real render: {table}"
        );
    }
}

#[test]
fn stats_row_tokens_match_real_render() {
    assert_stats_match_real_render(&["--detail", "full", "--include-schemas"]);
}

// The flat view renders differently from the service view; the estimate must
// follow the grouping flag rather than always sizing the service view.
#[test]
fn stats_row_tokens_match_real_render_under_flat() {
    assert_stats_match_real_render(&["--flat", "--detail", "standard"]);
}

// A service name wider than the SERVICE header widens the first column: the
// header is padded to the name, and every line still has the same length.
#[test]
fn stats_pads_service_column_to_a_long_name() {
    let table = stats_output(&[STATS_LONG_SERVICE, "--stats"]);
    let lines: Vec<&str> = table.lines().collect();
    let name = "Long Service Name";

    assert_eq!(lines.len(), 3, "{table}");
    assert!(
        lines[0].starts_with(&format!(
            "{:<width$}   ENDPOINTS",
            "SERVICE",
            width = name.len()
        )),
        "{table}"
    );
    assert!(lines[1].starts_with(&format!("{name}   ")), "{table}");
    assert!(
        lines[2].starts_with(&format!("{:<width$}   ", "TOTAL", width = name.len())),
        "{table}"
    );
    let width = lines[0].chars().count();
    assert!(
        lines.iter().all(|line| line.chars().count() == width),
        "{table}"
    );

    let rows = stats_rows(&table);
    assert_eq!((rows[0].0.as_str(), rows[0].1), (name, 2));
    assert_eq!((rows[1].0.as_str(), rows[1].1), ("TOTAL", 2));
}

// Writing a stats table to a file and budgeting a dry run are both
// meaningless; clap rejects the combinations with its usage error.
#[test]
fn stats_conflicts_with_output() {
    vimanam()
        .args([OAS3, "--stats", "-o", "stats.txt"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn stats_conflicts_with_max_tokens() {
    vimanam()
        .args([OAS3, "--stats", "--max-tokens", "100"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("cannot be used with"));
}

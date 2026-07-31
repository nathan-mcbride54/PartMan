//! Fixture evidence for the exact-pinned Slint AOT compiler boundary.

#[path = "../build_support/aot.rs"]
mod aot;
#[path = "../build_support/environment.rs"]
mod slint_environment;

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use aot::{
    AotError, CompileRequest, ForbiddenSyntax, GENERATED_RUST_FILENAME, compile_and_write,
    compile_to_memory, pinned_configuration, write_compiled_ui,
};
use i_slint_compiler::{
    ComponentSelection, DefaultTranslationContext, EmbedResourcesKind, OpenImportCallback,
};
use slint_environment::{
    DEP_MCU_EMBED_TEXTURES, KNOWN_SLINT_ENVIRONMENT_NAMES, NameSemantics,
    PARTMAN_SLINT_GUARD_NONCE, guard_environment_entries, is_forbidden_name_bytes,
};

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

const TOKEN_SOURCE: &str = r#"
import { Palette } from "std-widgets.slint";

export enum ProbeTheme { dark, light }

export global PartmanGeneratedThemeAdapter {
    out property <ProbeTheme> system-theme: Palette.color-scheme == ColorScheme.light
        ? ProbeTheme.light
        : (Palette.color-scheme == ColorScheme.dark ? ProbeTheme.dark : ProbeTheme.dark);
}

export global ProbeTokens {
    public pure function spacing(theme: ProbeTheme) -> length {
        if (theme == ProbeTheme.dark) { return 12px; }
        return 16px;
    }
}

export component ProbeWindow inherits Window {
    background: #16181c;
    default-font-family: "";
}

export component ProbeUnsafeWindow inherits Window { }
export component ProbeUnsafeTextWindow inherits Window {
    background: #16181c;
    default-font-family: "";
    Text { text: "probe"; }
}
"#;

const SUCCESS_ROOT: &str = r#"
import { ProbePanel } from "panel.slint";
import { ProbeTheme, ProbeTokens, ProbeWindow } from "../token-contract.slint";

export component ProbeApp inherits ProbeWindow {
    in property <ProbeTheme> theme: ProbeTheme.dark;
    out property <length> gap: ProbeTokens.spacing(root.theme);
    width: 320px;
    height: 200px;
    ProbePanel { }
}
"#;

const PANEL_SOURCE: &str = r#"
import { ProbeLeaf } from "leaf.slint";
export component ProbePanel inherits Rectangle { ProbeLeaf { } }
"#;

const LEAF_SOURCE: &str =
    "export component ProbeLeaf inherits Rectangle { background: #16181c; }\n";

struct Fixture {
    base: PathBuf,
    ui_root: PathBuf,
    root: PathBuf,
    token_contract: PathBuf,
    output_directory: PathBuf,
}

impl Fixture {
    fn new(root_source: &str) -> Self {
        let temporary_root = std::env::temp_dir();
        let base = (0..1_000)
            .find_map(|_| {
                let sequence = FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed);
                let candidate = temporary_root.join(format!(
                    "partman-aot-fixture-{}-{sequence}",
                    std::process::id()
                ));
                match fs::create_dir(&candidate) {
                    Ok(()) => Some(candidate),
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => None,
                    Err(error) => panic!("cannot create fixture directory: {error}"),
                }
            })
            .expect("a unique fixture directory is available");
        let ui_root = base.join("ui");
        let output_directory = base.join("out");
        fs::create_dir(&ui_root).expect("UI fixture directory is created");
        fs::create_dir(&output_directory).expect("output fixture directory is created");
        let root = ui_root.join("main.slint");
        let token_contract = base.join("token-contract.slint");
        fs::write(&root, root_source).expect("root fixture is written");
        fs::write(&token_contract, TOKEN_SOURCE).expect("token fixture is written");
        Self {
            base,
            ui_root,
            root,
            token_contract,
            output_directory,
        }
    }

    fn write_ui(&self, relative_path: &str, source: &str) -> PathBuf {
        let path = self.ui_root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("nested UI fixture directory is created");
        }
        fs::write(&path, source).expect("UI fixture source is written");
        path
    }

    fn request(&self) -> CompileRequest<'_> {
        CompileRequest {
            root: &self.root,
            ui_root: &self.ui_root,
            token_contract: &self.token_contract,
            output_directory: &self.output_directory,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if self.base.parent() == Some(std::env::temp_dir().as_path()) {
            let _ignored = fs::remove_dir_all(&self.base);
        }
    }
}

fn successful_fixture() -> Fixture {
    let fixture = Fixture::new(SUCCESS_ROOT);
    fixture.write_ui("panel.slint", PANEL_SOURCE);
    fixture.write_ui("leaf.slint", LEAF_SOURCE);
    fixture
}

fn canonical_set(paths: impl IntoIterator<Item = PathBuf>) -> BTreeSet<PathBuf> {
    paths
        .into_iter()
        .map(|path| fs::canonicalize(path).expect("fixture path canonicalizes"))
        .collect()
}

#[cfg(unix)]
fn try_create_file_symlink(target: &PathBuf, link: &PathBuf) -> bool {
    std::os::unix::fs::symlink(target, link).expect("Unix fixture symlink is created");
    true
}

#[cfg(windows)]
fn try_create_file_symlink(target: &PathBuf, link: &PathBuf) -> bool {
    match std::os::windows::fs::symlink_file(target, link) {
        Ok(()) => true,
        Err(error)
            if error.kind() == io::ErrorKind::PermissionDenied
                || error.raw_os_error() == Some(1314) =>
        {
            false
        }
        Err(error) => panic!("unexpected Windows symlink failure: {error}"),
    }
}

// Requirements: UI-008, SEC-010
//   The exact compiler configuration is Fluent, accessible, AOT-only, resource-embedded, warning-strict, and has no experimental, native-menu, debug, or translation behavior
// Evidence: pinned_configuration_sets_every_available_field_explicitly
#[test]
fn pinned_configuration_sets_every_available_field_explicitly() {
    let callback: OpenImportCallback = Rc::new(|_path| Box::pin(async { None }));
    let configuration = pinned_configuration(callback).expect("guarded configuration is created");

    assert_eq!(
        configuration.embed_resources,
        EmbedResourcesKind::EmbedAllResources
    );
    assert!(configuration.include_paths.is_empty());
    assert!(configuration.library_paths.is_empty());
    assert_eq!(configuration.style.as_deref(), Some("fluent"));
    assert!(configuration.open_import_callback.is_some());
    assert!(configuration.resource_url_mapper.is_none());
    assert!(!configuration.inline_all_elements);
    assert_eq!(configuration.const_scale_factor, None);
    assert!(configuration.accessibility);
    assert!(!configuration.enable_experimental);
    assert_eq!(configuration.translation_domain, None);
    assert_eq!(
        configuration.default_translation_context,
        DefaultTranslationContext::ComponentName
    );
    assert!(configuration.no_native_menu);
    assert_eq!(configuration.cpp_namespace, None);
    assert!(configuration.error_on_binding_loop_with_window_layout);
    assert!(!configuration.debug_info);
    assert!(configuration.debug_hooks.is_none());
    assert_eq!(
        configuration.components_to_generate,
        ComponentSelection::ExportedWindows
    );
    assert_eq!(configuration.library_name, None);
    assert_eq!(configuration.rust_module, None);
}

// Requirements: SEC-010
//   Windows names are ASCII-case-insensitive, Unix names are byte-exact even when non-Unicode, values never enter errors, and the nonce plus compiler DEP input are refused
// Evidence: ambient_name_guard_is_cross_platform_prefix_complete_and_value_blind
#[test]
fn ambient_name_guard_is_cross_platform_prefix_complete_and_value_blind() {
    assert!(is_forbidden_name_bytes(
        b"slint_future_setting",
        NameSemantics::Windows
    ));
    assert!(!is_forbidden_name_bytes(
        b"slint_future_setting",
        NameSemantics::Unix
    ));
    assert!(is_forbidden_name_bytes(b"SLINT_\xff", NameSemantics::Unix));
    assert!(is_forbidden_name_bytes(
        PARTMAN_SLINT_GUARD_NONCE.as_bytes(),
        NameSemantics::Unix
    ));
    assert!(is_forbidden_name_bytes(
        DEP_MCU_EMBED_TEXTURES.as_bytes(),
        NameSemantics::Unix
    ));
    assert!(is_forbidden_name_bytes(
        b"dep_mcu_board_support_mcu_embed_textures",
        NameSemantics::Windows
    ));

    let secret = "value-that-must-not-appear";
    let error = guard_environment_entries(
        [(OsString::from("sLiNt_STYLE"), OsString::from(secret))],
        NameSemantics::Windows,
    )
    .expect_err("mixed-case Windows prefix must be refused");
    assert_eq!(error.name(), "sLiNt_STYLE");
    assert!(!error.to_string().contains(secret));
    guard_environment_entries(
        [(OsString::from("slint_style"), OsString::from(secret))],
        NameSemantics::Unix,
    )
    .expect("lower-case Unix name is outside the byte-exact prefix");
}

// Requirements: SEC-010
//   Cargo's fixed invalidation roster covers every audited Slint 1.17.1 build/runtime name without duplicates while the full-prefix guard remains forward-compatible
// Evidence: known_environment_invalidation_roster_is_exact_and_duplicate_free
#[test]
fn known_environment_invalidation_roster_is_exact_and_duplicate_free() {
    const EXPECTED: &[&str] = &[
        "SLINT_ASSET_SECTION",
        "SLINT_BACKEND",
        "SLINT_BUNDLE_TRANSLATIONS",
        "SLINT_COMPILER_DENY_WARNINGS",
        "SLINT_CPP_NAMESPACE",
        "SLINT_DEBUG_PERFORMANCE",
        "SLINT_DEFAULT_FONT",
        "SLINT_DESTROY_WINDOW_ON_HIDE",
        "SLINT_EMBED_RESOURCES",
        "SLINT_EMBED_TEXTURES",
        "SLINT_EMIT_DEBUG_INFO",
        "SLINT_ENABLE_EXPERIMENTAL_FEATURES",
        "SLINT_FONT_PATH",
        "SLINT_FONT_SIZES",
        "SLINT_FULLSCREEN",
        "SLINT_INCLUDE_GENERATED",
        "SLINT_INLINING",
        "SLINT_LINE_BY_LINE",
        "SLINT_LIVE_PREVIEW",
        "SLINT_MACRO_CACHE",
        "SLINT_SCALE_FACTOR",
        "SLINT_SLOW_ANIMATIONS",
        "SLINT_SOFTWARE_RENDERER_PARLEY_DISABLED",
        "SLINT_STYLE",
        "SLINT_WGPU_CPU",
    ];
    assert_eq!(KNOWN_SLINT_ENVIRONMENT_NAMES, EXPECTED);
    assert!(
        KNOWN_SLINT_ENVIRONMENT_NAMES
            .windows(2)
            .all(|pair| pair[0] < pair[1])
    );
    assert!(!KNOWN_SLINT_ENVIRONMENT_NAMES.contains(&PARTMAN_SLINT_GUARD_NONCE));
    assert!(!KNOWN_SLINT_ENVIRONMENT_NAMES.contains(&DEP_MCU_EMBED_TEXTURES));
}

// Requirements: SEC-010
//   The pinned compiler emits deterministic bytes for typed imports, reports the complete canonical source graph, discovers no resources, and selects only the fixed OUT_DIR filename
// Evidence: typed_fixture_compilation_is_deterministic_tracked_and_resource_free
#[test]
fn typed_fixture_compilation_is_deterministic_tracked_and_resource_free() {
    let fixture = successful_fixture();
    let first = compile_to_memory(fixture.request()).expect("typed fixture compiles");
    let second = compile_to_memory(fixture.request()).expect("typed fixture recompiles");

    assert_eq!(first.generated_rust(), second.generated_rust());
    assert!(!first.generated_rust().is_empty());
    assert!(String::from_utf8_lossy(first.generated_rust()).contains("ProbeApp"));
    assert!(first.resource_files().is_empty());
    assert_eq!(
        first.output_path().file_name(),
        Some(GENERATED_RUST_FILENAME.as_ref())
    );
    assert_eq!(
        first.output_path().parent(),
        Some(
            fs::canonicalize(&fixture.output_directory)
                .expect("OUT_DIR canonicalizes")
                .as_path()
        )
    );
    assert_eq!(
        first.tracked_files(),
        &canonical_set([
            fixture.root.clone(),
            fixture.ui_root.join("panel.slint"),
            fixture.ui_root.join("leaf.slint"),
            fixture.token_contract.clone(),
        ])
    );
}

// Requirements: UI-001, SEC-010
//   The production root compiles to includable Rust while using only typed contrast/metric APIs and explicit Window style bindings
// Evidence: production_root_compiles_against_the_generated_token_contract
#[test]
fn production_root_compiles_against_the_generated_token_contract() {
    let fixture = Fixture::new(
        "import { ProbeWindow } from \"../token-contract.slint\"; export component Unused inherits ProbeWindow { }",
    );
    let manifest_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repository_root = manifest_directory.join("../..");
    let ui_root = manifest_directory.join("ui");
    let root = ui_root.join("main.slint");
    let token_contract =
        repository_root.join("packages/design-tokens/generated/partman-tokens.slint");
    let compiled = compile_to_memory(CompileRequest {
        root: &root,
        ui_root: &ui_root,
        token_contract: &token_contract,
        output_directory: &fixture.output_directory,
    })
    .expect("production compiler probe compiles");

    assert!(String::from_utf8_lossy(compiled.generated_rust()).contains("PartmanApp"));
    assert!(
        compiled
            .tracked_files()
            .contains(&fs::canonicalize(token_contract).expect("token contract canonicalizes"))
    );
    assert!(compiled.resource_files().is_empty());
}

// Requirements: UI-013, SEC-010
//   Root and imported syntax reject translations, assets, raw/upstream palettes, style metrics, and standard widgets before code generation or resource I/O
// Evidence: forbidden_language_constructs_are_rejected_in_roots_and_imports
#[test]
fn forbidden_language_constructs_are_rejected_in_roots_and_imports() {
    let cases = [
        (
            "export component Probe inherits Window { }",
            ForbiddenSyntax::UngovernedStyleBuiltin,
        ),
        (
            "import { ProbeWindow } from \"../token-contract.slint\"; export component Probe inherits ProbeWindow { in property <string> label: \"PartMan\"; }",
            ForbiddenSyntax::EmbeddedDisplayString,
        ),
        (
            "import { ProbeWindow } from \"../token-contract.slint\"; export component Probe inherits ProbeWindow { in property <string> label: @tr(\"probe\"); }",
            ForbiddenSyntax::Translation,
        ),
        (
            "import { ProbeWindow } from \"../token-contract.slint\"; export component Probe inherits ProbeWindow { in property <image> icon: @image-url(\"absent.png\"); }",
            ForbiddenSyntax::ImageUrl,
        ),
        (
            "import \"absent.ttf\"; import { ProbeWindow } from \"../token-contract.slint\"; export component Probe inherits ProbeWindow { }",
            ForbiddenSyntax::FontImport,
        ),
        (
            "import { ProbeWindow } from \"../token-contract.slint\"; export component Probe inherits ProbeWindow { in property <color> value: PartmanRawGeneratedPalette.color-test; }",
            ForbiddenSyntax::RawGeneratedPalette,
        ),
        (
            "import { ProbeWindow } from \"../token-contract.slint\"; export component Probe inherits ProbeWindow { in property <color> value: Palette.foreground; }",
            ForbiddenSyntax::UpstreamPalette,
        ),
        (
            "import { ProbeWindow } from \"../token-contract.slint\"; export component Probe inherits ProbeWindow { in property <length> value: StyleMetrics.layout-spacing; }",
            ForbiddenSyntax::UpstreamStyleMetrics,
        ),
        (
            "import { Button } from \"std-widgets.slint\"; import { ProbeWindow } from \"../token-contract.slint\"; export component Probe inherits ProbeWindow { Button { } }",
            ForbiddenSyntax::StandardWidgets,
        ),
    ];
    for (source, expected) in cases {
        let fixture = Fixture::new(source);
        let error = compile_to_memory(fixture.request()).expect_err("root policy must reject");
        assert!(matches!(error, AotError::Policy { syntax, .. } if syntax == expected));
    }

    let fixture = Fixture::new(
        "import { Bad } from \"bad.slint\"; import { ProbeWindow } from \"../token-contract.slint\"; export component Probe inherits ProbeWindow { Bad { } }",
    );
    fixture.write_ui(
        "bad.slint",
        "export component Bad inherits Rectangle { in property <string> label: @tr(\"bad\"); }",
    );
    let error = compile_to_memory(fixture.request()).expect_err("import policy must reject");
    assert!(matches!(
        error,
        AotError::Policy {
            syntax: ForbiddenSyntax::Translation,
            ..
        }
    ));

    let palette_fixture = successful_fixture();
    fs::write(
        &palette_fixture.token_contract,
        TOKEN_SOURCE.replacen("Palette.color-scheme", "Palette.foreground", 1),
    )
    .expect("mutated token contract is written");
    let error = compile_to_memory(palette_fixture.request())
        .expect_err("generated exception cannot read a Palette brush");
    assert!(matches!(
        error,
        AotError::Policy {
            syntax: ForbiddenSyntax::UpstreamPalette,
            ..
        }
    ));

    let widget_fixture = successful_fixture();
    fs::write(
        &widget_fixture.token_contract,
        TOKEN_SOURCE.replace("import { Palette }", "import { Button, Palette }"),
    )
    .expect("mutated widget import is written");
    let error = compile_to_memory(widget_fixture.request())
        .expect_err("generated exception cannot import an additional widget");
    assert!(matches!(
        error,
        AotError::Policy {
            syntax: ForbiddenSyntax::StandardWidgets,
            ..
        }
    ));
}

// Requirements: UI-008, SEC-010
//   Lowered PartMan-owned Window and text builtins retain authored bindings for every property the pinned compiler could otherwise source from Palette or StyleMetrics
// Work-Package: WP-030
// Evidence: implicit_style_defaults_are_rejected_after_lowering
#[test]
fn implicit_style_defaults_are_rejected_after_lowering() {
    let window = Fixture::new(
        "import { ProbeUnsafeWindow } from \"../token-contract.slint\"; export component Probe inherits ProbeUnsafeWindow { }",
    );
    let error = compile_to_memory(window.request()).expect_err("implicit Window style must fail");
    assert!(matches!(
        error,
        AotError::ImplicitStyleBinding {
            element: "Window",
            property: "background",
            ..
        }
    ));

    let text = Fixture::new(
        r#"import { ProbeUnsafeTextWindow } from "../token-contract.slint";
        export component Probe inherits ProbeUnsafeTextWindow { }"#,
    );
    let error = compile_to_memory(text.request()).expect_err("implicit Text style must fail");
    assert!(matches!(
        error,
        AotError::ImplicitStyleBinding {
            element: "Text",
            property: "color",
            ..
        }
    ));
}

// Requirements: SEC-010
//   Canonical imports are confined to the UI root plus one exact token-contract exception, including when a relative import names an existing file
// Evidence: import_callback_rejects_canonical_escape
#[test]
fn import_callback_rejects_canonical_escape() {
    let fixture = Fixture::new(
        "import { Escaped } from \"../escaped.slint\"; import { ProbeWindow } from \"../token-contract.slint\"; export component Probe inherits ProbeWindow { Escaped { } }",
    );
    let escaped = fixture.base.join("escaped.slint");
    fs::write(
        &escaped,
        "export component Escaped inherits Rectangle { }\n",
    )
    .expect("escaped fixture is written");

    let error = compile_to_memory(fixture.request()).expect_err("escaped import must fail");
    assert!(
        matches!(error, AotError::Boundary { path, .. } if path == fs::canonicalize(escaped).expect("escape canonicalizes"))
    );
}

// Requirements: SEC-010
//   Missing roots and generated-token inputs fail before compiler construction or output mutation instead of falling back to working-directory or include-path discovery
// Evidence: missing_required_inputs_are_refused
#[test]
fn missing_required_inputs_are_refused() {
    let missing_root = Fixture::new(
        "import { ProbeWindow } from \"../token-contract.slint\"; export component Probe inherits ProbeWindow { }",
    );
    fs::remove_file(&missing_root.root).expect("root fixture is removed");
    let error = compile_to_memory(missing_root.request()).expect_err("missing root must fail");
    assert!(matches!(error, AotError::Io { .. }));

    let missing_contract = successful_fixture();
    fs::remove_file(&missing_contract.token_contract).expect("token fixture is removed");
    let error = compile_to_memory(missing_contract.request())
        .expect_err("missing token contract must fail");
    assert!(matches!(error, AotError::Io { .. }));
}

// Requirements: SEC-010
//   Source and output symbolic links cannot redirect reads or writes outside the canonical compiler boundary; Windows attempts the proof and skips only when the OS denies symlink privilege
// Evidence: source_and_output_symlinks_are_refused
#[test]
fn source_and_output_symlinks_are_refused() {
    let import_fixture = Fixture::new(
        "import { Escaped } from \"linked.slint\"; import { ProbeWindow } from \"../token-contract.slint\"; export component Probe inherits ProbeWindow { Escaped { } }",
    );
    let escaped = import_fixture.base.join("escaped.slint");
    fs::write(
        &escaped,
        "export component Escaped inherits Rectangle { }\n",
    )
    .expect("symlink target source is written");
    let linked = import_fixture.ui_root.join("linked.slint");
    if !try_create_file_symlink(&escaped, &linked) {
        #[cfg(windows)]
        {
            return;
        }
        #[cfg(unix)]
        panic!("Unix symlink helper returned without creating its link");
    }
    let error =
        compile_to_memory(import_fixture.request()).expect_err("source symlink must be refused");
    assert!(matches!(error, AotError::Boundary { .. }));

    let output_fixture = successful_fixture();
    let compiled = compile_to_memory(output_fixture.request()).expect("output fixture compiles");
    let target = output_fixture.base.join("outside-output.rs");
    fs::write(&target, b"outside-sentinel").expect("output symlink target is written");
    assert!(try_create_file_symlink(
        &target,
        &compiled.output_path().to_path_buf()
    ));
    let error = write_compiled_ui(&compiled).expect_err("output symlink must be refused");
    assert!(matches!(error, AotError::Boundary { .. }));
    assert_eq!(
        fs::read(&target).expect("symlink target remains readable"),
        b"outside-sentinel"
    );
}

// Requirements: SEC-010
//   Every compiler warning, note, and error is fatal rather than being printed and ignored
// Evidence: compiler_warnings_and_errors_are_both_fatal
#[test]
fn compiler_warnings_and_errors_are_both_fatal() {
    let warning_fixture = Fixture::new(
        "import { ProbeWindow } from \"../token-contract.slint\"; export Probe := ProbeWindow { }",
    );
    let warning = compile_to_memory(warning_fixture.request()).expect_err("warning must fail");
    assert!(matches!(warning, AotError::Diagnostics(report) if report.contains("deprecated")));

    let error_fixture = Fixture::new(
        "import { ProbeWindow } from \"../token-contract.slint\"; export component Probe inherits ProbeWindow { definitely-not-a-property: true; }",
    );
    let error = compile_to_memory(error_fixture.request()).expect_err("error must fail");
    assert!(
        matches!(error, AotError::Diagnostics(report) if report.contains("definitely-not-a-property"))
    );
}

// Requirements: SEC-010
//   Compilation failure cannot mutate prior output, while successful bytes replace only the fixed regular file via exclusive creation and non-file destinations are refused intact
// Evidence: output_write_is_fixed_exclusive_and_failure_atomic
#[test]
fn output_write_is_fixed_exclusive_and_failure_atomic() {
    let failing = Fixture::new(
        "import { ProbeWindow } from \"../token-contract.slint\"; export component Probe inherits ProbeWindow { definitely-not-a-property: true; }",
    );
    let destination = failing.output_directory.join(GENERATED_RUST_FILENAME);
    fs::write(&destination, b"sentinel").expect("sentinel output is written");
    compile_and_write(failing.request()).expect_err("failed compile must not write");
    assert_eq!(
        fs::read(&destination).expect("sentinel remains readable"),
        b"sentinel"
    );

    let successful = successful_fixture();
    let compiled = compile_to_memory(successful.request()).expect("valid fixture compiles");
    let destination = compiled.output_path().to_path_buf();
    fs::write(&destination, b"old-output").expect("old regular output is written");
    write_compiled_ui(&compiled).expect("regular output is replaced");
    assert_eq!(
        fs::read(&destination).expect("generated output is readable"),
        compiled.generated_rust()
    );

    fs::remove_file(&destination).expect("generated fixture file is removed");
    fs::create_dir(&destination).expect("non-file destination fixture is created");
    let error = write_compiled_ui(&compiled).expect_err("directory destination must be refused");
    assert!(matches!(error, AotError::Boundary { .. }));
    assert!(destination.is_dir());
}

// Requirements: SEC-010
//   The application manifest exposes only the reviewed Winit/accessibility runtime, closed renderer features, and exact AOT compiler boundary
// Work-Package: WP-030
// Evidence: manifest_and_generated_runtime_boundary_are_exact
#[test]
fn manifest_and_generated_runtime_boundary_are_exact() {
    let manifest = include_str!("../Cargo.toml");
    for exact in [
        "default = [\"renderer-femtovg\"]",
        "renderer-femtovg = [\"slint/renderer-femtovg\"]",
        "renderer-software = [\"slint/renderer-software\"]",
        "comparison-combined = [\"renderer-femtovg\", \"renderer-software\"]",
        "slint = { version = \"=1.17.1\", default-features = false, features = [\"std\", \"backend-winit\", \"accessibility\", \"compat-1-2\"] }",
        "unicode-segmentation = \"=1.13.3\"",
    ] {
        assert_eq!(
            manifest.matches(exact).count(),
            1,
            "exact boundary: {exact}"
        );
    }
    assert!(!manifest.contains("slint-build"));
    assert_eq!(
        manifest
            .matches("i-slint-compiler = { version = \"=1.17.1\"")
            .count(),
        2
    );
    assert_eq!(manifest.matches("spin_on = \"=0.1.1\"").count(), 2);
    assert!(include_str!("../src/lib.rs").contains("partman_ui.rs"));
}

// Requirements: SEC-010
//   Cargo reruns the AOT build for its manifest, build script, shared compiler adapter, shared environment policy, root, generated contract, schema, imports, resources, and guarded environment names without rustc-env handoff
// Evidence: build_script_declares_complete_rerun_inputs_and_no_rustc_environment
#[test]
fn build_script_declares_complete_rerun_inputs_and_no_rustc_environment() {
    let source = include_str!("../build.rs");
    for expected in [
        "Cargo.toml",
        "build.rs",
        "build_support/aot.rs",
        "build_support/environment.rs",
        "schemas/design-tokens.json",
        "packages/design-tokens/generated/partman-tokens.slint",
        "tracked_files()",
        "resource_files()",
        "KNOWN_SLINT_ENVIRONMENT_NAMES",
        "PARTMAN_SLINT_GUARD_NONCE",
        "DEP_MCU_EMBED_TEXTURES",
    ] {
        assert!(
            source.contains(expected),
            "build script must track {expected}"
        );
    }
    assert!(!source.contains("rustc-env"));
    assert!(
        source
            .find("guard_current_environment")
            .expect("guard call exists")
            < source
                .find("emit_environment_invalidation")
                .expect("directive call exists")
    );
}

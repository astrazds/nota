use std::path::PathBuf;

#[test]
fn committed_csharp_factories_match_the_command_inventory() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let commands = std::fs::read_to_string(root.join("crates/nota-app/src/command.rs")).unwrap();
    let debounce =
        std::fs::read_to_string(root.join("crates/nota-core/src/note_list_interaction.rs"))
            .unwrap();
    let generated = csharp_factories(&commands, &debounce);
    let committed_path = root.join("windows/Nota.Windows/Interop/Commands.g.cs");
    let committed = std::fs::read_to_string(&committed_path).unwrap_or_default();
    assert_eq!(
        committed, generated,
        "Commands.g.cs drifted from commands! and SEARCH_DEBOUNCE_MS"
    );
}

fn csharp_factories(commands: &str, debounce_source: &str) -> String {
    let debounce = debounce_source
        .lines()
        .find_map(|line| {
            let line = line.trim();
            let rest = line.strip_prefix("pub const SEARCH_DEBOUNCE_MS: i32 = ")?;
            Some(rest.trim_end_matches(';').to_string())
        })
        .expect("SEARCH_DEBOUNCE_MS");
    let inventory = commands
        .split_once("commands! {")
        .unwrap()
        .1
        .split_once("\n}")
        .unwrap()
        .0;
    let mut methods = String::new();
    for line in inventory.lines() {
        let line = line.trim().trim_end_matches(',');
        if line.is_empty() {
            continue;
        }
        let (wire, rest) = line
            .trim_start_matches('"')
            .split_once("\" ")
            .unwrap_or_else(|| panic!("command line {line}"));
        let (name, rest) = rest.split_once(' ').unwrap();
        let fields = rest
            .trim_start_matches('{')
            .split_once('}')
            .unwrap()
            .0
            .trim();
        methods.push_str(&factory(wire, name, fields));
        methods.push('\n');
    }
    let mut output = String::from(
        "// Generated from the commands! inventory. Do not edit by hand.\nnamespace Nota.Windows.Interop;\n\npublic static class Commands\n{\n    public const int SearchDebounceMs = ",
    );
    output.push_str(&debounce);
    output.push_str(";\n\n");
    output.push_str(&methods);
    output.push_str("}\n");
    output
}

fn factory(wire: &str, name: &str, fields: &str) -> String {
    if wire == "format" {
        return format!(
            "    public static object Format(string content, int startUtf16, int endUtf16, string kind) => new {{ command = \"{wire}\", content, start_utf16 = startUtf16, end_utf16 = endUtf16, kind }};"
        );
    }
    if fields.is_empty() {
        return format!("    public static object {name}() => new {{ command = \"{wire}\" }};");
    }
    let mut parameters = Vec::new();
    let mut properties = Vec::new();
    for field in fields.split(',') {
        let field = field.trim();
        if field.is_empty() {
            continue;
        }
        let (ident, ty) = field.split_once(':').unwrap();
        let ident = ident.trim();
        let ty = ty.trim();
        let camel = camel(ident);
        let csharp = match ty {
            "f64" => "double",
            "u64" => "ulong",
            "bool" => "bool",
            "String" => "string",
            "OptionString" => "string?",
            "Uuid" | "PathBuf" | "EditorViewMode" | "ThemePreference" | "MarkdownCommand" => {
                "string"
            }
            other => panic!("unmapped command field type {other}"),
        };
        parameters.push(format!("{csharp} {camel}"));
        if camel == ident {
            properties.push(ident.to_string());
        } else {
            properties.push(format!("{ident} = {camel}"));
        }
    }
    format!(
        "    public static object {name}({}) => new {{ command = \"{wire}\", {} }};",
        parameters.join(", "),
        properties.join(", ")
    )
}

fn camel(name: &str) -> String {
    let mut parts = name.split('_');
    let mut out = parts.next().unwrap_or("").to_string();
    for part in parts {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.push(first.to_ascii_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
}

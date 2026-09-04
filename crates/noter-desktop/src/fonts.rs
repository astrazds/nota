use std::path::{Path, PathBuf};

pub const BUNDLED_FONT_FILES: &[&str] = &[
    "source-sans-3-latin-wght-normal.woff2",
    "source-sans-3-latin-wght-italic.woff2",
    "source-code-pro-latin-wght-normal.woff2",
    "source-code-pro-latin-wght-italic.woff2",
];

pub fn bundled_font_search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(dir) = std::env::var_os("NOTER_FONT_DIR") {
        dirs.push(PathBuf::from(dir));
    }
    if let Some(dirs_env) = std::env::var_os("XDG_DATA_DIRS") {
        for part in std::env::split_paths(&dirs_env) {
            dirs.push(part.join("net.astrazds.Noter/fonts"));
        }
    }
    dirs.push(PathBuf::from("/usr/share/net.astrazds.Noter/fonts"));
    dirs.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts"),
    );
    dirs
}

pub fn bundled_font_paths() -> Vec<PathBuf> {
    bundled_font_search_dirs()
        .into_iter()
        .flat_map(|dir| {
            BUNDLED_FONT_FILES
                .iter()
                .map(move |file| dir.join(file))
        })
        .filter(|path| path.is_file())
        .collect()
}

pub fn is_node_modules_font_path(path: &Path) -> bool {
    path.components().any(|component| {
        component.as_os_str() == "node_modules"
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_fonts_do_not_depend_on_node_modules() {
        assert!(bundled_font_search_dirs().iter().all(|dir| !is_node_modules_font_path(dir)));
        assert!(BUNDLED_FONT_FILES.iter().all(|file| file.ends_with(".woff2")));
    }
}

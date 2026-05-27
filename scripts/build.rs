use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("file_icons_map.rs");

    let mut svg_files = Vec::<PathBuf>::new();
    collect_svg_files(Path::new("assets/file_icons"), &mut svg_files);
    svg_files.sort();
    for path in &svg_files {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    let mut map_content = String::new();
    map_content.push_str("pub static FILE_ICONS: &[(&str, &[u8])] = &[\n");

    for path in &svg_files {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let key = file_name.strip_suffix(".svg").unwrap();

        let content = fs::read_to_string(path).expect("Failed to read SVG file");
        let white_content = content
            .replace("stroke=\"black\"", "stroke=\"white\"")
            .replace("fill=\"black\"", "fill=\"white\"")
            .replace("stroke=\"#000000\"", "stroke=\"white\"")
            .replace("fill=\"#000000\"", "fill=\"white\"")
            .replace("stroke=\"#000\"", "stroke=\"white\"")
            .replace("fill=\"#000\"", "fill=\"white\"")
            .replace("stroke='black'", "stroke='white'")
            .replace("fill='black'", "fill='white'")
            .replace("stroke='#000000'", "stroke='white'")
            .replace("fill='#000000'", "fill='white'")
            .replace("stroke='#000'", "stroke='white'")
            .replace("fill='#000'", "fill='white'")
            .replace("stroke=\"#1e1e24\"", "stroke=\"white\"")
            .replace("fill=\"#1e1e24\"", "fill=\"white\"")
            .replace("stroke=\"#180c25\"", "stroke=\"white\"")
            .replace("fill=\"#180c25\"", "fill=\"white\"");

        let bytes = white_content.as_bytes();
        map_content.push_str(&format!("    (\"{}\", &{:?}),\n", key, bytes));
    }
    map_content.push_str("];\n");

    fs::write(&dest_path, map_content).unwrap();
}

fn collect_svg_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_svg_files(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("svg") {
            out.push(path);
        }
    }
}

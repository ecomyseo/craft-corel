//! Compare the modules read from a project file with exported source
//! files in a folder: `cargo run --example compare -- file.gms folder`.

fn norm(s: &str) -> Vec<String> {
    s.lines()
        .map(|l| l.trim_end().to_string())
        .filter(|l| !l.trim_start().starts_with("Attribute ") && !l.is_empty())
        .collect()
}

fn main() {
    let mut args = std::env::args().skip(1);
    let gms = std::fs::read(args.next().expect("file")).expect("read");
    let folder = args.next().expect("folder");
    let p = tracedraw_vba_project::read_gms(&gms).expect("project");
    let mut paths = Vec::new();
    for e in std::fs::read_dir(&folder).expect("folder").flatten() {
        paths.push(e.path());
    }
    let src = tracedraw_vba_project::read_files(&paths).expect("sources");
    for m in &p.modules {
        match src
            .modules
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(&m.name))
        {
            None => println!("{}: no source file", m.name),
            Some(s) => {
                let (a, b) = (norm(&m.source), norm(&s.source));
                let same = a == b;
                let diff = a.iter().zip(&b).position(|(x, y)| x != y);
                println!(
                    "{}: {} ({} vs {} lines){}",
                    m.name,
                    if same { "same" } else { "DIFFERENT" },
                    a.len(),
                    b.len(),
                    diff.map(|i| format!(" first diff at {}: {:?} | {:?}", i, a[i], b[i]))
                        .unwrap_or_default()
                );
            }
        }
    }
}

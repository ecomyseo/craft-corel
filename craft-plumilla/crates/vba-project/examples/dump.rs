//! Print what a VBA project file holds: modules (with their line counts)
//! and forms with their controls. `cargo run --example dump -- file.gms`.

fn main() {
    let path = std::env::args().nth(1).expect("a .gms file");
    let bytes = std::fs::read(&path).expect("read");
    match tracedraw_vba_project::read_gms(&bytes) {
        Err(e) => println!("error: {e}"),
        Ok(p) => {
            println!(
                "project {:?} protected={} code_page={} warnings={:?}",
                p.name, p.protected, p.code_page, p.warnings
            );
            for m in &p.modules {
                println!(
                    "module {} {:?} lines={} source={}",
                    m.name,
                    m.kind,
                    m.source.lines().count(),
                    m.has_source
                );
            }
            for f in &p.forms {
                println!("form {:?}", f.name);
                fn walk(c: &[tracedraw_vba_project::Control], depth: usize) {
                    for c in c {
                        println!(
                            "{}{:?} {} {:?} at ({:.1},{:.1}) {:.1}x{:.1}",
                            "  ".repeat(depth + 1),
                            c.kind,
                            c.name,
                            c.caption,
                            c.left,
                            c.top,
                            c.width,
                            c.height
                        );
                        walk(&c.children, depth + 1);
                    }
                }
                walk(&f.controls, 0);
            }
        }
    }
}

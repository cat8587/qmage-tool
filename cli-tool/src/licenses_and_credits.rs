pub const LICENSE_TEXT: &'static str = include_str!("../../LICENSE");
pub const LICENSE_NOTE: &'static str = "Use `qmage-tool license LGPL-2.1` and `qmage-tool license CC0-1.0` to view license terms and `qmage-tool credits` for the other software used here";
pub const CREDITS_TEXT: &'static str = include_str!("../../CREDITS");
pub const CREDITS_NOTE: &'static str = "Use `qmage-tool license <license>` to view a license's terms or `qmage-tool license` to view the project's license.";
pub const LICENSE_LIST: [(&'static str, &'static [&'static str], &'static str); 8] = [
    ("MIT", &[], include_str!("../../licenses/MIT.txt")),
    (
        "LGPL-2.1",
        &["LGPL-2.1-or-later"],
        include_str!("../../licenses/LGPL-2.1.txt"),
    ),
    (
        "Apache-2.0",
        &[],
        include_str!("../../licenses/Apache-2.0.txt"),
    ),
    ("CC0-1.0", &[], include_str!("../../licenses/CC0-1.0.txt")),
    (
        "Unicode-3.0",
        &[],
        include_str!("../../licenses/Unicode-3.0.txt"),
    ),
    ("Zlib", &["libpng"], include_str!("../../licenses/Zlib.txt")),
    (
        "Unlicense",
        &[],
        include_str!("../../licenses/Unlicense.txt"),
    ),
    ("0BSD", &[], include_str!("../../licenses/0BSD.txt")),
];

pub fn print_credits(note: bool) {
    println!("{}", CREDITS_TEXT);
    if note {
        println!();
        println!("{}", CREDITS_NOTE);
    }
}

pub fn print_project_license(note: bool) {
    println!("{}", LICENSE_TEXT);
    if note {
        println!();
        println!("{}", LICENSE_NOTE);
    }
}

pub fn find_license(spdx: &str) -> Option<&'static str> {
    let spdx = spdx.trim().to_uppercase();
    for (name, aka, text) in LICENSE_LIST {
        if spdx == name.to_uppercase() {
            return Some(text);
        } else {
            for other_name in aka {
                if spdx == other_name.to_uppercase() {
                    return Some(text);
                }
            }
        }
    }
    None
}

pub fn print_license_text(spdx: &str) -> Option<()> {
    let found_text = find_license(spdx);
    if let Some(t) = found_text {
        println!("{}", t);
        Some(())
    } else {
        let license_list = LICENSE_LIST
            .iter()
            .map(|(name, _, _)| *name)
            .collect::<Vec<_>>();
        println!(
            "Licence {} not found, known licenses:\n{}",
            spdx,
            license_list.join(", ")
        );
        None
    }
}

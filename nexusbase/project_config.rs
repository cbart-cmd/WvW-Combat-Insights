use toml_edit::Document;

pub struct Project {
    pub name: String,
    pub signature: i32,
    pub update_link: String,
    pub language_key: String,
    pub data_directory: String,
    pub settings_file: String,
}

pub fn parse(source: &str) -> Result<Project, String> {
    let doc = source
        .parse::<Document<_>>()
        .map_err(|error| error.to_string())?;
    let fields = [
        "name",
        "signature",
        "update_link",
        "language_key",
        "data_directory",
        "settings_file",
    ];
    for (key, _) in doc.iter() {
        if !fields.contains(&key) {
            return Err(format!("unknown project setting: {key}"));
        }
    }
    let text = |key: &str| -> Result<String, String> {
        let value = doc
            .get(key)
            .and_then(|item| item.as_str())
            .ok_or_else(|| format!("{key} must be a string"))?;
        if value.trim().is_empty() || value.chars().any(char::is_control) {
            return Err(format!(
                "{key} must be non-empty and contain no control characters"
            ));
        }
        Ok(value.to_owned())
    };
    let signature = doc
        .get("signature")
        .and_then(|item| item.as_integer())
        .and_then(|value| i32::try_from(value).ok())
        .filter(|value| *value != 0)
        .ok_or("signature must be a non-zero i32")?;
    let project = Project {
        name: text("name")?,
        signature,
        update_link: text("update_link")?,
        language_key: text("language_key")?,
        data_directory: text("data_directory")?,
        settings_file: text("settings_file")?,
    };
    let endpoint = project
        .update_link
        .strip_prefix("https://")
        .ok_or("update_link must use HTTPS")?;
    let (host, path) = endpoint
        .split_once('/')
        .ok_or("update_link must include a host and DLL path")?;
    if host.is_empty()
        || host.contains(['@', '\\', ' ', '?', '#'])
        || !path.ends_with(".dll")
        || path.contains(['\\', ' ', '?', '#'])
    {
        return Err("update_link must point directly to an HTTPS DLL".into());
    }
    for (key, value) in [
        ("data_directory", &project.data_directory),
        ("settings_file", &project.settings_file),
    ] {
        if value == "."
            || value == ".."
            || value.ends_with(['.', ' '])
            || value
                .chars()
                .any(|c| c.is_control() || "<>:\"/\\|?*".contains(c))
        {
            return Err(format!("{key} must be a single Windows filename"));
        }
    }
    Ok(project)
}

impl Project {
    pub fn constants(&self) -> String {
        [
            ("NAME", &self.name),
            ("UPDATE_LINK", &self.update_link),
            ("LANGUAGE_KEY", &self.language_key),
            ("DATA_DIRECTORY", &self.data_directory),
            ("SETTINGS_FILE", &self.settings_file),
        ]
        .map(|(key, value)| format!("pub const {key}: &str = {value:?};\n"))
        .concat()
            + &format!("pub const SIGNATURE: i32 = {};\n", self.signature)
    }

    pub fn export(&self) -> String {
        format!(
            "nexus::export! {{ name: {:?}, signature: {}, load: crate::nexusbase::lifecycle::load, unload: crate::nexusbase::lifecycle::unload, flags: nexus::AddonFlags::None, provider: nexus::UpdateProvider::Direct, update_link: {:?}, }}",
            self.name, self.signature, self.update_link
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = include_str!("..\\addon.toml");

    #[test]
    fn project_identity_and_generated_export_agree() {
        let project = parse(SOURCE).unwrap();
        let expected = match env!("CARGO_PKG_NAME") {
            "fight_analysis_danger" => (
                "WvW Combat Insights",
                -0x4641_4444,
                "fight_analysis_danger",
                "settings.txt",
            ),
            "timers" => ("Timers", -0x426F_6F6E, "timers_wvw", "settings.json"),
            name => panic!("add identity regression expectations for {name}"),
        };
        assert_eq!(
            (
                &*project.name,
                project.signature,
                &*project.data_directory,
                &*project.settings_file
            ),
            expected
        );
        assert!(
            project
                .export()
                .contains(&format!("name: {:?}", project.name))
        );
        assert!(
            project
                .export()
                .contains(&format!("signature: {}", project.signature))
        );
        assert!(
            project
                .export()
                .contains(&format!("update_link: {:?}", project.update_link))
        );
        assert!(project.constants().contains(&format!(
            "pub const UPDATE_LINK: &str = {:?};",
            project.update_link
        )));
        assert!(
            project
                .export()
                .contains("provider: nexus::UpdateProvider::Direct")
        );
        if env!("CARGO_PKG_NAME") == "fight_analysis_danger" {
            assert_eq!(
                project.update_link,
                "https://raw.githubusercontent.com/cbart-cmd/gw2addonsreleases/main/fight_analysis_danger.dll"
            );
        }
    }

    #[test]
    fn rejects_missing_unknown_wrong_type_and_unsafe_settings() {
        assert!(parse("").is_err());
        assert!(parse(&format!("{SOURCE}\nunknown = true")).is_err());
        for (from, to) in [
            ("https://", "http://"),
            (".dll\"", ".txt\""),
            ("signature = ", "signature = \""),
            ("data_directory = \"", "data_directory = \"..\\\\"),
            ("settings_file = \"", "settings_file = \"C:\\\\"),
            ("name = \"", "name = \"\\u0000"),
        ] {
            assert!(
                parse(&SOURCE.replacen(from, to, 1)).is_err(),
                "{from} -> {to}"
            );
        }
        let project = parse(SOURCE).unwrap();
        let original = format!("signature = {}", project.signature);
        for invalid in ["0", "2147483648", "\"text\""] {
            assert!(parse(&SOURCE.replace(&original, &format!("signature = {invalid}"))).is_err());
        }
    }
}

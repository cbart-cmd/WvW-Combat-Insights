//! Official weapon-item artwork, bundled offline; provenance in nexusbase/assets/weapons/sources.json.

pub fn icon(weapon: &str) -> Option<&'static [u8]> {
    Some(match weapon {
        "Axe" => include_bytes!("..\\assets\\weapons\\axe.png"),
        "Dagger" => include_bytes!("..\\assets\\weapons\\dagger.png"),
        "Focus" => include_bytes!("..\\assets\\weapons\\focus.png"),
        "Greatsword" => include_bytes!("..\\assets\\weapons\\greatsword.png"),
        "Hammer" => include_bytes!("..\\assets\\weapons\\hammer.png"),
        "Longbow" => include_bytes!("..\\assets\\weapons\\longbow.png"),
        "Mace" => include_bytes!("..\\assets\\weapons\\mace.png"),
        "Pistol" => include_bytes!("..\\assets\\weapons\\pistol.png"),
        "Rifle" => include_bytes!("..\\assets\\weapons\\rifle.png"),
        "Scepter" => include_bytes!("..\\assets\\weapons\\scepter.png"),
        "Shield" => include_bytes!("..\\assets\\weapons\\shield.png"),
        "Shortbow" => include_bytes!("..\\assets\\weapons\\shortbow.png"),
        "Spear" => include_bytes!("..\\assets\\weapons\\spear.png"),
        "Speargun" => include_bytes!("..\\assets\\weapons\\speargun.png"),
        "Staff" => include_bytes!("..\\assets\\weapons\\staff.png"),
        "Sword" => include_bytes!("..\\assets\\weapons\\sword.png"),
        "Torch" => include_bytes!("..\\assets\\weapons\\torch.png"),
        "Trident" => include_bytes!("..\\assets\\weapons\\trident.png"),
        "Warhorn" => include_bytes!("..\\assets\\weapons\\warhorn.png"),
        _ => return None,
    })
}

/// Capability is not evidence of use: a land spear must not be tinted aquatic.
pub fn underwater_capable(weapon: &str) -> bool {
    matches!(weapon, "Spear" | "Speargun" | "Trident")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_weapon_has_official_png_item_artwork() {
        for weapon in [
            "Axe",
            "Dagger",
            "Focus",
            "Greatsword",
            "Hammer",
            "Longbow",
            "Mace",
            "Pistol",
            "Rifle",
            "Scepter",
            "Shield",
            "Shortbow",
            "Spear",
            "Speargun",
            "Staff",
            "Sword",
            "Torch",
            "Trident",
            "Warhorn",
        ] {
            assert!(icon(weapon).unwrap().starts_with(b"\x89PNG\r\n\x1a\n"));
        }
        assert!(icon("unknown").is_none());
        assert!(underwater_capable("Spear"));
        assert!(!underwater_capable("Staff"));
    }
}

use crate::Profiles;

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Individual {
    pub crates: Profiles,
}

impl From<Individual> for Profiles {
    fn from(value: Individual) -> Self {
        value.crates
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::parser::{constants::PATH_YAML_CONFIG, utils};
    use crate::{Profile, ProfileSettings};

    use super::*;

    #[test]
    fn can_read_config() {
        let config = utils::read_profile(PATH_YAML_CONFIG).unwrap();
        let crates = serde_yaml::from_str::<Individual>(&config).unwrap();
        let profiles: Profiles = crates.into();
        assert!(!profiles.is_empty());
    }

    #[test]
    fn can_parse_config() {
        let config = r#"
            crates:
                - name: foo
                  std: false
                  fast-mode: true
                - name: bar
                  patch: |
                      use std::io;
                  inject-cc-flags: true
                - name: baz
                  custom-main: |
                    fn main() {
                        println!("Hello, world!");
                    }
                  run-prover: false
                - name: qux
                  should-fail: true
                  fast-mode: true
        "#;

        let foo = Profile::new(
            "foo".to_string(),
            ProfileSettings {
                std: false,
                fast_mode: true,
                ..Default::default()
            },
            None,
        )
        .unwrap();
        let bar = Profile::new(
            "bar".to_string(),
            ProfileSettings {
                patch: Some("use std::io;\n".to_string()),
                inject_cc_flags: true,
                ..Default::default()
            },
            None,
        )
        .unwrap();
        let baz = Profile::new(
            "baz".to_string(),
            ProfileSettings {
                custom_main: Some(
                    r#"fn main() {
    println!("Hello, world!");
}
"#
                    .to_string(),
                ),
                run_prover: false,
                ..Default::default()
            },
            None,
        )
        .unwrap();
        let qux = Profile::new(
            "qux".to_string(),
            ProfileSettings {
                should_fail: true,
                fast_mode: true,
                ..Default::default()
            },
            None,
        )
        .unwrap();
        let expected_profiles: HashSet<Profile> = [foo, bar, baz, qux].into_iter().collect();

        let batches = serde_yaml::from_str::<Individual>(config).unwrap();
        let profiles: Profiles = batches.into();

        assert_eq!(profiles.len(), 4);
        assert_eq!(expected_profiles, profiles.into_iter().collect());
    }
}

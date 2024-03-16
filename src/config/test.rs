use map_macro::hashbrown::hash_map;

use super::*;

#[test]
fn test_config_parse_empty() {
    let parsed: Config = "
        
    "
    .parse()
    .unwrap();

    let config = Config::default();

    assert_eq!(parsed, config);
}

#[test]
fn test_config_parse_watch() {
    let parsed: Config = "
        [watch]
        enabled = false
        force-poll = true
    "
    .parse()
    .unwrap();

    let config = Config {
        tasks: hash_map!(),
        watch: Watch {
            enabled: false,
            force_poll: true,
        },
    };

    assert_eq!(parsed, config);
}

#[test]
fn test_config_parse_empty_task() {
    let parsed: Config = "
        [task.foo]
    "
    .parse()
    .unwrap();

    let config = Config {
        tasks: hash_map! {
            "foo".to_owned() => Task::default(),
        },
        watch: Watch::default(),
    };

    assert_eq!(parsed, config);
}

#[test]
fn test_config_parse_multi_empty_tasks() {
    let parsed: Config = "
        [task.foo]

        [task.foo.path]

        [task.bar.env]
    "
    .parse()
    .unwrap();

    let config = Config {
        tasks: hash_map! {
            "foo".to_owned() => Task {
                path: Overridable::Custom(Inheritable {
                    config: Path::default(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            "bar".to_owned() => Task {
                env: Overridable::Custom(Inheritable {
                    config: Env::default(),
                    ..Default::default()
                }),
                ..Default::default()
            },
        },
        watch: Watch::default(),
    };

    assert_eq!(parsed, config);
}

#[test]
fn test_config_parse_complex_tasks() {
    let parsed: Config = "
        [task.foo]
        name = 'Foo'

        [task.foo.path]
        dirs = ['/bin']

        [task.foo.env.vars]
        FOO_ENV = 'foo env value'
        BAR_ENV = 'bar env value'

        [task.bar]
        extends = 'foo'
        name = 'Bar'
        cron = '* * * * * *'

        [task.bar.env.vars]
        BAR_ENV = 'overridden bar env'

        [task.baz]
        extends = ['foo', 'bar']
    "
    .parse()
    .unwrap();

    // This is crazy but it's the only time this struct will be used directly like this.
    // It's pretty much just an intermediate struct to make config parsing easy.
    let config = Config {
        tasks: hash_map! {
            "foo".to_owned() => Task {
                config: TaskConfig {
                    name: "Foo".to_owned().into(),
                    ..Default::default()
                },
                path: Overridable::Custom(Inheritable {
                    config: Path {
                        dirs: vec!["/bin".to_owned()],
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                env: Overridable::Custom(Inheritable {
                    config: Env {
                        vars: hash_map! {
                            "FOO_ENV".to_owned() => "foo env value".to_owned(),
                            "BAR_ENV".to_owned() => "bar env value".to_owned(),
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                ..Default::default()
            },

            "bar".to_owned() => Task {
                extends: Some(MultiStr::Single("foo".to_owned())),
                config: TaskConfig {
                    name: Some("Bar".to_owned()),
                    cron: Some("* * * * * *".to_owned()),
                    ..Default::default()
                },
                env: Overridable::Custom(Inheritable {
                    config: Env {
                        vars: hash_map! {
                            "BAR_ENV".to_owned() => "overridden bar env".to_owned(),
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                ..Default::default()
            },

            "baz".to_owned() => Task {
                extends: Some(MultiStr::Multi(vec!["foo".to_owned(), "bar".to_owned()])),
                ..Default::default()
            }
        },
        watch: Watch::default(),
    };

    assert_eq!(parsed, config);
}

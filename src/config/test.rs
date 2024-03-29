use map_macro::hashbrown::hash_map;
use pretty_assertions::assert_eq;

use super::*;

#[test]
fn test_parse_empty() {
    let parsed: Config = "
        
    "
    .parse()
    .unwrap();

    let config = Config::default();

    assert_eq!(parsed, config);
}

#[test]
fn test_parse_watch() {
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
fn test_parse_empty_task() {
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
fn test_parse_multi_empty_tasks() {
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
fn test_parse_complex_tasks() {
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

#[test]
fn test_resolve_empty() {
    let (_, resolved) = "
        
    "
    .parse::<Config>()
    .unwrap()
    .try_into()
    .unwrap();

    let tasks = hash_map!();

    assert_eq!(resolved, tasks)
}

#[test]
fn test_resolve_simple() {
    let (_, resolved) = "
        [task.foo]
        name = 'Foo'

        [task.foo.path]
        dirs = ['/bin']

        [task.foo.env.vars]
        FOO_ENV = 'foo env value'
        BAR_ENV = 'bar env value'
    "
    .parse::<Config>()
    .unwrap()
    .try_into()
    .unwrap();

    let tasks = hash_map! {
        "foo".to_owned() => ResolvedTask {
            config: TaskConfig {
                name: Some("Foo".to_owned()),
                ..Default::default()
            },
            path: Some(Path {
                dirs: vec![rstr("/bin")],
                ..Default::default()
            }),
            env: Some(Env {
                vars: hash_map! {
                    rstr("FOO_ENV") => rstr("foo env value"),
                    rstr("BAR_ENV") => rstr("bar env value"),
                },
                ..Default::default()
            }),
            ..Default::default()
        },
    };

    assert_eq!(resolved, tasks)
}

#[test]
fn test_resolve_complex() {
    let (_, resolved) = "
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
        shell = '/bin/bash'

        [task.bar.env.vars]
        BAR_ENV = 'overridden bar env'
        BAZ_ENV = 'additional env'

        [task.baz.path]
        dirs = ['/usr/bin']

        [task.qoz]
        extends = ['bar', 'baz']
    "
    .parse::<Config>()
    .unwrap()
    .try_into()
    .unwrap();

    let tasks = hash_map! {
        "foo".to_owned() => ResolvedTask {
            config: TaskConfig {
                name: Some("Foo".to_owned()),
                ..Default::default()
            },
            path: Some(Path {
                dirs: vec![rstr("/bin")],
                ..Default::default()
            }),
            env: Some(Env {
                vars: hash_map! {
                    rstr("FOO_ENV") => rstr("foo env value"),
                    rstr("BAR_ENV") => rstr("bar env value"),
                },
                ..Default::default()
            }),
            ..Default::default()
        },

        "bar".to_owned() => ResolvedTask {
            config: TaskConfig {
                name: Some("Bar".to_owned()),
                cron: Some("* * * * * *".to_owned()),
                ..Default::default()
            },
            shell: Some(vec![rstr("/bin/bash")]),
            path: Some(Path {
                dirs: vec![rstr("/bin")],
                ..Default::default()
            }),
            env: Some(Env {
                vars: hash_map! {
                    rstr("FOO_ENV") => rstr("foo env value"),
                    rstr("BAR_ENV") => rstr("overridden bar env"),
                    rstr("BAZ_ENV") => rstr("additional env"),
                },
                ..Default::default()
            }),
            ..Default::default()
        },

        "baz".to_owned() => ResolvedTask {
            path: Some(Path {
                dirs: vec![rstr("/usr/bin")],
                ..Default::default()
            }),
            ..Default::default()
        },

        "qoz".to_owned() => ResolvedTask {
            shell: Some(vec![rstr("/bin/bash")]),
            path: Some(Path {
                dirs: vec![rstr("/usr/bin"), rstr("/bin")],
                ..Default::default()
            }),
            env: Some(Env {
                vars: hash_map! {
                    rstr("FOO_ENV") => rstr("foo env value"),
                    rstr("BAR_ENV") => rstr("overridden bar env"),
                    rstr("BAZ_ENV") => rstr("additional env"),
                },
                ..Default::default()
            }),
            ..Default::default()
        },
    };

    assert_eq!(resolved, tasks)
}

#[test]
fn test_path_merge() {
    let a = Path {
        dirs: vec![rstr("/a"), rstr("/b")],
        apply: PathApplyMethod::Before,
    };

    let b = Path {
        dirs: vec![rstr("/c"), rstr("/d")],
        apply: PathApplyMethod::After,
    };

    let dirs = b
        .dirs
        .iter()
        .cloned()
        .chain(a.dirs.iter().cloned())
        .collect();
    let merged = a.merge(b);

    assert_eq!(
        merged,
        Path {
            dirs,
            apply: PathApplyMethod::After,
        }
    )
}

#[test]
fn test_env_merge() {
    let a = Env {
        vars: hash_map! {
            rstr("A") => rstr("env a"),
            rstr("B") => rstr("env b"),
        },
        merge: false,
    };

    let b = Env {
        vars: hash_map! {
            rstr("B") => rstr("env b overridden"),
            rstr("C") => rstr("env c"),
        },
        merge: true,
    };

    let clone = |(k, v): (&Rstr, &Rstr)| (k.clone(), v.clone());
    let vars: HashMap<_, _> = a
        .vars
        .iter()
        .map(clone)
        .chain(b.vars.iter().map(clone))
        .collect();
    assert_eq!(vars.len(), 3);
    let merged = a.merge(b);

    assert_eq!(merged, Env { vars, merge: true });
}

fn rstr(s: &str) -> Rc<String> {
    Rc::new(s.to_owned())
}

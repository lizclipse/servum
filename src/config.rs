#[cfg(test)]
mod test;

use std::{collections::HashMap, hash::Hash, rc::Rc, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// Task definitions.
    #[serde(default, rename = "task")]
    pub tasks: Vec<Task>,
    /// Config watcher config.
    #[serde(default)]
    pub watch: Watch,
    /// Shell config options.
    ///
    /// Tasks do not use this by default - instead, they are
    /// executed directly unless explicitly set otherwise.
    #[serde(default)]
    pub shell: Global<Shell>,
    /// Path config.
    ///
    /// Tasks will use this by default, but it will default to the
    /// value of the PATH env var.
    #[serde(default = "Global::default_enabled")]
    pub path: Global<Path>,
    /// Env Config.
    ///
    /// Tasks will use this by default, but it will default to the
    /// env vars given to the process.
    #[serde(default = "Global::default_enabled")]
    pub env: Global<Env>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Global<T> {
    /// Whether this global config will be used by tasks by default.
    /// Tasks can individually choose to use or not use the global
    /// settings, and (for some options) can extend them.
    ///
    /// The default for this is per-option.
    #[serde(default)]
    pub use_by_default: bool,
    /// The config option.
    #[serde(flatten, default)]
    pub config: T,
}

impl<T> Global<T>
where
    T: Default,
{
    fn default_enabled() -> Self {
        Self {
            use_by_default: true,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct TaskConfig {
    /// An optional nice name for the task.
    pub name: Option<String>,
    /// A cron string defining when this task should be run.
    pub cron: Option<String>,
    /// The command to run, along with arguments.
    ///
    /// If `shell` is enabled for this task, then this command
    /// will be run with it.
    #[serde(default)]
    pub cmd: Vec<String>,
    /// If enabled, then this task will be run when the process first
    /// starts.
    ///
    /// This is mostly useful for when this service is configured to start
    /// at boot, since it would allow the task to be run on boot.
    ///
    /// Defaults to `false`.
    pub on_start: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Task {
    /// Base config for the task.
    #[serde(flatten)]
    pub config: TaskConfig,
    /// The shell to use for this task.
    ///
    /// Can be a bool to use the globally-configured one or not,
    /// or set to a string/string-array to use a custom one.
    ///
    /// Defaults to the global config.
    #[serde(default)]
    pub shell: Overridable<ShellPath>,
    /// A custom PATH env var for this task.
    ///
    /// Can be a bool to use the globally-configured one or not,
    /// or set to a custom value.
    #[serde(default)]
    pub path: Overridable<Inheritable<Path>>,
    /// A custom env vars for this task.
    ///
    /// Can be a bool to use the globally-configured ones or not,
    /// or set to a custom value.
    #[serde(default)]
    pub env: Overridable<Inheritable<Env>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Overridable<T> {
    #[default]
    Unset,
    Use(bool),
    Custom(T),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Inheritable<T> {
    /// Whether to inerit the global config and extend it.
    ///
    /// Defaults to `false`.
    #[serde(default)]
    pub inherit: bool,
    /// The config option.
    #[serde(flatten, default)]
    pub config: T,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Watch {
    /// Whether the config file should be watched and reloaded upon changes.
    ///
    /// If set to `false`, then any running instances will need to be restarted
    /// to pick up any further changes.
    ///
    /// Defaults to `true`
    #[serde(default = "default_watch_enabled")]
    pub enabled: bool,
    /// Whether to force the usage of the fallback poll-watcher. Mostly as an
    /// escape hatch if the default doesn't work for some reason.
    #[serde(default)]
    pub force_poll: bool,
}

fn default_watch_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Shell<C = ShellPath> {
    /// The shell to use for tasks by default if set, optionally with args.
    /// Either a string or an array of strings can be given.
    ///
    /// Defaults to `/bin/sh`
    #[serde(default)]
    pub cmd: C,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellPath {
    Single(String),
    Multi(Vec<String>),
}

impl Default for ShellPath {
    fn default() -> Self {
        Self::Single("/bin/sh".to_owned().into())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Path<S = String> {
    /// The directories to include in the PATH env var.
    #[serde(default)]
    pub dirs: Vec<S>,
    /// How to apply the set directories to the PATH env var.
    #[serde(default)]
    pub apply: PathApplyMethod,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PathApplyMethod {
    /// Prefixes the given directories to the PATH env var.
    ///
    /// This is the default.
    #[default]
    Before,
    /// Suffixes the given directories to the PATH env var.
    After,
    /// Overrites the PATH env var entirely with the given directories.
    Overwrite,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Env<S = String>
where
    S: Hash + Eq,
{
    /// Environment variables to pass to the command when a task runs.
    ///
    /// # Notes
    ///
    /// If the `PATH` variable is set here, then it will be overridden
    /// by the `path` config (i.e. it will not have an affect).
    /// If the `path` config has not been set, then the variable will
    /// be passed through as normal. or maybe it'll be merged fuck knows
    #[serde(default)]
    pub vars: HashMap<S, S>,
    /// If enabled, then the env vars given here will be merged in with
    /// the ones given to this process.
    ///
    /// Defaults to `true`.
    #[serde(default = "default_env_merge")]
    pub merge: bool,
}

fn default_env_merge() -> bool {
    true
}

// ---------- Impls ----------

type Rstr = Rc<String>;

#[derive(Debug, Clone, Default)]
pub struct ResolvedTask {
    pub config: TaskConfig,
    // TODO: validate that first value resolves to a valid file.
    pub shell: Option<Vec<Rstr>>,
    pub path: Path<Rstr>,
    pub env: Env<Rstr>,
}

impl From<Config> for (Watch, Vec<ResolvedTask>) {
    fn from(
        Config {
            tasks,
            watch,
            shell,
            path,
            env,
        }: Config,
    ) -> Self {
        let shell: Global<Shell<Vec<Rstr>>> = shell.map(|c| c.into());
        let shell_path = shell.map(|s| s.cmd);
        let path: Global<Path<Rstr>> = path.map(|c| c.into());
        let env: Global<Env<Rstr>> = env.map(|c| c.into());

        let tasks = tasks
            .into_iter()
            .map(|task| ResolvedTask {
                config: task.config,
                shell: task
                    .shell
                    .map_custom(|s| s.into())
                    .resolve(&shell_path)
                    .and_then(|s| if s.is_empty() { None } else { Some(s) }),
                path: task
                    .path
                    .map_custom(|p| p.map(|p| p.into()).resolve(&path.config))
                    .resolve(&path)
                    .unwrap_or_default(),
                env: task
                    .env
                    .map_custom(|e| e.map(|e| e.into()).resolve(&env.config))
                    .resolve(&env)
                    .unwrap_or_default(),
            })
            .collect();

        (watch, tasks)
    }
}

impl<T> Global<T> {
    pub fn map<O>(self, f: impl FnOnce(T) -> O) -> Global<O> {
        Global {
            use_by_default: self.use_by_default,
            config: f(self.config),
        }
    }
}

impl From<Shell<ShellPath>> for Shell<Vec<Rstr>> {
    fn from(value: Shell<ShellPath>) -> Self {
        Self {
            cmd: value.cmd.into(),
        }
    }
}

impl From<ShellPath> for Vec<Rstr> {
    fn from(value: ShellPath) -> Self {
        match value {
            ShellPath::Single(v) => vec![Rc::new(v)],
            ShellPath::Multi(vs) => vs.into_iter().map(Rc::new).collect(),
        }
    }
}

pub trait Mergeable {
    fn merge(self, other: Self) -> Self;
}

impl From<Path<String>> for Path<Rstr> {
    fn from(value: Path<String>) -> Self {
        Self {
            dirs: value.dirs.into_iter().map(Rc::new).collect(),
            apply: value.apply,
        }
    }
}

impl Mergeable for Path<Rstr> {
    fn merge(mut self, mut other: Self) -> Self {
        other.dirs.append(&mut self.dirs);
        Self {
            dirs: other.dirs,
            apply: other.apply,
        }
    }
}

impl From<Env<String>> for Env<Rstr> {
    fn from(value: Env<String>) -> Self {
        Self {
            vars: value
                .vars
                .into_iter()
                .map(|(k, v)| (Rc::new(k), Rc::new(v)))
                .collect(),
            merge: value.merge,
        }
    }
}

impl Mergeable for Env<Rstr> {
    fn merge(mut self, other: Self) -> Self {
        self.vars.extend(other.vars.into_iter());
        Self {
            vars: self.vars,
            merge: other.merge,
        }
    }
}

impl<T> Overridable<T>
where
    T: Clone,
{
    pub fn resolve(self, global: &Global<T>) -> Option<T> {
        match self {
            Self::Unset if global.use_by_default => Some(global.config.clone()),
            Self::Use(true) => Some(global.config.clone()),
            Self::Unset | Self::Use(false) => None,
            Self::Custom(v) => Some(v),
        }
    }
}

impl<T> Overridable<T> {
    pub fn map_custom<O>(self, f: impl FnOnce(T) -> O) -> Overridable<O> {
        match self {
            Self::Unset => Overridable::Unset,
            Self::Use(u) => Overridable::Use(u),
            Self::Custom(v) => Overridable::Custom(f(v)),
        }
    }
}

impl<T> Inheritable<T>
where
    T: Mergeable + Clone,
{
    pub fn resolve(self, global: &T) -> T {
        if self.inherit {
            let global = global.clone();
            global.merge(self.config)
        } else {
            self.config
        }
    }
}

impl<T> Inheritable<T> {
    pub fn map<O>(self, f: impl FnOnce(T) -> O) -> Inheritable<O> {
        Inheritable {
            inherit: self.inherit,
            config: f(self.config),
        }
    }
}

impl FromStr for Config {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

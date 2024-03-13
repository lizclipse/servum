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
pub struct Shell<S = String>
where
    S: From<String>,
{
    /// The shell to use for tasks by default if set, optionally with args.
    /// Either a string or an array of strings can be given.
    ///
    /// Defaults to `/bin/sh`
    #[serde(default)]
    pub cmd: ShellPath<S>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellPath<S = String> {
    Single(S),
    Multi(Vec<S>),
}

impl<S> Default for ShellPath<S>
where
    S: From<String>,
{
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

#[derive(Debug, Clone, Default)]
pub struct ResolvedTask {
    pub config: TaskConfig,
    pub shell: Vec<Rc<String>>,
    pub path: Path<Rc<String>>,
    pub env: Env<Rc<String>>,
}

impl From<Config> for (Watch, Vec<ResolvedTask>) {
    fn from(_value: Config) -> Self {
        todo!()
    }
}

impl From<ShellPath> for Vec<Rc<String>> {
    fn from(value: ShellPath) -> Self {
        match value {
            ShellPath::Single(v) => vec![Rc::new(v)],
            ShellPath::Multi(vs) => vs.into_iter().map(Rc::new).collect(),
        }
    }
}

pub trait Mergeable {
    fn merge(&self, other: &Self) -> Self;
}

impl From<Path<String>> for Path<Rc<String>> {
    fn from(value: Path<String>) -> Self {
        Self {
            dirs: value.dirs.into_iter().map(Rc::new).collect(),
            apply: value.apply,
        }
    }
}

impl Mergeable for Path<Rc<String>> {
    fn merge(&self, _other: &Self) -> Self {
        todo!()
    }
}

impl From<Env<String>> for Env<Rc<String>> {
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

impl Mergeable for Env<Rc<String>> {
    fn merge(&self, _other: &Self) -> Self {
        todo!()
    }
}

impl<T> Overridable<T> {}

impl<T> Inheritable<T> where T: Mergeable {}

impl FromStr for Config {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

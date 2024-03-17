#[cfg(test)]
mod test;

use std::{hash::Hash, rc::Rc, str::FromStr};

use color_eyre::eyre;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct Config {
    /// Task definitions.
    #[serde(default, rename = "task")]
    pub tasks: HashMap<String, Task>,
    /// Config watcher config.
    #[serde(default)]
    pub watch: Watch,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct TaskConfig {
    /// An optional nice name for the task.
    pub name: Option<String>,
    /// A cron string defining when this task should be run.
    ///
    /// If the command is currently running, then it will not be run again.
    pub cron: Option<String>,
    /// The command to run, along with arguments.
    ///
    /// If `shell` is enabled for this task, then this command
    /// will be run with it.
    pub cmd: Option<MultiStr>,
    /// The command to use to stop the running process.
    /// This is used for tasks that are intended to be long running
    /// and have a dedicated way to be shut-down.
    ///
    /// If not set, a SIGINT will be attempted to be sent on *nix platforms.
    /// The process will then be killed if any of the following are true:
    ///
    /// - The process does not shutdown within the specified timeout
    /// - The current system is Windows
    /// - The command specified returns a non-zero exit-code
    pub cmd_stop: Option<MultiStr>,
    /// The time (in milliseconds) to wait for the process to stop gracefully.
    /// If set to 0, then:
    ///
    /// - If `cmd_stop` is set, then it will be executed and not waited on
    /// - If not set, then the process will be killed straight away without SIGINT
    ///   being sent first.
    ///
    /// Defaults to 10 seconds (10_000).
    pub stop_timeout: usize,
    /// If enabled, then this task will be run when the process first
    /// starts.
    ///
    /// This is mostly useful for if a task should be run immediately
    /// or in the background.
    ///
    /// Defaults to `false`.
    pub on_start: bool,
    /// Whether the task is enabled.
    /// This is mainly to allow a task to be disabled or stopped without stopping
    /// the main scheduler or removing the task entirely.
    ///
    /// If this task currently has a running process, then it will
    /// be stopped according to the _current_ version of the config.
    ///
    /// Defaults to `true`.
    pub enabled: bool,
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self {
            name: None,
            cron: None,
            cmd: None,
            cmd_stop: None,
            stop_timeout: 10_000,
            on_start: false,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct Task {
    /// Task(s) to extend from.
    pub extends: Option<MultiStr>,
    /// Base config for the task.
    #[serde(flatten)]
    pub config: TaskConfig,
    /// The shell to use for this task.
    /// Can be set to `false` to unset (only applies when extending a task).
    pub shell: Overridable<MultiStr>,
    /// A custom PATH env var for this task.
    /// Can be set to `false` to unset (only applies when extending a task).
    pub path: Overridable<Inheritable<Path>>,
    /// A custom env vars for this task.
    /// Can be set to `false` to unset (only applies when extending a task).
    pub env: Overridable<Inheritable<Env>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Overridable<T> {
    #[default]
    Unset,
    Use(bool),
    Custom(T),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct Inheritable<T> {
    /// Whether to replace this option entirely instead of merging.
    ///
    /// Defaults to `false`.
    pub replace: bool,
    /// The config option.
    #[serde(flatten)]
    pub config: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct Watch {
    /// Whether the config file should be watched and reloaded upon changes.
    ///
    /// If set to `false`, then any running instances will need to be restarted
    /// to pick up any further changes.
    ///
    /// Defaults to `true`
    pub enabled: bool,
    /// Whether to force the usage of the fallback poll-watcher. Mostly as an
    /// escape hatch if the default doesn't work for some reason.
    pub force_poll: bool,
}

impl Default for Watch {
    fn default() -> Self {
        Self {
            enabled: true,
            force_poll: false,
        }
    }
}

/// A simple wrapper to allow either `"single string"` or `["multiple", "strings"]`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MultiStr {
    Single(String),
    Multi(Vec<String>),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct Path<S = String> {
    /// The directories to include in the PATH env var.
    pub dirs: Vec<S>,
    /// How to apply the set directories to the PATH env var.
    pub apply: PathApplyMethod,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
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
    pub vars: HashMap<S, S>,
    /// If enabled, then the env vars given here will be merged in with
    /// the ones given to this process.
    ///
    /// Defaults to `true`.
    pub merge: bool,
}

impl<S> Default for Env<S>
where
    S: Hash + Eq,
{
    fn default() -> Self {
        Self {
            vars: HashMap::default(),
            merge: true,
        }
    }
}

// ---------- Impls ----------

type Rstr = Rc<String>;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedTask {
    pub config: TaskConfig,
    // TODO: validate that first value resolves to a valid file.
    pub shell: Option<Vec<Rstr>>,
    pub path: Option<Path<Rstr>>,
    pub env: Option<Env<Rstr>>,
}

impl TryFrom<Config> for (Watch, HashMap<String, ResolvedTask>) {
    type Error = eyre::Error;

    fn try_from(Config { mut tasks, watch }: Config) -> Result<Self, Self::Error> {
        // Check that all tasks extend from known tasks.
        for task in tasks.values() {
            let Some(extends) = &task.extends else {
                continue;
            };

            match extends {
                MultiStr::Single(e) => {
                    if tasks.get(e).is_none() {
                        eyre::bail!("Unknown task `{}`", e);
                    }
                }
                MultiStr::Multi(es) => {
                    for e in es {
                        if tasks.get(e).is_none() {
                            eyre::bail!("Unknown task `{}`", e);
                        }
                    }
                }
            }
        }

        let mut resolved: HashMap<_, _> = tasks
            .extract_if(|_k, v| v.extends.is_empty())
            .map(|(k, task)| (k, task.into()))
            .collect();

        while !tasks.is_empty() {
            let start_len = tasks.len();
            let mut next = HashMap::new();

            for (id, task) in tasks {
                match resolve_task(task, &resolved) {
                    Ok(task) => {
                        resolved.insert(id, task);
                    }
                    Err(task) => {
                        next.insert(id, task);
                    }
                }
            }

            if next.len() == start_len {
                eyre::bail!("Extends dependency cycle detected");
            }

            tasks = next;
        }

        Ok((watch, resolved))
    }
}

#[allow(clippy::result_large_err)]
fn resolve_task(
    task: Task,
    resolved: &HashMap<String, ResolvedTask>,
) -> Result<ResolvedTask, Task> {
    let mut parents = vec![];

    match &task.extends {
        Some(MultiStr::Single(e)) => match resolved.get(e) {
            Some(p) => parents.push(p),
            None => return Err(task),
        },
        Some(MultiStr::Multi(es)) => {
            for e in es {
                match resolved.get(e) {
                    Some(p) => parents.push(p),
                    None => return Err(task),
                }
            }
        }
        _ => (),
    }

    #[allow(clippy::type_complexity)]
    let (shell, path, env): (Option<Vec<Rstr>>, Option<Path<Rstr>>, Option<Env<Rstr>>) = parents
        .into_iter()
        .fold((None, None, None), |(shell, path, env), p| {
            (
                match (shell, &p.shell) {
                    (Some(shell), None) => Some(shell),
                    (_, Some(shell)) => Some(shell.clone()),
                    _ => None,
                },
                match (path, &p.path) {
                    (Some(path), Some(p_path)) => Some(path.merge(p_path.clone())),
                    (Some(path), None) => Some(path),
                    (None, Some(path)) => Some(path.clone()),
                    _ => None,
                },
                match (env, &p.env) {
                    (Some(env), Some(p_env)) => Some(env.merge(p_env.clone())),
                    (Some(env), None) => Some(env),
                    (None, Some(env)) => Some(env.clone()),
                    _ => None,
                },
            )
        });

    Ok(ResolvedTask {
        config: task.config,
        shell: task.shell.map_custom(Into::into).resolve(shell.as_ref()),
        path: task
            .path
            .map_custom(|p| p.map(Into::into).resolve(path.as_ref()))
            .resolve(path.as_ref()),
        env: task
            .env
            .map_custom(|e| e.map(Into::into).resolve(env.as_ref()))
            .resolve(env.as_ref()),
    })
}

impl From<Task> for ResolvedTask {
    fn from(task: Task) -> Self {
        ResolvedTask {
            config: task.config,
            shell: task.shell.map_custom(Into::into).resolve(None),
            path: task
                .path
                .map_custom(|p| p.map(Into::into).resolve(None))
                .resolve(None),
            env: task
                .env
                .map_custom(|e| e.map(Into::into).resolve(None))
                .resolve(None),
        }
    }
}

trait IsEmpty {
    fn is_empty(&self) -> bool;
}

impl IsEmpty for Option<MultiStr> {
    fn is_empty(&self) -> bool {
        match self {
            None => true,
            Some(MultiStr::Multi(v)) if v.is_empty() => true,
            _ => false,
        }
    }
}

impl From<MultiStr> for Vec<Rstr> {
    fn from(value: MultiStr) -> Self {
        match value {
            MultiStr::Single(v) => vec![Rc::new(v)],
            MultiStr::Multi(vs) => vs.into_iter().map(Rc::new).collect(),
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
        other.dirs.dedup();
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
        self.vars.extend(other.vars);
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
    pub fn resolve(self, parent: Option<&T>) -> Option<T> {
        match (self, parent) {
            (Self::Use(true), Some(parent)) | (Self::Unset, Some(parent)) => Some(parent.clone()),
            (Self::Custom(v), _) => Some(v),
            _ => None,
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
    pub fn resolve(self, parent: Option<&T>) -> T {
        match (self.replace, parent) {
            (false, Some(parent)) => {
                let parent = parent.clone();
                parent.merge(self.config)
            }
            _ => self.config,
        }
    }
}

impl<T> Inheritable<T> {
    pub fn map<O>(self, f: impl FnOnce(T) -> O) -> Inheritable<O> {
        Inheritable {
            replace: self.replace,
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

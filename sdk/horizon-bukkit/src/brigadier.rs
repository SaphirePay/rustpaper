//! Paper's Brigadier commands (`io.papermc.paper.command.brigadier`) and the Folia schedulers.
//!
//! A tree is walked over the words of the line on the plugin's own command, which the game knows as
//! `name [args:text]`, so its completions come from the tree through a tab completer.
use crate::scheduler;
use crate::{CommandSender, Plugin, Player};
use std::collections::BTreeMap;
use std::sync::Arc;

/// What an argument takes.
#[derive(Clone, Debug, PartialEq)]
pub enum ArgumentType {
    /// bukkit: com/mojang/brigadier/arguments/StringArgumentType.word
    Word,
    /// bukkit: com/mojang/brigadier/arguments/StringArgumentType.greedyString
    Greedy,
    /// bukkit: com/mojang/brigadier/arguments/IntegerArgumentType.integer
    Integer { min: i32, max: i32 },
    /// bukkit: com/mojang/brigadier/arguments/DoubleArgumentType.doubleArg
    Double { min: f64, max: f64 },
    /// bukkit: com/mojang/brigadier/arguments/BoolArgumentType.bool
    Bool,
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.player
    Player,
    /// bukkit: io/papermc/paper/command/brigadier/argument/ArgumentTypes.players
    Players,
    /// Any other argument, `fam_paper::argument_types`: over several words if it needs them.
    Custom(crate::fam_paper::CustomArgument),
}

impl ArgumentType {
    pub(crate) fn accepts(&self, word: &str) -> bool {
        match self {
            Self::Word | Self::Greedy | Self::Players => true,
            Self::Integer { min, max } => word.parse::<i32>().is_ok_and(|value| value >= *min && value <= *max),
            Self::Double { min, max } => word.parse::<f64>().is_ok_and(|value| value >= *min && value <= *max),
            Self::Bool => matches!(word, "true" | "false"),
            Self::Player => crate::get_player(word).is_some(),
            Self::Custom(custom) => custom.take(&[word]).is_some(),
        }
    }
    pub(crate) fn suggestions(&self) -> Vec<String> {
        match self {
            Self::Bool => vec!["true".to_owned(), "false".to_owned()],
            Self::Player | Self::Players => crate::get_online_players().iter().map(|player| player.get_name().to_owned()).collect(),
            Self::Custom(custom) => custom.suggestions(),
            _ => Vec::new(),
        }
    }
}

type Run = Arc<dyn Fn(&CommandContext) -> i32 + Send + Sync>;
type Requires = Arc<dyn Fn(&CommandSender) -> bool + Send + Sync>;

/// bukkit: io/papermc/paper/command/brigadier/Commands.literal, io/papermc/paper/command/brigadier/Commands.argument
///
/// A node of a command tree, as Mojang's builders make them.
#[derive(Clone)]
pub struct CommandNode {
    pub(crate) name: String,
    pub(crate) argument: Option<ArgumentType>,
    pub(crate) children: Vec<Self>,
    pub(crate) run: Option<Run>,
    requires: Option<Requires>,
    /// `Commands.restricted`: a click needs the player's confirmation.
    pub(crate) restricted: bool,
}

impl CommandNode {
    /// bukkit: io/papermc/paper/command/brigadier/Commands.literal
    pub fn literal(name: &str) -> Self {
        Self { name: name.to_owned(), argument: None, children: Vec::new(), run: None, requires: None, restricted: false }
    }
    /// bukkit: io/papermc/paper/command/brigadier/Commands.argument
    pub fn argument(name: &str, kind: ArgumentType) -> Self {
        Self { argument: Some(kind), ..Self::literal(name) }
    }
    /// bukkit: com/mojang/brigadier/builder/ArgumentBuilder.then
    pub fn then(mut self, child: Self) -> Self {
        self.children.push(child);
        self
    }
    /// bukkit: com/mojang/brigadier/builder/ArgumentBuilder.executes
    pub fn executes(mut self, run: impl Fn(&CommandContext) -> i32 + Send + Sync + 'static) -> Self {
        self.run = Some(Arc::new(run));
        self
    }
    /// bukkit: com/mojang/brigadier/builder/ArgumentBuilder.requires
    pub fn requires(mut self, test: impl Fn(&CommandSender) -> bool + Send + Sync + 'static) -> Self {
        self.requires = Some(Arc::new(test));
        self
    }
    fn allowed(&self, sender: &CommandSender) -> bool {
        self.requires.as_ref().is_none_or(|test| test(sender))
    }
    fn dispatch(&self, context: &mut CommandContext, words: &[&str]) -> Option<i32> {
        let Some((first, rest)) = words.split_first() else {
            return self.run.as_ref().map(|run| run(context));
        };
        for child in self.children.iter().filter(|child| child.allowed(&context.sender)) {
            match &child.argument {
                None if child.name == *first => return child.dispatch(context, rest),
                Some(ArgumentType::Greedy) => {
                    context.arguments.insert(child.name.clone(), words.join(" "));
                    return child.dispatch(context, &[]);
                }
                Some(ArgumentType::Custom(custom)) => {
                    if let Some(used) = custom.take(words) {
                        context.arguments.insert(child.name.clone(), words[..used].join(" "));
                        return child.dispatch(context, &words[used..]);
                    }
                }
                Some(kind) if kind.accepts(first) => {
                    context.arguments.insert(child.name.clone(), (*first).to_owned());
                    return child.dispatch(context, rest);
                }
                _ => {}
            }
        }
        None
    }
    /// Whether every word finds a node, as `dispatch` walks: the line is then incomplete, and otherwise a word is wrong.
    fn reaches_end(&self, sender: &CommandSender, words: &[&str]) -> bool {
        let Some((first, rest)) = words.split_first() else {
            return true;
        };
        for child in self.children.iter().filter(|child| child.allowed(sender)) {
            match &child.argument {
                None if child.name == *first => return child.reaches_end(sender, rest),
                Some(ArgumentType::Greedy) => return true,
                Some(ArgumentType::Custom(custom)) => {
                    if let Some(used) = custom.take(words) {
                        return child.reaches_end(sender, &words[used..]);
                    }
                }
                Some(kind) if kind.accepts(first) => return child.reaches_end(sender, rest),
                _ => {}
            }
        }
        false
    }
    /// `dispatch` with nothing run: the messages on a line, by argument. The client's tree has no `requires` in it
    /// and the game's hands are full, so no one is refused and a player's name is taken on trust.
    pub(crate) fn signable(&self, words: &[&str], found: &mut Vec<(String, String)>) -> bool {
        let Some((first, rest)) = words.split_first() else {
            return true;
        };
        for child in &self.children {
            match &child.argument {
                None if child.name == *first => return child.signable(rest, found),
                None => {}
                Some(ArgumentType::Greedy) => return child.signable(&[], found),
                Some(ArgumentType::Custom(custom)) => {
                    let used = custom.take(words).unwrap_or(1);
                    if custom.is_message() {
                        found.push((child.name.clone(), words[..used].join(" ")));
                    }
                    return child.signable(&words[used..], found);
                }
                Some(ArgumentType::Player | ArgumentType::Players) => return child.signable(rest, found),
                Some(kind) if kind.accepts(first) => return child.signable(rest, found),
                Some(_) => {}
            }
        }
        false
    }
    /// A signed message somewhere in the tree.
    pub(crate) fn speaks(&self) -> bool {
        self.children.iter().any(|child| matches!(&child.argument, Some(ArgumentType::Custom(custom)) if custom.is_message()) || child.speaks())
    }
    pub(crate) fn complete(&self, sender: &CommandSender, words: &[&str]) -> Vec<String> {
        let Some((first, rest)) = words.split_first() else {
            return Vec::new();
        };
        if rest.is_empty() {
            let options = self.children.iter().filter(|child| child.allowed(sender)).flat_map(|child| match &child.argument {
                None => vec![child.name.clone()],
                Some(kind) => kind.suggestions(),
            });
            return options.filter(|option| crate::extra::starts_with_ignore_case(option, first)).collect();
        }
        self.children
            .iter()
            .filter(|child| child.allowed(sender))
            .flat_map(|child| match &child.argument {
                Some(ArgumentType::Custom(custom)) => custom.take(words).filter(|used| *used < words.len()).map_or_else(Vec::new, |used| child.complete(sender, &words[used..])),
                Some(kind) if kind.accepts(first) => child.complete(sender, rest),
                None if child.name == *first => child.complete(sender, rest),
                _ => Vec::new(),
            })
            .collect()
    }
}

/// bukkit: com/mojang/brigadier/context/CommandContext, io/papermc/paper/command/brigadier/CommandSourceStack
///
/// Who ran a command and what each argument was.
#[derive(Clone)]
pub struct CommandContext {
    sender: CommandSender,
    arguments: BTreeMap<String, String>,
    /// `CommandSourceStack.withLocation` and `withExecutor`, over the sender's own.
    pub(crate) location: Option<crate::Location>,
    pub(crate) executor: Option<crate::fam_paper::CommandExecutor>,
}

impl CommandContext {
    pub(crate) fn of_sender(sender: CommandSender) -> Self { Self { sender, arguments: BTreeMap::new(), location: None, executor: None } }
    /// bukkit: io/papermc/paper/command/brigadier/CommandSourceStack.getSender, com/mojang/brigadier/context/CommandContext.getSource
    pub fn get_sender(&self) -> &CommandSender {
        &self.sender
    }
    /// The player a command runs as, for the many that want no other.
    pub fn get_executor(&self) -> Option<&Player> {
        match &self.executor {
            Some(crate::fam_paper::CommandExecutor::Player(player)) => Some(player),
            Some(crate::fam_paper::CommandExecutor::Entity(_)) => None,
            None => self.sender.as_player(),
        }
    }
    /// bukkit: com/mojang/brigadier/arguments/StringArgumentType.getString, com/mojang/brigadier/context/CommandContext.getArgument
    pub fn get_string(&self, name: &str) -> Option<&str> {
        self.arguments.get(name).map(String::as_str)
    }
    /// bukkit: com/mojang/brigadier/arguments/IntegerArgumentType.getInteger
    pub fn get_integer(&self, name: &str) -> Option<i32> {
        self.get_string(name)?.parse().ok()
    }
    /// bukkit: com/mojang/brigadier/arguments/DoubleArgumentType.getDouble
    pub fn get_double(&self, name: &str) -> Option<f64> {
        self.get_string(name)?.parse().ok()
    }
    /// bukkit: com/mojang/brigadier/arguments/BoolArgumentType.getBool
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        self.get_string(name)?.parse().ok()
    }
    /// bukkit: io/papermc/paper/command/brigadier/argument/resolvers/selector/PlayerSelectorArgumentResolver.resolve
    pub fn get_player(&self, name: &str) -> Option<Player> {
        crate::get_player(self.get_string(name)?)
    }
}

impl Plugin {
    /// bukkit: io/papermc/paper/command/brigadier/Commands.register, io/papermc/paper/plugin/lifecycle/event/LifecycleEventManager.registerEventHandler
    ///
    /// Call it in `on_enable`: the game's command tree is made when the mods are loaded.
    pub fn register_command(&self, node: CommandNode) {
        let name = node.name.clone();
        let tree = Arc::new(node);
        if !crate::fam_ev_world::note_command(self, &name, &tree) {
            // A client signs only what it parses as a message, which `[args:text]` would take first.
            if tree.speaks() {
                crate::add_command_usage(self.host, &name, &name);
                crate::fam_paper::note_signing_tree(self, &tree);
            } else {
                crate::add_command(self.host, &name);
            }
            crate::fam_paper::tell_client(self, &tree);
        }
        run_by(self, &name, tree.clone());
        self.set_tab_completer(&name, move |sender, _, args| tree.complete(sender, args));
    }
}

/// What the word `name` runs: a tree.
pub(crate) fn run_by(plugin: &Plugin, name: &str, runner: Arc<CommandNode>) {
    plugin.set_executor(name, move |sender, _, args| {
        let mut context = CommandContext { sender: sender.clone(), arguments: BTreeMap::new(), location: None, executor: None }.from_source();
        if runner.allowed(sender) && runner.dispatch(&mut context, args).is_none() {
            // The game's own words (command.unknown.command / command.unknown.argument).
            let text = if runner.reaches_end(sender, args) { "Unknown or incomplete command. See below for error" } else { "Incorrect argument for command" };
            sender.send_message(&format!("{}{text}", crate::chat_color::RED));
        }
        true
    });
}

/// `io.papermc.paper.threadedregions.scheduler.GlobalRegionScheduler`, `RegionScheduler` and
/// `EntityScheduler`: one thread runs everything here, so each is the server's own scheduler.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RegionScheduler;

/// bukkit: org/bukkit/Bukkit.getRegionScheduler, org/bukkit/Server.getRegionScheduler
pub fn get_region_scheduler() -> RegionScheduler {
    RegionScheduler
}

impl RegionScheduler {
    /// bukkit: io/papermc/paper/threadedregions/scheduler/RegionScheduler.run
    pub fn run(&self, task: impl FnMut() + Send + 'static) -> u32 {
        scheduler::run_task(task)
    }
    /// bukkit: io/papermc/paper/threadedregions/scheduler/RegionScheduler.execute
    pub fn execute(&self, task: impl FnMut() + Send + 'static) {
        scheduler::run_task(task);
    }
    /// bukkit: io/papermc/paper/threadedregions/scheduler/RegionScheduler.runDelayed
    pub fn run_delayed(&self, delay: u64, task: impl FnMut() + Send + 'static) -> u32 {
        scheduler::run_task_later(delay, task)
    }
    /// bukkit: io/papermc/paper/threadedregions/scheduler/RegionScheduler.runAtFixedRate
    pub fn run_at_fixed_rate(&self, delay: u64, period: u64, task: impl FnMut() + Send + 'static) -> u32 {
        scheduler::run_task_timer(delay, period, task)
    }
    /// One task, by its id.
    pub fn cancel(&self, task: u32) {
        scheduler::cancel_task(task);
    }
}

/// `io.papermc.paper.threadedregions.scheduler.AsyncScheduler`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AsyncScheduler;

/// bukkit: org/bukkit/Bukkit.getAsyncScheduler, org/bukkit/Server.getAsyncScheduler
pub fn get_async_scheduler() -> AsyncScheduler {
    AsyncScheduler
}

impl AsyncScheduler {
    /// bukkit: io/papermc/paper/threadedregions/scheduler/AsyncScheduler.runNow
    pub fn run_now(&self, task: impl FnOnce() + Send + 'static) {
        scheduler::run_task_asynchronously(task);
    }
    /// bukkit: io/papermc/paper/threadedregions/scheduler/AsyncScheduler.runDelayed
    pub fn run_delayed(&self, delay: std::time::Duration, task: impl FnOnce() + Send + 'static) {
        scheduler::run_task_asynchronously(move || {
            std::thread::sleep(delay);
            task();
        });
    }
}

/// bukkit: io/papermc/paper/threadedregions/scheduler/ScheduledTask.cancel
pub fn cancel_scheduled_task(task: u32) {
    scheduler::cancel_task(task);
}

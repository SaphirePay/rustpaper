//! Paper's dialogs: each class is the JSON of the game's own dialog format, and showing one is
//! `/dialog show`, which takes a dialog written out in full.
#![allow(missing_docs)]
use crate::adventure::Component;
use crate::data::snbt;
use crate::{ItemStack, Player};
use horizon_mod::{Value, run_command};
use serde_json::{Map, json};

fn text(component: &Component) -> Value {
    component.0.clone()
}

/// `io.papermc.paper.registry.data.dialog.action.DialogAction`.
#[derive(Clone, Debug, PartialEq)]
pub struct DialogAction(pub Value);

impl DialogAction {
    /// bukkit: io/papermc/paper/registry/data/dialog/action/DialogAction.commandTemplate
    pub fn command_template(template: &str) -> Self {
        Self(json!({ "type": "minecraft:dynamic/run_command", "template": template }))
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/action/DialogAction.staticAction
    ///
    /// `click` is a click event as the game writes it: `{ "action": "run_command", "command": "..." }`.
    pub fn static_action(click: Value) -> Self {
        Self(click)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/action/DialogAction.customClick
    pub fn custom_click(id: &str, additions: Option<Value>) -> Self {
        let mut action = json!({ "type": "minecraft:dynamic/custom", "id": id });
        if let Some(additions) = additions {
            action["additions"] = additions;
        }
        Self(action)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/action/DialogAction$CommandTemplateAction.template
    pub fn template(&self) -> &str {
        self.0["template"].as_str().unwrap_or_default()
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/action/DialogAction$CustomClickAction.id
    pub fn id(&self) -> &str {
        self.0["id"].as_str().unwrap_or_default()
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/action/DialogAction$CustomClickAction.additions
    pub fn additions(&self) -> &Value {
        &self.0["additions"]
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/action/DialogAction$StaticAction.value
    pub fn value(&self) -> &Value {
        &self.0
    }
}

/// `io.papermc.paper.registry.data.dialog.ActionButton`.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionButton(pub Value);

impl ActionButton {
    /// bukkit: io/papermc/paper/registry/data/dialog/ActionButton.create
    pub fn create(label: &Component, tooltip: Option<&Component>, width: i32, action: Option<DialogAction>) -> Self {
        let mut button = Self::builder(label).width(width);
        if let Some(tooltip) = tooltip {
            button = button.tooltip(tooltip);
        }
        if let Some(action) = action {
            button = button.action(action);
        }
        button.build()
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/ActionButton.builder
    pub fn builder(label: &Component) -> ActionButtonBuilder {
        ActionButtonBuilder(json!({ "label": text(label) }))
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/ActionButton.label
    pub fn label(&self) -> &Value {
        &self.0["label"]
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/ActionButton.tooltip
    pub fn tooltip(&self) -> &Value {
        &self.0["tooltip"]
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/ActionButton.width
    pub fn width(&self) -> i64 {
        self.0["width"].as_i64().unwrap_or(150)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/ActionButton.action
    pub fn action(&self) -> Option<DialogAction> {
        self.0.get("action").cloned().map(DialogAction)
    }
}

/// `io.papermc.paper.registry.data.dialog.ActionButton$Builder`.
#[derive(Clone, Debug)]
pub struct ActionButtonBuilder(Value);

impl ActionButtonBuilder {
    /// bukkit: io/papermc/paper/registry/data/dialog/ActionButton$Builder.tooltip
    pub fn tooltip(mut self, tooltip: &Component) -> Self {
        self.0["tooltip"] = text(tooltip);
        self
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/ActionButton$Builder.width
    pub fn width(mut self, width: i32) -> Self {
        self.0["width"] = json!(width);
        self
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/ActionButton$Builder.action
    pub fn action(mut self, action: DialogAction) -> Self {
        self.0["action"] = action.0;
        self
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/ActionButton$Builder.build
    pub fn build(self) -> ActionButton {
        ActionButton(self.0)
    }
}

/// `io.papermc.paper.registry.data.dialog.body.DialogBody`.
#[derive(Clone, Debug, PartialEq)]
pub struct DialogBody(pub Value);

impl DialogBody {
    /// bukkit: io/papermc/paper/registry/data/dialog/body/DialogBody.plainMessage
    pub fn plain_message(contents: &Component, width: Option<i32>) -> Self {
        let mut body = json!({ "type": "minecraft:plain_message", "contents": text(contents) });
        if let Some(width) = width {
            body["width"] = json!(width);
        }
        Self(body)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/body/DialogBody.item, io/papermc/paper/registry/data/dialog/body/ItemDialogBody$Builder.build
    pub fn item(stack: &ItemStack) -> ItemDialogBodyBuilder {
        ItemDialogBodyBuilder(json!({ "type": "minecraft:item", "item": stack.to_json() }))
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/body/PlainMessageDialogBody.contents
    pub fn contents(&self) -> &Value {
        &self.0["contents"]
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/body/PlainMessageDialogBody.width, io/papermc/paper/registry/data/dialog/body/ItemDialogBody.width
    pub fn width(&self) -> i64 {
        self.0["width"].as_i64().unwrap_or(16)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/body/ItemDialogBody.height
    pub fn height(&self) -> i64 {
        self.0["height"].as_i64().unwrap_or(16)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/body/ItemDialogBody.item
    pub fn stack(&self) -> Option<ItemStack> {
        ItemStack::read(&self.0["item"])
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/body/ItemDialogBody.description
    pub fn description(&self) -> &Value {
        &self.0["description"]
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/body/ItemDialogBody.showDecorations
    pub fn show_decorations(&self) -> bool {
        self.0["show_decorations"].as_bool().unwrap_or(true)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/body/ItemDialogBody.showTooltip
    pub fn show_tooltip(&self) -> bool {
        self.0["show_tooltip"].as_bool().unwrap_or(true)
    }
}

/// `io.papermc.paper.registry.data.dialog.body.ItemDialogBody$Builder`.
#[derive(Clone, Debug)]
pub struct ItemDialogBodyBuilder(Value);

impl ItemDialogBodyBuilder {
    /// bukkit: io/papermc/paper/registry/data/dialog/body/ItemDialogBody$Builder.description
    pub fn description(mut self, description: &DialogBody) -> Self {
        self.0["description"] = description.0.clone();
        self
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/body/ItemDialogBody$Builder.showDecorations
    pub fn show_decorations(mut self, show: bool) -> Self {
        self.0["show_decorations"] = json!(show);
        self
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/body/ItemDialogBody$Builder.showTooltip
    pub fn show_tooltip(mut self, show: bool) -> Self {
        self.0["show_tooltip"] = json!(show);
        self
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/body/ItemDialogBody$Builder.width
    pub fn width(mut self, width: i32) -> Self {
        self.0["width"] = json!(width);
        self
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/body/ItemDialogBody$Builder.height
    pub fn height(mut self, height: i32) -> Self {
        self.0["height"] = json!(height);
        self
    }
    pub fn build(self) -> DialogBody {
        DialogBody(self.0)
    }
}

/// `io.papermc.paper.registry.data.dialog.input.DialogInput`, and its four kinds with their builders:
/// a builder's calls set the field, so one type stands for all of them.
#[derive(Clone, Debug, PartialEq)]
pub struct DialogInput(pub Value);

impl DialogInput {
    /// bukkit: io/papermc/paper/registry/data/dialog/input/DialogInput.bool
    pub fn bool(key: &str, label: &Component) -> Self {
        Self(json!({ "type": "minecraft:boolean", "key": key, "label": text(label) }))
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/DialogInput.numberRange
    pub fn number_range(key: &str, label: &Component, start: f32, end: f32) -> Self {
        Self(json!({ "type": "minecraft:number_range", "key": key, "label": text(label), "start": start, "end": end }))
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/DialogInput.singleOption
    pub fn single_option(key: &str, label: &Component, entries: &[OptionEntry]) -> Self {
        let options: Vec<Value> = entries.iter().map(|entry| entry.0.clone()).collect();
        Self(json!({ "type": "minecraft:single_option", "key": key, "label": text(label), "options": options }))
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/DialogInput.text
    pub fn text(key: &str, label: &Component) -> Self {
        Self(json!({ "type": "minecraft:text", "key": key, "label": text(label) }))
    }
    fn with(mut self, field: &str, value: Value) -> Self {
        self.0[field] = value;
        self
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/BooleanDialogInput$Builder.initial, io/papermc/paper/registry/data/dialog/input/NumberRangeDialogInput$Builder.initial, io/papermc/paper/registry/data/dialog/input/TextDialogInput$Builder.initial
    pub fn initial(self, value: impl Into<Value>) -> Self {
        self.with("initial", value.into())
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/BooleanDialogInput$Builder.onTrue
    pub fn on_true(self, value: &str) -> Self {
        self.with("on_true", json!(value))
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/BooleanDialogInput$Builder.onFalse
    pub fn on_false(self, value: &str) -> Self {
        self.with("on_false", json!(value))
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/NumberRangeDialogInput$Builder.width, io/papermc/paper/registry/data/dialog/input/SingleOptionDialogInput$Builder.width, io/papermc/paper/registry/data/dialog/input/TextDialogInput$Builder.width
    pub fn width(self, width: i32) -> Self {
        self.with("width", json!(width))
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/NumberRangeDialogInput$Builder.labelFormat
    pub fn label_format(self, format: &str) -> Self {
        self.with("label_format", json!(format))
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/NumberRangeDialogInput$Builder.step
    pub fn step(self, step: f32) -> Self {
        self.with("step", json!(step))
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/SingleOptionDialogInput$Builder.labelVisible, io/papermc/paper/registry/data/dialog/input/TextDialogInput$Builder.labelVisible
    pub fn label_visible(self, visible: bool) -> Self {
        self.with("label_visible", json!(visible))
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/TextDialogInput$Builder.maxLength
    pub fn max_length(self, length: i32) -> Self {
        self.with("max_length", json!(length))
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/TextDialogInput$Builder.multiline, io/papermc/paper/registry/data/dialog/input/TextDialogInput$MultilineOptions.create
    pub fn multiline(self, max_lines: Option<i32>, height: Option<i32>) -> Self {
        let mut options = json!({});
        if let Some(max_lines) = max_lines {
            options["max_lines"] = json!(max_lines);
        }
        if let Some(height) = height {
            options["height"] = json!(height);
        }
        self.with("multiline", options)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/BooleanDialogInput$Builder.build, io/papermc/paper/registry/data/dialog/input/NumberRangeDialogInput$Builder.build, io/papermc/paper/registry/data/dialog/input/SingleOptionDialogInput$Builder.build, io/papermc/paper/registry/data/dialog/input/TextDialogInput$Builder.build
    pub fn build(self) -> Self {
        self
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/BooleanDialogInput.label, io/papermc/paper/registry/data/dialog/input/NumberRangeDialogInput.label, io/papermc/paper/registry/data/dialog/input/SingleOptionDialogInput.label, io/papermc/paper/registry/data/dialog/input/TextDialogInput.label
    pub fn label(&self) -> &Value {
        &self.0["label"]
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/BooleanDialogInput.initial, io/papermc/paper/registry/data/dialog/input/NumberRangeDialogInput.initial, io/papermc/paper/registry/data/dialog/input/TextDialogInput.initial
    pub fn get_initial(&self) -> &Value {
        &self.0["initial"]
    }
}

/// `io.papermc.paper.registry.data.dialog.input.SingleOptionDialogInput$OptionEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct OptionEntry(pub Value);

impl OptionEntry {
    /// bukkit: io/papermc/paper/registry/data/dialog/input/SingleOptionDialogInput$OptionEntry.create
    pub fn create(id: &str, display: Option<&Component>, initial: bool) -> Self {
        let mut entry = json!({ "id": id, "initial": initial });
        if let Some(display) = display {
            entry["display"] = text(display);
        }
        Self(entry)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/SingleOptionDialogInput$OptionEntry.id
    pub fn id(&self) -> &str {
        self.0["id"].as_str().unwrap_or_default()
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/SingleOptionDialogInput$OptionEntry.display
    pub fn display(&self) -> &Value {
        &self.0["display"]
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/input/SingleOptionDialogInput$OptionEntry.initial
    pub fn initial(&self) -> bool {
        self.0["initial"].as_bool().unwrap_or(false)
    }
}

/// `io.papermc.paper.registry.data.dialog.type.DialogType`: what the buttons are.
#[derive(Clone, Debug, PartialEq)]
pub struct DialogType(pub Value);

impl DialogType {
    /// bukkit: io/papermc/paper/registry/data/dialog/type/DialogType.notice
    pub fn notice(action: Option<&ActionButton>) -> Self {
        let mut kind = json!({ "type": "minecraft:notice" });
        if let Some(action) = action {
            kind["action"] = action.0.clone();
        }
        Self(kind)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/type/DialogType.confirmation
    pub fn confirmation(yes: &ActionButton, no: &ActionButton) -> Self {
        Self(json!({ "type": "minecraft:confirmation", "yes": yes.0, "no": no.0 }))
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/type/DialogType.multiAction, io/papermc/paper/registry/data/dialog/type/MultiActionType$Builder.build
    pub fn multi_action(actions: &[ActionButton], exit: Option<&ActionButton>, columns: i32) -> Self {
        let buttons: Vec<&Value> = actions.iter().map(|button| &button.0).collect();
        let mut kind = json!({ "type": "minecraft:multi_action", "actions": buttons, "columns": columns });
        if let Some(exit) = exit {
            kind["exit_action"] = exit.0.clone();
        }
        Self(kind)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/type/DialogType.serverLinks
    pub fn server_links(exit: Option<&ActionButton>, columns: i32, button_width: i32) -> Self {
        let mut kind = json!({ "type": "minecraft:server_links", "columns": columns, "button_width": button_width });
        if let Some(exit) = exit {
            kind["exit_action"] = exit.0.clone();
        }
        Self(kind)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/type/DialogType.dialogList, io/papermc/paper/registry/data/dialog/type/DialogListType$Builder.build
    pub fn dialog_list(dialogs: &[&str], exit: Option<&ActionButton>, columns: i32, button_width: i32) -> Self {
        let mut kind = json!({ "type": "minecraft:dialog_list", "dialogs": dialogs, "columns": columns, "button_width": button_width });
        if let Some(exit) = exit {
            kind["exit_action"] = exit.0.clone();
        }
        Self(kind)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/type/MultiActionType$Builder.columns, io/papermc/paper/registry/data/dialog/type/DialogListType$Builder.columns
    pub fn columns(mut self, columns: i32) -> Self {
        self.0["columns"] = json!(columns);
        self
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/type/MultiActionType$Builder.exitAction, io/papermc/paper/registry/data/dialog/type/DialogListType$Builder.exitAction
    pub fn exit_action(mut self, action: &ActionButton) -> Self {
        self.0["exit_action"] = action.0.clone();
        self
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/type/DialogListType$Builder.buttonWidth
    pub fn button_width(mut self, width: i32) -> Self {
        self.0["button_width"] = json!(width);
        self
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/type/ConfirmationType.yesButton
    pub fn yes_button(&self) -> Option<ActionButton> {
        self.0.get("yes").cloned().map(ActionButton)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/type/ConfirmationType.noButton
    pub fn no_button(&self) -> Option<ActionButton> {
        self.0.get("no").cloned().map(ActionButton)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/type/MultiActionType.actions
    pub fn actions(&self) -> Vec<ActionButton> {
        self.0["actions"].as_array().into_iter().flatten().cloned().map(ActionButton).collect()
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/type/MultiActionType.exitAction, io/papermc/paper/registry/data/dialog/type/DialogListType.exitAction, io/papermc/paper/registry/data/dialog/type/ServerLinksType.exitAction
    pub fn get_exit_action(&self) -> Option<ActionButton> {
        self.0.get("exit_action").cloned().map(ActionButton)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/type/MultiActionType.columns, io/papermc/paper/registry/data/dialog/type/DialogListType.columns, io/papermc/paper/registry/data/dialog/type/ServerLinksType.columns
    pub fn get_columns(&self) -> i64 {
        self.0["columns"].as_i64().unwrap_or(2)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/type/DialogListType.buttonWidth, io/papermc/paper/registry/data/dialog/type/ServerLinksType.buttonWidth
    pub fn get_button_width(&self) -> i64 {
        self.0["button_width"].as_i64().unwrap_or(150)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/type/NoticeType.action
    pub fn action(&self) -> Option<ActionButton> {
        self.0.get("action").cloned().map(ActionButton)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/type/DialogListType.dialogs
    pub fn dialogs(&self) -> Vec<String> {
        self.0["dialogs"].as_array().into_iter().flatten().filter_map(|id| id.as_str().map(str::to_owned)).collect()
    }
}

/// `io.papermc.paper.registry.data.dialog.DialogBase`: what every dialog has.
#[derive(Clone, Debug, PartialEq)]
pub struct DialogBase(pub Map<String, Value>);

impl DialogBase {
    /// bukkit: io/papermc/paper/registry/data/dialog/DialogBase.builder, io/papermc/paper/registry/data/dialog/DialogBase.create, io/papermc/paper/registry/data/dialog/DialogBase$Builder.build
    pub fn builder(title: &Component) -> Self {
        let mut base = Map::new();
        base.insert("title".to_owned(), text(title));
        Self(base)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/DialogBase$Builder.externalTitle
    pub fn external_title(mut self, title: &Component) -> Self {
        self.0.insert("external_title".to_owned(), text(title));
        self
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/DialogBase$Builder.canCloseWithEscape
    pub fn can_close_with_escape(mut self, can: bool) -> Self {
        self.0.insert("can_close_with_escape".to_owned(), json!(can));
        self
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/DialogBase$Builder.pause
    pub fn pause(mut self, pause: bool) -> Self {
        self.0.insert("pause".to_owned(), json!(pause));
        self
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/DialogBase$Builder.afterAction
    ///
    /// `action` is a name of `DialogAfterAction`: `CLOSE`, `NONE` or `WAIT_FOR_RESPONSE`.
    pub fn after_action(mut self, action: &str) -> Self {
        self.0.insert("after_action".to_owned(), json!(action.to_ascii_lowercase()));
        self
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/DialogBase$Builder.body
    pub fn body(mut self, body: &[DialogBody]) -> Self {
        let rows: Vec<&Value> = body.iter().map(|row| &row.0).collect();
        self.0.insert("body".to_owned(), json!(rows));
        self
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/DialogBase$Builder.inputs
    pub fn inputs(mut self, inputs: &[DialogInput]) -> Self {
        let rows: Vec<&Value> = inputs.iter().map(|row| &row.0).collect();
        self.0.insert("inputs".to_owned(), json!(rows));
        self
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/DialogBase.title
    pub fn title(&self) -> Option<&Value> {
        self.0.get("title")
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/DialogBase.externalTitle
    pub fn get_external_title(&self) -> Option<&Value> {
        self.0.get("external_title")
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/DialogBase.canCloseWithEscape
    pub fn get_can_close_with_escape(&self) -> bool {
        self.0.get("can_close_with_escape").and_then(Value::as_bool).unwrap_or(true)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/DialogBase.pause
    pub fn get_pause(&self) -> bool {
        self.0.get("pause").and_then(Value::as_bool).unwrap_or(true)
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/DialogBase.afterAction
    pub fn get_after_action(&self) -> &str {
        self.0.get("after_action").and_then(Value::as_str).unwrap_or("close")
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/DialogBase.body
    pub fn get_body(&self) -> Vec<DialogBody> {
        self.0.get("body").and_then(Value::as_array).into_iter().flatten().cloned().map(DialogBody).collect()
    }
    /// bukkit: io/papermc/paper/registry/data/dialog/DialogBase.inputs
    pub fn get_inputs(&self) -> Vec<DialogInput> {
        self.0.get("inputs").and_then(Value::as_array).into_iter().flatten().cloned().map(DialogInput).collect()
    }
}

/// `io.papermc.paper.registry.data.dialog.DialogRegistryEntry`, and `io.papermc.paper.dialog.Dialog`:
/// a base and a type, one JSON.
#[derive(Clone, Debug, PartialEq)]
pub struct Dialog(pub Value);

impl Dialog {
    /// bukkit: io/papermc/paper/registry/data/dialog/DialogRegistryEntry$Builder.base, io/papermc/paper/registry/data/dialog/DialogRegistryEntry$Builder.type, io/papermc/paper/registry/data/dialog/DialogRegistryEntry.base, io/papermc/paper/registry/data/dialog/DialogRegistryEntry.type, io/papermc/paper/registry/data/dialog/DialogRegistryEntry$Builder.registryValueSet
    pub fn create(base: DialogBase, kind: DialogType) -> Self {
        let mut dialog = kind.0;
        for (key, value) in base.0 {
            dialog[key] = value;
        }
        Self(dialog)
    }
    /// bukkit~: org/bukkit/entity/Player.showDialog | sent to the player by name, as `/dialog show` does
    pub fn show_to(&self, player: &Player) {
        run_command(&format!("dialog show {} {}", player.get_name(), snbt(&self.0)));
    }
}

/// bukkit: org/bukkit/entity/Player.closeDialog
pub fn close_dialog(player: &Player) {
    run_command(&format!("dialog clear {}", player.get_name()));
}

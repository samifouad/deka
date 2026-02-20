use core::{CommandSpec, Context, Registry};
use serde_json::Value;
use std::collections::BTreeMap;
use stdio::{raw, warn_simple};

const COMMAND: CommandSpec = CommandSpec {
    name: "task",
    category: "runtime",
    summary: "run a task from deka.json",
    aliases: &[],
    subcommands: &[],
    handler: cmd,
};

pub fn register(registry: &mut Registry) {
    registry.add_command(COMMAND);
}

pub fn cmd(context: &Context) {
    let Some(json) = load_deka_json() else {
        stdio::error("task", "deka.json not found in current directory");
        return;
    };

    let (tasks, used_scripts) = extract_tasks(&json);
    if used_scripts {
        warn_simple("scripts is deprecated for tasks; prefer tasks in deka.json");
    }

    let task_name = requested_task_name(context);
    if task_name.is_none() {
        print_task_list(&tasks);
        return;
    }

    let task_name = task_name.unwrap();
    if !tasks.contains_key(task_name) {
        stdio::error("task", &format!("unknown task `{}`", task_name));
        print_task_list(&tasks);
        return;
    }

    stdio::error(
        "task",
        "task execution is not implemented yet (see tasks/DEKA-TASK-RUNNER.md Task 2)",
    );
}

fn requested_task_name<'a>(context: &'a Context) -> Option<&'a str> {
    if let Some(first) = context.args.positionals.first() {
        return Some(first.as_str());
    }
    if context.args.commands.len() > 1 {
        return Some(context.args.commands[1].as_str());
    }
    None
}

#[derive(Debug, Clone)]
struct TaskDef {
    command: String,
    description: Option<String>,
    dependencies: Vec<String>,
}

fn load_deka_json() -> Option<Value> {
    let cwd = std::env::current_dir().ok()?;
    let path = cwd.join("deka.json");
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn extract_tasks(json: &Value) -> (BTreeMap<String, TaskDef>, bool) {
    let tasks = json
        .get("tasks")
        .and_then(|value| value.as_object());
    let (source, used_scripts) = if tasks.is_some() {
        (tasks, false)
    } else {
        (
            json.get("scripts").and_then(|value| value.as_object()),
            true,
        )
    };

    let mut out = BTreeMap::new();
    let Some(source) = source else {
        return (out, used_scripts);
    };

    for (name, value) in source {
        if let Some(command) = value.as_str() {
            out.insert(
                name.to_string(),
                TaskDef {
                    command: command.to_string(),
                    description: None,
                    dependencies: Vec::new(),
                },
            );
            continue;
        }

        let Some(obj) = value.as_object() else {
            warn_simple(&format!(
                "task `{}` ignored: expected string or object",
                name
            ));
            continue;
        };

        let command = obj
            .get("command")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let Some(command) = command else {
            warn_simple(&format!(
                "task `{}` ignored: missing required `command`",
                name
            ));
            continue;
        };

        let description = obj
            .get("description")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());
        let dependencies = obj
            .get("dependencies")
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(|value| value.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        out.insert(
            name.to_string(),
            TaskDef {
                command,
                description,
                dependencies,
            },
        );
    }

    (out, used_scripts)
}

fn print_task_list(tasks: &BTreeMap<String, TaskDef>) {
    if tasks.is_empty() {
        stdio::error("task", "no tasks found in deka.json");
        return;
    }

    raw("Tasks:");
    for (name, task) in tasks {
        let mut line = String::new();
        line.push_str("  ");
        line.push_str(name);
        if let Some(desc) = task.description.as_deref() {
            line.push_str("  - ");
            line.push_str(desc);
        }
        if !task.dependencies.is_empty() {
            line.push_str("  (deps: ");
            line.push_str(&task.dependencies.join(", "));
            line.push(')');
        }
        if !task.command.is_empty() {
            line.push_str("  -> ");
            line.push_str(&task.command);
        }
        raw(&line);
    }
}

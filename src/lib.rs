use zed::{http_client::HttpMethod, http_client::HttpRequest};
use zed_extension_api::{
    self as zed, http_client::RedirectPolicy, Result, SlashCommand, SlashCommandArgumentCompletion,
    SlashCommandOutput, SlashCommandOutputSection, Worktree,
};

const FABRIC_URL: &str =
    "https://raw.githubusercontent.com/danielmiessler/fabric/refs/heads/main/patterns/{}/system.md";

struct SlashCommandsExampleExtension;

impl zed::Extension for SlashCommandsExampleExtension {
    fn new() -> Self {
        SlashCommandsExampleExtension
    }

    fn complete_slash_command_argument(
        &self,
        command: SlashCommand,
        _args: Vec<String>,
    ) -> Result<Vec<zed_extension_api::SlashCommandArgumentCompletion>, String> {
        match command.name.as_str() {
            "echo" | "github" | "fabric" => Ok(vec![]),
            "pick-one" => Ok(vec![
                SlashCommandArgumentCompletion {
                    label: "Option One".to_string(),
                    new_text: "option-1".to_string(),
                    run_command: true,
                },
                SlashCommandArgumentCompletion {
                    label: "Option Two".to_string(),
                    new_text: "option-2".to_string(),
                    run_command: true,
                },
                SlashCommandArgumentCompletion {
                    label: "Option Three".to_string(),
                    new_text: "option-3".to_string(),
                    run_command: true,
                },
            ]),
            command => Err(format!("unknown slash command: \"{command}\"")),
        }
    }

    fn run_slash_command(
        &self,
        command: SlashCommand,
        args: Vec<String>,
        _worktree: Option<&Worktree>,
    ) -> Result<SlashCommandOutput, String> {
        match command.name.as_str() {
            "github" => self.handle_github_command(args),
            "fabric" => self.handle_fabric_command(args),
            "echo" => self.handle_echo_command(args),
            "pick-one" => self.handle_pick_one_command(args),
            command => Err(format!("unknown slash command: \"{command}\"")),
        }
    }
}

impl SlashCommandsExampleExtension {
    fn download_file(&self, url: &str) -> Result<String, String> {
        let request = HttpRequest {
            method: HttpMethod::Get,
            url: url.to_string(),
            headers: vec![("User-Agent".to_string(), "Zed-Extension".to_string())],
            body: None,
            redirect_policy: RedirectPolicy::FollowAll,
        };

        match zed::http_client::fetch(&request) {
            Ok(response) => Ok(String::from_utf8_lossy(&response.body).to_string()),
            Err(e) => Err(format!("API request failed. Error: {}.", e)),
        }
    }

    fn handle_fabric_command(&self, args: Vec<String>) -> Result<SlashCommandOutput, String> {
        if args.is_empty() {
            return Err("Need to provide a pattern name".to_string());
        }
        let pattern = args.join(" ");
        let url = FABRIC_URL.replace("{}", &pattern);

        match self.download_file(&url) {
            Ok(content) => Ok(zed::SlashCommandOutput {
                text: content,
                sections: vec![SlashCommandOutputSection {
                    range: (0..url.len()).into(),
                    label: "Fabric".to_string(),
                }],
            }),
            Err(e) => Ok(zed::SlashCommandOutput {
                text: e,
                sections: vec![],
            }),
        }
    }

    fn handle_github_command(&self, args: Vec<String>) -> Result<SlashCommandOutput, String> {
        if args.is_empty() {
            return Err("Need to provide a repo path".to_string());
        }
        let text = args.join(" ");
        let url = format!("https://github.com/{}", text);

        match self.download_file(&url) {
            Ok(content) => Ok(zed::SlashCommandOutput {
                text: content,
                sections: vec![SlashCommandOutputSection {
                    range: (0..url.len()).into(),
                    label: "GitHub".to_string(),
                }],
            }),
            Err(e) => Ok(zed::SlashCommandOutput {
                text: e,
                sections: vec![],
            }),
        }
    }

    fn handle_echo_command(&self, args: Vec<String>) -> Result<SlashCommandOutput, String> {
        if args.is_empty() {
            return Err("nothing to echo".to_string());
        }

        let text = args.join(" ");

        Ok(SlashCommandOutput {
            sections: vec![SlashCommandOutputSection {
                range: (0..text.len()).into(),
                label: "Echo".to_string(),
            }],
            text,
        })
    }

    fn handle_pick_one_command(&self, args: Vec<String>) -> Result<SlashCommandOutput, String> {
        let Some(selection) = args.first() else {
            return Err("no option selected".to_string());
        };

        if !["option-1", "option-2", "option-3"].contains(&selection.as_str()) {
            return Err(format!("{} is not a valid option", selection));
        }

        let text = format!("You chose {}.", selection);

        Ok(SlashCommandOutput {
            sections: vec![SlashCommandOutputSection {
                range: (0..text.len()).into(),
                label: format!("Pick One: {}", selection),
            }],
            text,
        })
    }
}

zed::register_extension!(SlashCommandsExampleExtension);

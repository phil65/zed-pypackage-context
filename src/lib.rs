use zed::{http_client::HttpMethod, http_client::HttpRequest};
use zed_extension_api::{
    self as zed, http_client::RedirectPolicy, Result, SlashCommand, SlashCommandOutput,
    SlashCommandOutputSection, Worktree,
};

const PYPI_API_URL: &str = "https://pypi.org/pypi/{}/json";

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
            "github" | "pypi" => Ok(vec![]),
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
            "pypi" => self.handle_pypi_command(args),
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

    fn handle_github_command(&self, args: Vec<String>) -> Result<SlashCommandOutput, String> {
        if args.is_empty() {
            return Err("Need to provide a repo path".to_string());
        }
        let text = args.join(" ");
        let url = format!("https://uithub.com/{}", text);

        match self.download_file(&url) {
            Ok(content) => {
                let content_len = content.len();
                Ok(zed::SlashCommandOutput {
                    text: content.clone(),
                    sections: vec![SlashCommandOutputSection {
                        range: (0..content_len).into(),
                        label: format!("GitHub: {}", text),
                    }],
                })
            }
            Err(e) => Ok(zed::SlashCommandOutput {
                text: e,
                sections: vec![],
            }),
        }
    }
    fn handle_pypi_command(&self, args: Vec<String>) -> Result<SlashCommandOutput, String> {
        if args.is_empty() {
            return Err("Need to provide a package name".to_string());
        }
        let package_name = args.join("-");
        let url = PYPI_API_URL.replace("{}", &package_name);

        match self.download_file(&url) {
            Ok(content) => {
                let json: serde_json::Value = serde_json::from_str(&content)
                    .map_err(|e| format!("Failed to parse JSON: {}", e))?;
                let repo_url = json["info"]["project_urls"]["Source"]
                    .as_str()
                    .or_else(|| json["info"]["project_urls"]["Repository"].as_str())
                    .or_else(|| json["info"]["project_urls"]["Source Code"].as_str())
                    .ok_or_else(|| "GitHub URL not found".to_string())?;
                let uithub_url = repo_url.replace("github", "uithub");

                match self.download_file(&uithub_url) {
                    Ok(content) => Ok(zed::SlashCommandOutput {
                        text: content.clone(),
                        sections: vec![SlashCommandOutputSection {
                            range: (0..content.len()).into(),
                            label: format!("PyPI: {}", package_name),
                        }],
                    }),
                    Err(e) => Ok(zed::SlashCommandOutput {
                        text: e,
                        sections: vec![],
                    }),
                }
            }
            Err(e) => Ok(zed::SlashCommandOutput {
                text: e,
                sections: vec![],
            }),
        }
    }
}

zed::register_extension!(SlashCommandsExampleExtension);

use eframe::egui;
use egui_phosphor::regular::{
    ARROW_LEFT, ARROW_RIGHT, CHECK_CIRCLE, ENVELOPE, GITHUB_LOGO, KEY, LOCK, SHIELD_CHECK, USER,
    WARNING_CIRCLE,
};

use crate::auth::git_identity::{GhCliStatus, GitIdentity, GpgKeyInfo, SshKeyInfo};
use crate::auth::github_oauth::GitHubUser;

#[derive(Default)]
pub struct SetupWizardState {
    pub step: WizardStep,
    pub git_name: String,
    pub git_email: String,
    pub identity_detected: bool,
    pub identity: Option<GitIdentity>,
    pub gh_cli_status: Option<GhCliStatus>,
    pub ssh_keys: Vec<SshKeyInfo>,
    pub gpg_keys: Vec<GpgKeyInfo>,
    pub device_code_response: Option<DeviceFlowUiState>,
    pub auth_polling: bool,
    pub auth_error: Option<String>,
    pub github_user: Option<GitHubUser>,
    pub detection_started: bool,
}

#[derive(Default, PartialEq, Clone)]
pub enum WizardStep {
    #[default]
    GitIdentity,
    SshGpgKeys,
    GitHubAuth,
    Done,
}

#[derive(Clone)]
pub struct DeviceFlowUiState {
    pub user_code: String,
    pub verification_uri: String,
}

pub enum WizardAction {
    None,
    StartDetection,
    StartDeviceFlow,
    OpenVerificationUrl(String),
    Complete { git_name: String, git_email: String },
    Skip,
}

const STEP_COUNT: usize = 4;

pub fn show(ui: &mut egui::Ui, state: &mut SetupWizardState) -> WizardAction {
    let mut action = WizardAction::None;

    egui::Frame::NONE
        .inner_margin(egui::Margin::symmetric(24, 16))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 6.0);

            match state.step.clone() {
                WizardStep::GitIdentity => {
                    action = render_git_identity_step(ui, state);
                }
                WizardStep::SshGpgKeys => {
                    action = render_ssh_gpg_step(ui, state);
                }
                WizardStep::GitHubAuth => {
                    action = render_github_auth_step(ui, state);
                }
                WizardStep::Done => {
                    action = render_done_step(ui, state);
                }
            }

            ui.add_space(8.0);
            render_step_indicator(ui, &state.step);
        });

    action
}

fn render_step_indicator(ui: &mut egui::Ui, current_step: &WizardStep) {
    let active_index = match current_step {
        WizardStep::GitIdentity => 0,
        WizardStep::SshGpgKeys => 1,
        WizardStep::GitHubAuth => 2,
        WizardStep::Done => 3,
    };

    ui.horizontal(|ui| {
        let total_dot_width = STEP_COUNT as f32 * 12.0 + (STEP_COUNT as f32 - 1.0) * 6.0;
        let offset = (ui.available_width() - total_dot_width) / 2.0;
        ui.add_space(offset);

        for step_index in 0..STEP_COUNT {
            let radius = 4.0;
            let (rect, _response) =
                ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
            let center = rect.center();

            if step_index == active_index {
                ui.painter()
                    .circle_filled(center, radius, egui::Color32::from_rgb(28, 145, 220));
            } else if step_index < active_index {
                ui.painter()
                    .circle_filled(center, radius, egui::Color32::from_rgb(80, 180, 80));
            } else {
                ui.painter()
                    .circle_filled(center, radius, egui::Color32::from_rgb(80, 80, 80));
            }
        }
    });
}

fn render_git_identity_step(ui: &mut egui::Ui, state: &mut SetupWizardState) -> WizardAction {
    let mut action = WizardAction::None;

    if !state.detection_started {
        action = WizardAction::StartDetection;
    }

    ui.add_space(12.0);
    ui.label(
        egui::RichText::new("👋 Welcome to Palimpsest")
            .size(20.0)
            .strong(),
    );
    ui.label(
        egui::RichText::new("Let's set up your Git identity")
            .size(13.0)
            .color(egui::Color32::from_rgb(165, 165, 165)),
    );

    ui.add_space(16.0);

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(USER).size(14.0));
        ui.label(egui::RichText::new("Name").size(12.0));
    });
    ui.add_sized(
        [ui.available_width(), 24.0],
        egui::TextEdit::singleline(&mut state.git_name)
            .hint_text("Your name (e.g. Jane Doe)")
            .margin(egui::Margin::symmetric(6, 4)),
    );

    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(ENVELOPE).size(14.0));
        ui.label(egui::RichText::new("Email").size(12.0));
    });
    ui.add_sized(
        [ui.available_width(), 24.0],
        egui::TextEdit::singleline(&mut state.git_email)
            .hint_text("your@email.com")
            .margin(egui::Margin::symmetric(6, 4)),
    );

    ui.add_space(8.0);

    if let Some(ref cli_status) = state.gh_cli_status {
        ui.horizontal(|ui| {
            if cli_status.logged_in {
                ui.label(
                    egui::RichText::new(CHECK_CIRCLE)
                        .size(14.0)
                        .color(egui::Color32::from_rgb(80, 180, 80)),
                );
                let status_text = match &cli_status.username {
                    Some(username) => format!("gh CLI authenticated as {}", username),
                    None => "gh CLI authenticated".to_string(),
                };
                ui.label(egui::RichText::new(status_text).size(12.0));
            } else {
                ui.label(
                    egui::RichText::new(WARNING_CIRCLE)
                        .size(14.0)
                        .color(egui::Color32::from_rgb(200, 80, 80)),
                );
                ui.label(egui::RichText::new("gh CLI not authenticated").size(12.0));
            }
        });
    }

    ui.add_space(12.0);

    ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
        if ui
            .button(egui::RichText::new(format!("Next {}", ARROW_RIGHT)).size(12.0))
            .clicked()
        {
            state.step = WizardStep::SshGpgKeys;
        }
    });

    action
}

fn render_ssh_gpg_step(ui: &mut egui::Ui, state: &mut SetupWizardState) -> WizardAction {
    ui.add_space(12.0);
    ui.label(
        egui::RichText::new("🔒 Security Configuration")
            .size(20.0)
            .strong(),
    );
    ui.label(
        egui::RichText::new("Detected keys on your system")
            .size(13.0)
            .color(egui::Color32::from_rgb(165, 165, 165)),
    );

    ui.add_space(16.0);

    // SSH Keys section
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(KEY).size(14.0));
        ui.label(egui::RichText::new("SSH Keys").size(13.0).strong());
    });

    if state.ssh_keys.is_empty() {
        ui.label(
            egui::RichText::new("  No SSH keys detected")
                .size(12.0)
                .color(egui::Color32::from_rgb(140, 140, 140)),
        );
    } else {
        for ssh_key in &state.ssh_keys {
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new(CHECK_CIRCLE)
                        .size(12.0)
                        .color(egui::Color32::from_rgb(80, 180, 80)),
                );
                let key_label = format!("{} ({})", ssh_key.path, ssh_key.key_type);
                ui.label(egui::RichText::new(key_label).size(12.0));
            });
        }
    }

    ui.add_space(10.0);

    // GPG Keys section
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(SHIELD_CHECK).size(14.0));
        ui.label(egui::RichText::new("GPG Keys").size(13.0).strong());
    });

    if state.gpg_keys.is_empty() {
        ui.label(
            egui::RichText::new("  No GPG keys detected")
                .size(12.0)
                .color(egui::Color32::from_rgb(140, 140, 140)),
        );
    } else {
        for gpg_key in &state.gpg_keys {
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new(LOCK)
                        .size(12.0)
                        .color(egui::Color32::from_rgb(80, 180, 80)),
                );
                let key_label = format!("{} ({})", gpg_key.key_id, gpg_key.uid);
                ui.label(egui::RichText::new(key_label).size(12.0));
            });
        }
    }

    ui.add_space(20.0);

    ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
        if ui
            .button(egui::RichText::new(format!("Next {}", ARROW_RIGHT)).size(12.0))
            .clicked()
        {
            state.step = WizardStep::GitHubAuth;
        }
        if ui
            .button(egui::RichText::new(format!("{} Back", ARROW_LEFT)).size(12.0))
            .clicked()
        {
            state.step = WizardStep::GitIdentity;
        }
    });

    WizardAction::None
}

fn render_github_auth_step(ui: &mut egui::Ui, state: &mut SetupWizardState) -> WizardAction {
    let mut action = WizardAction::None;

    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(GITHUB_LOGO).size(22.0));
        ui.label(egui::RichText::new("Connect to GitHub").size(20.0).strong());
    });

    ui.add_space(12.0);

    if let Some(ref github_user) = state.github_user {
        // Successfully connected
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(CHECK_CIRCLE)
                    .size(16.0)
                    .color(egui::Color32::from_rgb(80, 180, 80)),
            );
            ui.label(
                egui::RichText::new(format!("Connected as {}", github_user.login))
                    .size(14.0)
                    .color(egui::Color32::from_rgb(80, 180, 80)),
            );
        });
    } else if let Some(ref error_message) = state.auth_error {
        // Error state
        ui.label(
            egui::RichText::new(format!("{} {}", WARNING_CIRCLE, error_message))
                .size(12.0)
                .color(egui::Color32::from_rgb(220, 80, 80)),
        );
        ui.add_space(8.0);
        if ui
            .button(egui::RichText::new(format!("{} Try Again", GITHUB_LOGO)).size(12.0))
            .clicked()
        {
            state.auth_error = None;
            action = WizardAction::StartDeviceFlow;
        }
    } else if let Some(ref device_flow_state) = state.device_code_response.clone() {
        // Device flow active — show code and verification URL
        ui.label(
            egui::RichText::new("Enter the code below at GitHub:")
                .size(13.0)
                .color(egui::Color32::from_rgb(165, 165, 165)),
        );

        ui.add_space(12.0);

        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new(&device_flow_state.user_code)
                    .size(28.0)
                    .strong()
                    .monospace(),
            );
        });

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Open:").size(12.0));
            if ui
                .link(egui::RichText::new(&device_flow_state.verification_uri).size(12.0))
                .clicked()
            {
                action =
                    WizardAction::OpenVerificationUrl(device_flow_state.verification_uri.clone());
            }
        });

        if state.auth_polling {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(
                    egui::RichText::new("Waiting for authorization...")
                        .size(12.0)
                        .color(egui::Color32::from_rgb(165, 165, 165)),
                );
            });
        }
    } else {
        // Initial state — prompt to connect
        ui.label(
            egui::RichText::new("Connect your GitHub account for remote features")
                .size(13.0)
                .color(egui::Color32::from_rgb(165, 165, 165)),
        );

        ui.add_space(16.0);

        ui.vertical_centered(|ui| {
            if ui
                .button(
                    egui::RichText::new(format!("{}  Connect to GitHub", GITHUB_LOGO)).size(13.0),
                )
                .clicked()
            {
                action = WizardAction::StartDeviceFlow;
            }
        });
    }

    ui.add_space(16.0);

    ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
        let finish_label = if state.github_user.is_some() {
            "Finish"
        } else {
            "Next"
        };
        if ui
            .button(egui::RichText::new(format!("{} {}", finish_label, ARROW_RIGHT)).size(12.0))
            .clicked()
        {
            state.step = WizardStep::Done;
        }
        if ui.button(egui::RichText::new("Skip").size(12.0)).clicked() {
            action = WizardAction::Skip;
        }
        if ui
            .button(egui::RichText::new(format!("{} Back", ARROW_LEFT)).size(12.0))
            .clicked()
        {
            state.step = WizardStep::SshGpgKeys;
        }
    });

    action
}

fn render_done_step(ui: &mut egui::Ui, state: &mut SetupWizardState) -> WizardAction {
    let mut action = WizardAction::None;

    ui.add_space(20.0);
    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new("✅ All Set!").size(22.0).strong());
    });

    ui.add_space(8.0);
    ui.label(
        egui::RichText::new("Here's a summary of your configuration:")
            .size(13.0)
            .color(egui::Color32::from_rgb(165, 165, 165)),
    );

    ui.add_space(16.0);

    // Summary items
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(USER).size(14.0));
        ui.label(egui::RichText::new("Name:").size(12.0).strong());
        let display_name = if state.git_name.is_empty() {
            "(not set)".to_string()
        } else {
            state.git_name.clone()
        };
        ui.label(egui::RichText::new(display_name).size(12.0));
    });

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(ENVELOPE).size(14.0));
        ui.label(egui::RichText::new("Email:").size(12.0).strong());
        let display_email = if state.git_email.is_empty() {
            "(not set)".to_string()
        } else {
            state.git_email.clone()
        };
        ui.label(egui::RichText::new(display_email).size(12.0));
    });

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(GITHUB_LOGO).size(14.0));
        ui.label(egui::RichText::new("GitHub:").size(12.0).strong());
        if let Some(ref github_user) = state.github_user {
            ui.label(
                egui::RichText::new(format!("{} {}", CHECK_CIRCLE, github_user.login))
                    .size(12.0)
                    .color(egui::Color32::from_rgb(80, 180, 80)),
            );
        } else {
            ui.label(
                egui::RichText::new("Not connected")
                    .size(12.0)
                    .color(egui::Color32::from_rgb(140, 140, 140)),
            );
        }
    });

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(KEY).size(14.0));
        ui.label(egui::RichText::new("SSH Keys:").size(12.0).strong());
        ui.label(egui::RichText::new(format!("{}", state.ssh_keys.len())).size(12.0));
    });

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(SHIELD_CHECK).size(14.0));
        ui.label(egui::RichText::new("GPG Keys:").size(12.0).strong());
        ui.label(egui::RichText::new(format!("{}", state.gpg_keys.len())).size(12.0));
    });

    ui.add_space(24.0);

    ui.vertical_centered(|ui| {
        if ui
            .button(
                egui::RichText::new(format!("Get Started {}", ARROW_RIGHT))
                    .size(14.0)
                    .strong(),
            )
            .clicked()
        {
            action = WizardAction::Complete {
                git_name: state.git_name.clone(),
                git_email: state.git_email.clone(),
            };
        }
    });

    action
}

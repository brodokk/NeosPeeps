//! The login page of the app

use eframe::{
	egui::{Button, ComboBox, SelectableLabel, TextEdit, Ui},
	epi,
};
use neos::api_client::{
	NeosRequestUserSession,
	NeosRequestUserSessionIdentifier,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use super::NeosPeepsApp;

impl NeosPeepsApp {
	fn identifier_picker(&mut self, ui: &mut Ui, is_loading: bool) {
		ComboBox::from_label("Login type")
			.selected_text(self.stored.identifier.as_ref())
			.show_ui(ui, |ui| {
				if ui
					.add(SelectableLabel::new(
						matches!(
							self.stored.identifier,
							NeosRequestUserSessionIdentifier::Username(_)
						),
						"Username",
					))
					.clicked()
				{
					self.stored.identifier = NeosRequestUserSessionIdentifier::Username(
						self.stored.identifier.inner().into(),
					);
				}

				if ui
					.add(SelectableLabel::new(
						matches!(
							self.stored.identifier,
							NeosRequestUserSessionIdentifier::Email(_)
						),
						"Email",
					))
					.clicked()
				{
					self.stored.identifier = NeosRequestUserSessionIdentifier::Email(
						self.stored.identifier.inner().into(),
					);
				}

				if ui
					.add(SelectableLabel::new(
						matches!(
							self.stored.identifier,
							NeosRequestUserSessionIdentifier::OwnerID(_)
						),
						"OwnerID",
					))
					.clicked()
				{
					self.stored.identifier = NeosRequestUserSessionIdentifier::OwnerID(
						self.stored.identifier.inner().into(),
					);
				}
			});

		let label = self.stored.identifier.as_ref().to_string();

		ui.add(
			TextEdit::singleline(self.stored.identifier.inner_mut())
				.hint_text(label)
				.interactive(!is_loading),
		);
	}

	pub fn login_page(&mut self, ui: &mut Ui, frame: &epi::Frame) {
		ui.heading("Log in");
		ui.label("Currently Neos' Oauth doesn't implement the required details for this application, thus logging in is the only way to actually use it.");

		let login_op_in_progress = self.runtime.neos_api.is_none();

		if login_op_in_progress {
			ui.vertical_centered_justified(|ui| {
				ui.label("Loading...");
			});
		}

		ui.add_enabled_ui(!login_op_in_progress, |ui| {
			ui.group(|ui| {
				self.identifier_picker(ui, login_op_in_progress);

				ui.add(
					TextEdit::singleline(&mut self.runtime.password)
						.password(true)
						.hint_text("Password")
						.interactive(!login_op_in_progress),
				);

				let totp_resp = ui.add(
					TextEdit::singleline(&mut self.runtime.totp)
						.hint_text("2FA")
						.interactive(!login_op_in_progress)
						.desired_width(80_f32),
				);

				if totp_resp.changed() {
					self.runtime.totp = self
						.runtime
						.totp
						.chars()
						.filter(|v| v.is_numeric())
						.take(6)
						.collect();
				}

				let submit_button_resp = ui.add(Button::new("Log in"));

				if submit_button_resp.clicked()
					&& !self.stored.identifier.inner().is_empty()
					&& !self.runtime.password.is_empty()
					&& !login_op_in_progress
					&& (self.runtime.totp.is_empty()
						|| self.runtime.totp.chars().count() == 6)
				{
					let rand_string: String = thread_rng()
						.sample_iter(&Alphanumeric)
						.take(30)
						.map(char::from)
						.collect();
					let mut session_request = NeosRequestUserSession::with_identifier(
						self.stored.identifier.clone(),
						std::mem::take(&mut self.runtime.password),
					)
					.remember_me(true)
					.machine_id(rand_string);

					if !self.runtime.totp.is_empty() {
						session_request =
							session_request.totp(std::mem::take(&mut self.runtime.totp));
					}

					self.login_new(session_request, frame);
				}
			});
		});
	}
}

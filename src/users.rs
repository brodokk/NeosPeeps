//! The friends page of the app

use std::{cmp::Ordering, rc::Rc};

use eframe::epi;
use neos::{
	api_client::{AnyNeos, Neos},
	NeosFriend,
	NeosUser,
	NeosUserOnlineStatus,
	NeosUserProfile,
	NeosUserStatus,
};

use crate::{app::NeosPeepsApp, image::TextureDetails};

fn order_users(s1: &NeosUserStatus, s2: &NeosUserStatus) -> Ordering {
	// if their current session is joinable
	if s1.current_session_access_level > s2.current_session_access_level {
		return Ordering::Less;
	};
	if s1.current_session_access_level < s2.current_session_access_level {
		return Ordering::Greater;
	};

	// if the friends are marked as online
	if s1.online_status == NeosUserOnlineStatus::Online
		&& s2.online_status != NeosUserOnlineStatus::Online
	{
		return Ordering::Less;
	};
	if s1.online_status != NeosUserOnlineStatus::Online
		&& s2.online_status == NeosUserOnlineStatus::Online
	{
		return Ordering::Greater;
	};

	// if at least not offline
	if s1.online_status != NeosUserOnlineStatus::Offline
		&& s2.online_status == NeosUserOnlineStatus::Offline
	{
		return Ordering::Less;
	};
	if s1.online_status == NeosUserOnlineStatus::Offline
		&& s2.online_status != NeosUserOnlineStatus::Offline
	{
		return Ordering::Greater;
	};
	Ordering::Equal
}

impl NeosPeepsApp {
	/// Refreshes friends in a background thread
	pub fn refresh_friends(&mut self, frame: &epi::Frame) {
		let neos_api_arc = match &self.runtime.neos_api {
			Some(api) => api.clone(),
			None => return,
		};

		self.threads.loading.friends.set(true);
		let friends_sender = self.threads.channels.friends_sender();
		self.threads.spawn_data_op(move || {
			if let AnyNeos::Authenticated(neos_api) = &*neos_api_arc {
				match neos_api.get_friends() {
					Ok(mut friends) => {
						friends.sort_by(|f1, f2| order_users(&f1.status, &f2.status));
						friends_sender.send(Ok(friends)).unwrap();
					}
					Err(e) => {
						friends_sender.send(Err(e.to_string())).unwrap();
					}
				}
			}
		});

		frame.request_repaint();
	}

	pub fn search_users(&mut self, frame: &epi::Frame) {
		let neos_api = match &self.runtime.neos_api {
			Some(api) => api.clone(),
			None => return,
		};

		self.threads.loading.users.set(true);
		let users_sender = self.threads.channels.users_sender();
		let search = self.stored.filter_search.clone();
		self.threads.spawn_data_op(move || {
			let res = neos_api.search_users(&search);
			users_sender.send(res.map_err(|e| e.to_string())).unwrap();
		});

		frame.request_repaint();
	}

	/// Gets the user for the user window
	pub fn get_user(&self, frame: &epi::Frame, id: &neos::id::User) {
		let neos_api = match &self.runtime.neos_api {
			Some(api) => api.clone(),
			None => return,
		};

		if let Some((w_id, _, _)) = &*self.runtime.user_window.borrow() {
			if w_id != id {
				return;
			}
		} else {
			*self.runtime.user_window.borrow_mut() = Some((id.clone(), None, None));
		}

		self.threads.loading.user.set(true);

		let id = id.clone();
		let user_sender = self.threads.channels.user_sender();
		self.threads.spawn_data_op(move || {
			let res = neos_api.get_user(id);
			user_sender.send(res.map_err(|e| e.to_string())).unwrap();
		});
		frame.request_repaint();
	}

	/// Gets the user status for the user window
	pub fn get_user_status(&self, frame: &epi::Frame, id: &neos::id::User) {
		let neos_api = match &self.runtime.neos_api {
			Some(api) => api.clone(),
			None => return,
		};
		if let Some((w_id, _, _)) = &*self.runtime.user_window.borrow() {
			if w_id != id {
				return;
			}
		} else {
			*self.runtime.user_window.borrow_mut() = Some((id.clone(), None, None));
		}

		self.threads.loading.user_status.set(true);

		let id = id.clone();
		let user_status_sender = self.threads.channels.user_status_sender();
		self.threads.spawn_data_op(move || {
			match neos_api.get_user_status(id.clone()) {
				Ok(user_status) => {
					user_status_sender.send(Ok((id, user_status))).unwrap();
				}
				Err(e) => user_status_sender.send(Err(e.to_string())).unwrap(),
			}
		});

		frame.request_repaint();
	}

	pub fn open_user(
		&self, frame: &epi::Frame, id: &neos::id::User, user: Option<NeosUser>,
		user_status: Option<NeosUserStatus>,
	) {
		let (missing_user, missing_status) =
			(user.is_none(), user_status.is_none());
		*self.runtime.user_window.borrow_mut() =
			Some((id.clone(), user, user_status));
		if missing_user {
			self.get_user(frame, id);
		}
		if missing_status {
			self.get_user_status(frame, id);
		}
	}

	pub fn get_pfp(
		&self, frame: &epi::Frame, profile: &Option<NeosUserProfile>,
	) -> Rc<TextureDetails> {
		let pfp_url = match profile {
			Some(profile) => &profile.icon_url,
			None => &None,
		};
		let pfp = match pfp_url {
			Some(pfp_url) => self.load_texture(pfp_url, frame),
			None => None,
		};

		pfp.unwrap_or_else(|| self.runtime.default_profile_picture.clone().unwrap())
	}

	pub fn user_to_friend(&self, user: &NeosUser) -> Option<&NeosFriend> {
		use rayon::prelude::*;

		self.runtime.friends.par_iter().find_any(|f| f.id == user.id)
	}
}

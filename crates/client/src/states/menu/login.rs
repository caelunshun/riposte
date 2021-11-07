use riposte_backend_api::{Authenticated, LogInRequest, RegisterRequest};

use crate::{
    backend::BackendResponse,
    context::Context,
    generated,
    state::StateAttachment,
    ui::{FillScreen, Z_FOREGROUND},
};

enum Action {
    SwitchPage(Page),
    AdvanceToMenu(Authenticated),
}

enum LogInMessage {
    RegisterLinkClicked,
    Submitted,
}

enum RegisterMessage {
    LogInLinkClicked,
    Submitted,
}

struct LogInPage {
    attachment: StateAttachment,
    handle: generated::LoginPage,
    response_handle: Option<BackendResponse<Authenticated>>,
}

impl LogInPage {
    pub fn new(context: &Context) -> Self {
        let attachment = context.state_manager().create_state();

        let (handle, _) =
            attachment.create_window::<generated::LoginPage, _>(FillScreen, Z_FOREGROUND);

        handle
            .register_link
            .get_mut()
            .on_click(|| LogInMessage::RegisterLinkClicked);
        handle.submit.get_mut().on_click(|| LogInMessage::Submitted);

        Self {
            attachment,
            handle,
            response_handle: None,
        }
    }

    pub fn update(&mut self, cx: &mut Context) -> Option<Action> {
        let mut action = None;
        let mut ui = cx.ui_mut();
        while let Some(msg) = ui.pop_message() {
            drop(ui);
            match msg {
                LogInMessage::RegisterLinkClicked => {
                    action = Some(Action::SwitchPage(Page::Register(RegisterPage::new(cx))));
                }
                LogInMessage::Submitted => {
                    self.response_handle = Some(cx.backend().log_in(LogInRequest {
                        username: self.handle.username_input.get().current_input().to_owned(),
                        password: self.handle.password_input.get().current_input().to_owned(),
                    }));
                }
            }
            ui = cx.ui_mut();
        }

        if let Some(response) = self.response_handle.as_ref().and_then(|h| h.get()) {
            match response {
                Ok(a) => {
                    log::info!("Logged in successfully.");
                    action = Some(Action::AdvanceToMenu(a.get_ref().clone()));
                }
                Err(e) => {
                    log::error!("Login error: {}", e);
                    self.handle.error_text.get_mut().set_text(
                        text!("Error: {}", e.message())
                      
                       
                    );
                    self.response_handle = None;
                }
            }
        }

        action
    }
}

struct RegisterPage {
    attachment: StateAttachment,
    handle: generated::RegisterPage,
    response_handle: Option<BackendResponse<Authenticated>>,
}

impl RegisterPage {
    pub fn new(context: &Context) -> Self {
        let attachment = context.state_manager().create_state();

        let (handle, _) =
            attachment.create_window::<generated::RegisterPage, _>(FillScreen, Z_FOREGROUND);

        handle
            .login_link
            .get_mut()
            .on_click(|| RegisterMessage::LogInLinkClicked);
        handle
            .submit
            .get_mut()
            .on_click(|| RegisterMessage::Submitted);

        Self {
            attachment,
            handle,
            response_handle: None,
        }
    }

    pub fn update(&mut self, cx: &mut Context) -> Option<Action> {
        let mut action = None;
        let mut ui = cx.ui_mut();
        while let Some(msg) = ui.pop_message() {
            drop(ui);
            match msg {
                RegisterMessage::LogInLinkClicked => {
                    action = Some(Action::SwitchPage(Page::LogIn(LogInPage::new(cx))));
                }
                RegisterMessage::Submitted => {
                    if self.handle.password_input.get().current_input()
                        != self.handle.verify_password_input.get().current_input()
                    {
                        self.handle
                            .error_text
                            .get_mut()
                            .set_text(text!("Passwords do not match"));
                    } else {
                        self.response_handle =
                            Some(cx.backend().register_account(RegisterRequest {
                                username:
                                    self.handle.username_input.get().current_input().to_owned(),
                                password:
                                    self.handle.password_input.get().current_input().to_owned(),
                                email: self.handle.email_input.get().current_input().to_owned(),
                            }));
                    }
                }
            }
            ui = cx.ui_mut();
        }

        if let Some(response) = self.response_handle.as_ref().and_then(|h| h.get()) {
            match response {
                Ok(a) => {
                    log::info!("Registered account successfully.");
                    action = Some(Action::AdvanceToMenu(a.get_ref().clone()));
                }
                Err(e) => {
                    log::error!("Register error: {}", e);
                    self.handle.error_text.get_mut().set_text(
                        text!("Error: {}", e.message())
                        
                    );
                    self.response_handle = None;
                }
            }
        }

        action
    }
}

enum Page {
    LogIn(LogInPage),
    Register(RegisterPage),
}

/// The login / register page, prompting the user
/// to authenticate with the backend service.
pub struct LoginState {
    attachment: StateAttachment,

    page: Page,
}

impl LoginState {
    pub fn new(cx: &Context) -> Self {
        let attachment = cx.state_manager().create_state();

        let page = Page::LogIn(LogInPage::new(cx));

        Self { attachment, page }
    }

    pub fn update(&mut self, cx: &mut Context) -> Option<Authenticated> {
        let action = match &mut self.page {
            Page::LogIn(p) => p.update(cx),
            Page::Register(p) => p.update(cx),
        };

        if let Some(action) = action {
            match action {
                Action::SwitchPage(p) => self.page = p,
                Action::AdvanceToMenu(a) => return Some(a),
            }
        }

        None
    }
}

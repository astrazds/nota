use std::cell::RefCell;
use std::rc::Rc;
use webkit6::glib::prelude::Cast;
use webkit6::prelude::{PolicyDecisionExt, WebViewExt};
use webkit6::{NavigationPolicyDecision, PolicyDecisionType, Settings, WebView};

use nota_app::preview::PREVIEW_CSP;

#[derive(Clone)]
pub struct SecurePreview {
    web_view: WebView,
    document: Rc<RefCell<String>>,
}

impl SecurePreview {
    pub fn new(open_external: impl Fn(String, bool) + 'static) -> Self {
        let settings = Settings::new();
        settings.set_enable_javascript(false);
        settings.set_enable_javascript_markup(false);
        settings.set_enable_html5_local_storage(false);
        settings.set_enable_offline_web_application_cache(false);
        settings.set_enable_page_cache(false);
        settings.set_enable_media(false);
        settings.set_enable_media_stream(false);
        settings.set_enable_mediasource(false);
        settings.set_enable_webgl(false);
        settings.set_enable_webrtc(false);

        let web_view = WebView::builder()
            .editable(false)
            .settings(&settings)
            .default_content_security_policy(PREVIEW_CSP)
            .build();
        web_view.connect_decide_policy(move |_view, decision, decision_type| {
            if !matches!(
                decision_type,
                PolicyDecisionType::NavigationAction | PolicyDecisionType::NewWindowAction
            ) {
                return false;
            }
            let Some(navigation) = decision.downcast_ref::<NavigationPolicyDecision>() else {
                decision.ignore();
                return true;
            };
            let Some(action) = navigation.navigation_action() else {
                decision.ignore();
                return true;
            };
            let Some(uri) = action.request().and_then(|request| request.uri()) else {
                decision.ignore();
                return true;
            };

            if uri == "about:blank" || uri.starts_with("about:blank#") {
                decision.use_();
                return true;
            }
            open_external(uri.to_string(), action.is_user_gesture());
            decision.ignore();
            true
        });

        Self {
            web_view,
            document: Rc::new(RefCell::new(String::new())),
        }
    }

    pub fn widget(&self) -> &WebView {
        &self.web_view
    }

    pub fn load_html(&self, document: &str) {
        if *self.document.borrow() != document {
            self.web_view.load_html(document, None);
            self.document.replace(document.to_string());
        }
    }
}

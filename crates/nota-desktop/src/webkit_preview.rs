use url::Url;
use webkit6::glib::prelude::Cast;
use webkit6::prelude::{PolicyDecisionExt, WebViewExt};
use webkit6::{NavigationPolicyDecision, PolicyDecisionType, Settings, WebView};

use crate::preview::{PREVIEW_CSP, external_navigation_target, preview_document};

#[derive(Clone)]
pub struct SecurePreview {
    web_view: WebView,
}

impl SecurePreview {
    pub fn new(open_external: impl Fn(Url) + 'static) -> Self {
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
            if let Some(target) = external_navigation_target(&uri, action.is_user_gesture()) {
                open_external(target);
            }
            decision.ignore();
            true
        });

        Self { web_view }
    }

    pub fn widget(&self) -> &WebView {
        &self.web_view
    }

    pub fn load_note(&self, title: &str, tags: &[String], markdown: &str, dark: bool) {
        self.web_view
            .load_html(&preview_document(title, tags, markdown, dark), None);
    }
}

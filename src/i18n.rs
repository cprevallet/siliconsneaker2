use fluent::{FluentArgs, FluentBundle, FluentResource};
use std::cell::RefCell;
use unic_langid::LanguageIdentifier;

thread_local! {
    static BUNDLE: RefCell<FluentBundle<FluentResource>> = RefCell::new({
        let lang_id: LanguageIdentifier = "en-US".parse().expect("Parsing failed");
        let mut bundle = FluentBundle::new(vec![lang_id]);

        let ftl_string = include_str!("../i18n/en-US/gui.ftl").to_string();
        let resource = FluentResource::try_new(ftl_string)
            .expect("Failed to parse an FTL string.");
        bundle.add_resource(resource).expect("Failed to add FTL resources.");

        bundle
    });
}

pub fn tr(msg_id: &str, args: Option<&FluentArgs>) -> String {
    BUNDLE.with(|bundle_cell| {
        let bundle = bundle_cell.borrow();
        let msg = bundle.get_message(msg_id).expect("Message not found");
        let pattern = msg.value().expect("Message has no value");
        let mut errors = vec![];
        bundle
            .format_pattern(pattern, args, &mut errors)
            .to_string()
    })
}

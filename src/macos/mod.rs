extern crate cocoa;
extern crate fruitbasket;
extern crate objc;
extern crate objc_foundation;

use std::sync::mpsc::Sender;

use objc::*;

use crate::NSCallback;

use self::cocoa::appkit::NSStatusBar;
use self::cocoa::appkit::{NSButton, NSMenu, NSMenuItem, NSStatusItem, NSVariableStatusItemLength};
use self::cocoa::base::{nil, YES};
use self::cocoa::foundation::NSString;
use self::fruitbasket::{FruitApp, FruitStopper};
use self::rustnsobject::{NSObj, NSObjCallbackTrait, NSObjTrait};

mod rustnsobject;

pub type Object = objc::runtime::Object;

pub struct OSXStatusBar {
    object: NSObj,
    app: FruitApp,
    status_bar_item: *mut objc::runtime::Object,
    menu_bar: *mut objc::runtime::Object,
}

impl OSXStatusBar {
    pub fn new(title: &str, tx: Sender<String>) -> OSXStatusBar {
        unsafe {
            let app = FruitApp::new();
            app.set_activation_policy(fruitbasket::ActivationPolicy::Prohibited);
            let status_bar = NSStatusBar::systemStatusBar(nil);

            let mut bar = OSXStatusBar {
                app,
                status_bar_item: status_bar.statusItemWithLength_(NSVariableStatusItemLength),
                menu_bar: NSMenu::new(nil),
                object: NSObj::alloc(tx),
            };

            // Default mode for menu bar items: blue highlight when selected
            let _: () = msg_send![bar.status_bar_item, setHighlightMode: YES];

            // Set title.  Only displayed if image fails to load.
            let title = NSString::alloc(nil).init_str(title);
            NSButton::setTitle_(bar.status_bar_item, title);
            let _: () = msg_send![title, release];

            bar.status_bar_item.setMenu_(bar.menu_bar);
            bar.object.cb_fn = Some(Box::new(move |s, sender| {
                let cb = s.get_value(sender);
                cb(sender, &s.tx);
            }));

            bar
        }
    }

    pub fn stopper(&self) -> FruitStopper {
        self.app.stopper()
    }

    // TODO: whole API should accept menu option.  this whole thing should
    // be split out into its own recursive menu-builder trait.  this is
    // horrible.
    pub fn add_item(
        &mut self,
        menu: Option<*mut Object>,
        item: &str,
        callback: NSCallback,
        selected: bool,
    ) -> *mut Object {
        unsafe {
            let txt = NSString::alloc(nil).init_str(item);
            let quit_key = NSString::alloc(nil).init_str("");
            let app_menu_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
                txt,
                self.object.selector(),
                quit_key,
            );
            let _: () = msg_send![txt, release];
            let _: () = msg_send![quit_key, release];
            self.object.add_callback(app_menu_item, callback);
            let objc = self.object.take_objc();
            let _: () = msg_send![app_menu_item, setTarget: objc];
            if selected {
                let _: () = msg_send![app_menu_item, setState: 1];
            }
            let item: *mut Object = app_menu_item;
            match menu {
                Some(menu) => {
                    menu.addItem_(app_menu_item);
                }
                None => {
                    self.menu_bar.addItem_(app_menu_item);
                }
            }
            let _: () = msg_send![app_menu_item, release];
            item
        }
    }

    pub fn set_title(&self, title: &str) {
        unsafe {
            let title = NSString::alloc(nil).init_str(title);
            NSButton::setTitle_(self.status_bar_item, title);
            let _: () = msg_send![title, release];
        }
    }

    pub fn run(&mut self, block: bool) {
        let period = match block {
            true => fruitbasket::RunPeriod::Forever,
            _ => fruitbasket::RunPeriod::Once,
        };

        let _ = self.app.run(period);
    }
}

// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    backend::{PwNodeObject, PwvucontrolManager},
    macros::*,
    ui::{PwStreamDropDown, PwVolumeBox, PwVolumeBoxImpl},
};
use glib::{clone, closure_local};
use gtk::{prelude::*, subclass::prelude::*};
use wireplumber as wp;

use super::volumebox::PwVolumeBoxExt;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/saivert/pwvucontrol/gtk/streambox.ui")]
    pub struct PwStreamBox {
        #[template_child]
        pub output_dropdown: TemplateChild<PwStreamDropDown>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PwStreamBox {
        const NAME: &'static str = "PwStreamBox";
        type Type = super::PwStreamBox;
        type ParentType = PwVolumeBox;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PwStreamBox {
        fn constructed(&self) {
            let manager = PwvucontrolManager::default();

            let obj = self.obj();
            let item = obj.node_object().expect("nodeobj");

            obj.set_default_node_change_handler(clone!(@weak self as widget => move || {
                widget.obj().update_output_device_dropdown();
            }));

            self.parent_constructed();

            if let Some(metadata) = manager.metadata() {
                let boundid = item.boundid();
                let widget = self.obj();
                let changed_closure = closure_local!(@watch widget =>
                    move |_obj: &wp::pw::Metadata, id: u32, key: Option<String>, _type: Option<String>, _value: Option<String>| {
                    let key = key.unwrap_or_default();
                    if id == boundid && key.contains("target.") {
                        pwvucontrol_info!("metadata changed handler id: {boundid} {key:?} {_value:?}!");
                        widget.update_output_device_dropdown();
                    }
                });
                metadata.connect_closure("changed", false, changed_closure);
            }

            // Create our custom output dropdown widget and add it to the layout
            self.output_dropdown.set_nodeobj(Some(&item));

            glib::idle_add_local_once(clone!(@weak self as widget => move || {
                widget.obj().update_output_device_dropdown();
            }));
        }
    }
    impl WidgetImpl for PwStreamBox {}
    impl ListBoxRowImpl for PwStreamBox {}
    impl PwVolumeBoxImpl for PwStreamBox {}

    impl PwStreamBox {}
}

glib::wrapper! {
    pub struct PwStreamBox(ObjectSubclass<imp::PwStreamBox>)
        @extends gtk::Widget, gtk::ListBoxRow, PwVolumeBox,
        @implements gtk::Actionable;
}

impl PwStreamBox {
    pub(crate) fn new(node_object: &impl IsA<PwNodeObject>) -> Self {
        glib::Object::builder().property("node-object", node_object).build()
    }

    pub(crate) fn update_output_device_dropdown(&self) {
        let manager = PwvucontrolManager::default();

        let item = self.node_object().expect("nodeobj");

        let sinkmodel = manager.get_model_for_nodetype(item.nodetype());

        let imp = self.imp();

        let output_dropdown = imp.output_dropdown.get();

        let id = self.default_node();

        let string = if let Some(node) = manager.get_node_by_id(id) {
            format!("Default ({})", node.name())
        } else {
            "Default".to_string()
        };
        output_dropdown.set_default_text(&string);


        if let Some(deftarget) = item.default_target() {
            if let Some(pos) = sinkmodel.get_node_pos_from_id(deftarget.boundid()) {
                pwvucontrol_info!(
                    "switching to preferred target pos={pos} boundid={} serial={}",
                    deftarget.boundid(),
                    deftarget.serial()
                );
                output_dropdown.set_selected_no_send(pos + 1);
            }
        } else {
            output_dropdown.set_selected_no_send(0);
        }
    }
}

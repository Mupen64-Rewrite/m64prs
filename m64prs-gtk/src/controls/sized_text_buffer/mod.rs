mod inner {
    use gtk::{prelude::*, subclass::prelude::*};
    use std::cell::Cell;

    #[derive(Default, glib::Properties)]
    #[properties(wrapper_type = super::SizedTextBuffer)]
    pub struct SizedTextBuffer {
        #[property(get, set, default = 100)]
        max_len: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SizedTextBuffer {
        const NAME: &'static str = "M64PRS_SizedTextBuffer";
        type Type = super::SizedTextBuffer;
        type ParentType = gtk::TextBuffer;
    }

    #[glib::derived_properties]
    impl ObjectImpl for SizedTextBuffer {}
    impl TextBufferImpl for SizedTextBuffer {
        fn insert_text(&self, iter: &mut gtk::TextIter, new_text: &str) {
            let obj = self.obj();
            let max_len = obj.max_len() as usize;

            let curr_bytes = obj.text(&obj.start_iter(), &obj.end_iter(), true).len();
            if curr_bytes + new_text.len() > max_len {
                // max clipped text length
                let clip_len = max_len - curr_bytes;
                let clip_pos = (0..clip_len)
                    .rev()
                    .find(|pos| new_text.is_char_boundary(*pos))
                    .unwrap();
                self.parent_insert_text(iter, &new_text[0..clip_pos]);
            } else {
                self.parent_insert_text(iter, new_text);
            }
        }
        
        fn insert_paintable(&self, _iter: &mut gtk::TextIter, _paintable: &gdk::Paintable) {}
    }
}

glib::wrapper! {
    /// A [`gtk::TextBuffer`] that clips its content to a specific number of bytes.
    pub struct SizedTextBuffer(ObjectSubclass<inner::SizedTextBuffer>)
        @extends gtk::TextBuffer;
}

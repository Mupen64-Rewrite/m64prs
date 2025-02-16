use glib::value::ToValue;

pub struct AutoderefClosureReturn<T>(pub T);

pub trait AutoderefClosureReturnNone {
    fn result_value(self) -> Option<glib::Value>;
}
impl AutoderefClosureReturnNone for &AutoderefClosureReturn<()> {
    fn result_value(self) -> Option<glib::Value> {
        None
    }
}

pub trait AutoderefClosureReturnSome {
    fn result_value(self) -> Option<glib::Value>;
}
impl<T: ToValue> AutoderefClosureReturnSome for AutoderefClosureReturn<T> {
    fn result_value(self) -> Option<glib::Value> {
        Some(glib::value::ToValue::to_value(&self.0))
    }
}
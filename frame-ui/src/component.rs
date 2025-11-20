use crate::Widget;

pub trait Component {
	fn render(&self) -> Box<dyn Widget>;
}

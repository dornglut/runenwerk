use anyhow::Result;

pub struct Node<C>{
	pub name: String,
	pub func: Box<dyn FnMut(&mut C) -> Result<()> + Send>,
}

impl<C> Node<C>{
	pub fn new<F>(name: &str, func: F) -> Self
	where
		F: FnMut(&mut C) -> Result<()> + Send + 'static
	{
		Self {
			name: name.to_string(),
			func: Box::new(func),
		}
	}
}
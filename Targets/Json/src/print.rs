use std::{
	io::{Result, Write},
	rc::Rc,
};

pub trait Print {
	fn print(&self, out: &mut dyn Write) -> Result<()>;
}

impl<T: Print + ?Sized> Print for &T {
	fn print(&self, out: &mut dyn Write) -> Result<()> {
		T::print(self, out)
	}
}

impl<T: Print + ?Sized> Print for Rc<T> {
	fn print(&self, out: &mut dyn Write) -> Result<()> {
		T::print(self, out)
	}
}

impl<T: Print> Print for [T] {
	fn print(&self, out: &mut dyn Write) -> Result<()> {
		let mut iter = self.iter();

		write!(out, "[")?;

		if let Some(first) = iter.next() {
			T::print(first, out)?;

			iter.try_for_each(|item| {
				write!(out, ", ")?;

				T::print(item, out)
			})?;
		}

		write!(out, "]")
	}
}

impl<T: Print> Print for Vec<T> {
	fn print(&self, out: &mut dyn Write) -> Result<()> {
		<[T]>::print(self, out)
	}
}

impl<T: Print, U: Print> Print for (T, U) {
	fn print(&self, out: &mut dyn Write) -> Result<()> {
		self.0.print(out)?;
		write!(out, ":")?;
		self.1.print(out)
	}
}

impl Print for u32 {
	fn print(&self, out: &mut dyn Write) -> Result<()> {
		write!(out, "{self}")
	}
}

impl Print for str {
	fn print(&self, out: &mut dyn Write) -> Result<()> {
		let inner = self.as_bytes().escape_ascii();

		write!(out, "\"{inner}\"")
	}
}

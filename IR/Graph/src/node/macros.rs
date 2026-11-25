//! Shared visitor macros for node operation types.
//!
//! Each concept file invokes `handle_sources!` inside its node impls to
//! generate `for_each_outer` / `for_each_mut_outer` methods without
//! hand-writing the traversal.

macro_rules! handle_field {
	($handler:ident, $name:ident, call) => {
		$handler($name);
	};
	($handler:ident, $name:ident, dereference_call) => {
		$handler(*$name);
	};
	($handler:ident, $name:ident, for_each, $($rest:tt)*) => {
		for item in $name {
			handle_field!($handler, item, $($rest)*);
		}
	};
	($handler:ident, $name:ident, method, $method:ident) => {
		$name.$method(&mut $handler);
	};
}

macro_rules! handle_argument_source {
	($handler:ident, $name:ident, ignore) => {};
	($handler:ident, $name:ident, link) => {
		handle_field!($handler, $name, dereference_call)
	};
	($handler:ident, $name:ident, link_list) => {
		handle_field!($handler, $name, for_each, dereference_call)
	};
	($handler:ident, $name:ident, method) => {
		handle_field!($handler, $name, method, for_each_outer)
	};
}

macro_rules! handle_mut_argument_source {
	($handler:ident, $name:ident, ignore) => {};
	($handler:ident, $name:ident, link) => {
		handle_field!($handler, $name, call)
	};
	($handler:ident, $name:ident, link_list) => {
		handle_field!($handler, $name, for_each, call)
	};
	($handler:ident, $name:ident, method) => {
		handle_field!($handler, $name, method, for_each_mut_outer)
	};
}

macro_rules! handle_binding {
	($name:ident, ignore) => {
		_
	};
	($name:ident, $variant:ident) => {
		$name
	};
}

macro_rules! handle_visitor {
	($name:ident, $type:ty, $visitor:ident, ( $( ($field:ident, $variant:ident) ),* )) => {
		pub(crate) fn $name<H>(&self, mut handler: H)
		where
			H: FnMut($type),
		{
			let Self { $($field: handle_binding!($field, $variant)),* } = self;

			$(
				$visitor!(handler, $field, $variant)
			);*;
		}
	};
}

macro_rules! handle_mut_visitor {
	($name:ident, $type:ty, $visitor:ident, ( $( ($field:ident, $variant:ident) ),* )) => {
		pub(crate) fn $name<H>(&mut self, mut handler: H)
		where
			H: FnMut(&mut $type),
		{
			let Self { $($field: handle_binding!($field, $variant)),* } = self;

			$(
				$visitor!(handler, $field, $variant)
			);*;
		}
	};
}

macro_rules! handle_sources {
	($( ($field:ident, $action:ident) ),*) => {
		handle_visitor!(for_each_outer, $crate::Link, handle_argument_source, ( $( ($field, $action) ),* ));
		handle_mut_visitor!(for_each_mut_outer, $crate::Link, handle_mut_argument_source, ( $( ($field, $action) ),* ));
	};
}

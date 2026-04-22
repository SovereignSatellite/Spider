/// A parsed runtime library section.
pub struct Section {
	/// The section dependency references.
	pub references: Box<[&'static str]>,
	/// The section name.
	pub name: &'static str,
	/// The section source contents.
	pub contents: &'static str,
}

impl Section {
	const SECTION_HEADER: &str = "-- SECTION ";
	const NEEDS_HEADER: &str = "-- NEEDS ";

	fn try_parse_header(
		source: &'static str,
		header: &'static str,
	) -> Option<(&'static str, &'static str)> {
		if !source.starts_with(header) {
			return None;
		}

		let source = &source[header.len()..];
		let end = source.find('\n')?;
		let (data, source) = source.split_at(end);

		Some((data.trim_end(), source.trim_start()))
	}

	fn parse_references(mut source: &'static str) -> (Box<[&'static str]>, &'static str) {
		let mut dependencies = Vec::new();

		while let Some((name, next)) = Self::try_parse_header(source, Self::NEEDS_HEADER) {
			dependencies.push(name);

			source = next;
		}

		(dependencies.into(), source)
	}

	fn parse_contents(source: &'static str) -> (&'static str, &'static str) {
		let end = source.find(Self::SECTION_HEADER).unwrap_or(source.len());
		let (content, source) = source.split_at(end);

		(content.trim(), source)
	}

	/// Tries to parse a section from the given source.
	///
	/// # Panics
	///
	/// Panics if the parsed references are not sorted.
	#[must_use]
	pub fn try_parse(source: &'static str) -> Option<(Self, &'static str)> {
		let (name, source) = Self::try_parse_header(source, Self::SECTION_HEADER)?;
		let (references, source) = Self::parse_references(source);
		let (contents, source) = Self::parse_contents(source);

		assert!(
			references.is_sorted(),
			"references for `{name}` should be sorted"
		);

		Some((
			Self {
				references,
				name,
				contents,
			},
			source,
		))
	}
}

/// A collection of runtime library sections.
pub struct Sections {
	list: Vec<Section>,
}

impl Sections {
	/// The bit library source.
	pub const BIT_SOURCE: &str = include_str!("../../runtime/builtin/bit.lua");
	/// The FFI library source.
	pub const FFI_SOURCE: &str = include_str!("../../runtime/builtin/ffi.lua");
	/// The math library source.
	pub const MATH_SOURCE: &str = include_str!("../../runtime/builtin/math.lua");

	/// The F32 assembly source.
	pub const F32_ASSEMBLY_SOURCE: &str = include_str!("../../runtime/assembly/f32.lua");

	/// The I32 core source.
	pub const I32_SOURCE: &str = include_str!("../../runtime/core/i32.lua");
	/// The I64 core source.
	pub const I64_SOURCE: &str = include_str!("../../runtime/core/i64.lua");
	/// The F32 core source.
	pub const F32_SOURCE: &str = include_str!("../../runtime/core/f32.lua");
	/// The F64 core source.
	pub const F64_SOURCE: &str = include_str!("../../runtime/core/f64.lua");
	/// The table core source.
	pub const TABLE_SOURCE: &str = include_str!("../../runtime/core/table.lua");
	/// The memory core source.
	pub const MEMORY_SOURCE: &str = include_str!("../../runtime/core/memory.lua");

	/// The Turing machine source runtime.
	pub const TURING_MACHINE_SOURCE: &str = include_str!("../../runtime/source/turing_machine.lua");
	/// The WebAssembly source runtime.
	pub const WEB_ASSEMBLY_SOURCE: &str = include_str!("../../runtime/source/web_assembly.lua");

	/// Creates a new section collection with all built-in sources.
	#[must_use]
	pub fn with_built_ins() -> Self {
		let mut sections = Self { list: Vec::new() };

		sections.parse_from(Self::BIT_SOURCE);
		sections.parse_from(Self::FFI_SOURCE);
		sections.parse_from(Self::MATH_SOURCE);

		sections.parse_from(Self::F32_ASSEMBLY_SOURCE);

		sections.parse_from(Self::I32_SOURCE);
		sections.parse_from(Self::I64_SOURCE);
		sections.parse_from(Self::F32_SOURCE);
		sections.parse_from(Self::F64_SOURCE);
		sections.parse_from(Self::TABLE_SOURCE);
		sections.parse_from(Self::MEMORY_SOURCE);

		sections.parse_from(Self::TURING_MACHINE_SOURCE);
		sections.parse_from(Self::WEB_ASSEMBLY_SOURCE);

		sections.resolve();

		sections
	}

	/// Parses sections from a source string.
	///
	/// # Panics
	///
	/// Panics if there is trailing unparsed data.
	pub fn parse_from(&mut self, mut source: &'static str) {
		while let Some((section, next)) = Section::try_parse(source) {
			self.list.push(section);

			source = next;
		}

		assert!(source.is_empty(), "trailing data in source\n{source}");
	}

	/// Sorts and validates sections, checking for duplicates.
	///
	/// # Panics
	///
	/// Panics if duplicate section names are found.
	pub fn resolve(&mut self) {
		self.list.sort_unstable_by_key(|&Section { name, .. }| name);

		for window in self.list.windows(2) {
			let Section { name: lhs, .. } = window[0];
			let Section { name: rhs, .. } = window[1];

			assert_ne!(lhs, rhs, "`{lhs}` section was duplicated");
		}
	}

	/// Finds a section by name.
	///
	/// # Panics
	///
	/// Panics if the section is not found.
	#[must_use]
	pub fn find(&self, name: &'static str) -> &Section {
		let Ok(position) = self
			.list
			.binary_search_by_key(&name, |&Section { name, .. }| name)
		else {
			unreachable!("`{name}` is not a section")
		};

		&self.list[position]
	}
}

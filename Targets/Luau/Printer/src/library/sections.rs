pub struct Section {
	pub references: Box<[&'static str]>,
	pub name: &'static str,
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

pub struct Sections {
	list: Vec<Section>,
}

impl Sections {
	pub const BASE_SOURCE: &str = include_str!("../../runtime/base.luau");
	pub const I32_SOURCE: &str = include_str!("../../runtime/i32.luau");
	pub const I64_SOURCE: &str = include_str!("../../runtime/i64.luau");
	pub const F32_SOURCE: &str = include_str!("../../runtime/f32.luau");
	pub const F64_SOURCE: &str = include_str!("../../runtime/f64.luau");
	pub const TABLE_SOURCE: &str = include_str!("../../runtime/table.luau");
	pub const MEMORY_SOURCE: &str = include_str!("../../runtime/memory.luau");

	#[must_use]
	pub fn with_built_ins() -> Self {
		let mut sections = Self { list: Vec::new() };

		sections.parse_from(Self::BASE_SOURCE);
		sections.parse_from(Self::I32_SOURCE);
		sections.parse_from(Self::I64_SOURCE);
		sections.parse_from(Self::F32_SOURCE);
		sections.parse_from(Self::F64_SOURCE);
		sections.parse_from(Self::TABLE_SOURCE);
		sections.parse_from(Self::MEMORY_SOURCE);
		sections.resolve();

		sections
	}

	pub fn parse_from(&mut self, mut source: &'static str) {
		while let Some((section, next)) = Section::try_parse(source) {
			self.list.push(section);

			source = next;
		}

		assert!(source.is_empty(), "trailing data in source\n{source}");
	}

	pub fn resolve(&mut self) {
		self.list.sort_unstable_by_key(|&Section { name, .. }| name);

		for window in self.list.windows(2) {
			let Section { name: lhs, .. } = window[0];
			let Section { name: rhs, .. } = window[1];

			assert_ne!(lhs, rhs, "`{lhs}` section was duplicated");
		}
	}

	pub fn find(&self, name: &'static str) -> &Section {
		let position = self
			.list
			.binary_search_by_key(&name, |&Section { name, .. }| name)
			.unwrap_or_else(|_| panic!("`{name}` is not a section"));

		&self.list[position]
	}
}

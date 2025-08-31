use alloc::vec::Vec;
use wasmparser::{
	BinaryReader, Data, Element, Export, FunctionBody, Global, Import, MemoryType, Parser, Payload,
	RecGroup, Result, SectionLimited, Table, TagType,
};

pub struct Sections<'data> {
	pub types: SectionLimited<'data, RecGroup>,
	pub imports: SectionLimited<'data, Import<'data>>,

	pub tables: SectionLimited<'data, Table<'data>>,
	pub elements: SectionLimited<'data, Element<'data>>,
	pub memories: SectionLimited<'data, MemoryType>,
	pub datas: SectionLimited<'data, Data<'data>>,
	pub globals: SectionLimited<'data, Global<'data>>,
	pub tags: SectionLimited<'data, TagType>,

	pub functions: SectionLimited<'data, u32>,
	pub code: Vec<FunctionBody<'data>>,

	pub start: Option<u32>,
	pub exports: SectionLimited<'data, Export<'data>>,
}

impl<'data> Sections<'data> {
	fn reader_with_empty<T>() -> SectionLimited<'static, T> {
		let reader = BinaryReader::new(&[0], 0);

		SectionLimited::new(reader).unwrap()
	}

	pub fn load(data: &'data [u8]) -> Self {
		let mut types = Self::reader_with_empty();
		let mut imports = Self::reader_with_empty();
		let mut functions = Self::reader_with_empty();
		let mut tables = Self::reader_with_empty();
		let mut memories = Self::reader_with_empty();
		let mut tags = Self::reader_with_empty();
		let mut globals = Self::reader_with_empty();
		let mut exports = Self::reader_with_empty();
		let mut start = None;
		let mut elements = Self::reader_with_empty();
		let mut datas = Self::reader_with_empty();
		let mut code = Vec::new();

		for payload in Parser::new(0).parse_all(data).map(Result::unwrap) {
			match payload {
				Payload::Version { .. }
				| Payload::CustomSection(_)
				| Payload::End(_)
				| Payload::DataCountSection { .. } => {}

				Payload::TypeSection(section) => types = section,
				Payload::ImportSection(section) => imports = section,
				Payload::FunctionSection(section) => functions = section,
				Payload::TableSection(section) => tables = section,
				Payload::MemorySection(section) => memories = section,
				Payload::TagSection(section) => tags = section,
				Payload::GlobalSection(section) => globals = section,
				Payload::ExportSection(section) => exports = section,
				Payload::StartSection { func, .. } => start = Some(func),
				Payload::ElementSection(section) => elements = section,
				Payload::DataSection(section) => datas = section,

				Payload::CodeSectionStart { count, .. } => {
					let count = count.try_into().unwrap();

					code.reserve_exact(count);
				}
				Payload::CodeSectionEntry(function_body) => code.push(function_body),

				payload => unimplemented!("{payload:?}"),
			}
		}

		Self {
			types,
			imports,
			tables,
			elements,
			memories,
			datas,
			globals,
			tags,
			functions,
			code,
			start,
			exports,
		}
	}
}

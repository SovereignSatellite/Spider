use datatest_stable::Result;
use wast::{
	QuoteWat, Wast, WastDirective, WastExecute, WastInvoke, WastRet, WastThread, Wat,
	lexer::Lexer,
	parser::{self, ParseBuffer},
	token::{Id, Span},
};

pub trait Visitor {
	fn visit_module(&mut self, quote_wat: QuoteWat<'_>) -> Result<()>;

	fn visit_module_definition(&mut self, quote_wat: QuoteWat<'_>) -> Result<()>;

	fn visit_module_instance(
		&mut self,
		span: Span,
		instance: Option<Id<'_>>,
		module: Option<Id<'_>>,
	) -> Result<()>;

	fn visit_assert_malformed(
		&mut self,
		span: Span,
		module: QuoteWat<'_>,
		message: &str,
	) -> Result<()>;

	fn visit_assert_invalid(
		&mut self,
		span: Span,
		module: QuoteWat<'_>,
		message: &str,
	) -> Result<()>;

	fn visit_register(&mut self, span: Span, name: &str, module: Option<Id<'_>>) -> Result<()>;

	fn visit_invoke(&mut self, wast_invoke: WastInvoke<'_>) -> Result<()>;

	fn visit_assert_trap(
		&mut self,
		span: Span,
		wast_execute: WastExecute<'_>,
		message: &str,
	) -> Result<()>;

	fn visit_assert_return(
		&mut self,
		span: Span,
		wast_execute: WastExecute<'_>,
		results: Vec<WastRet<'_>>,
	) -> Result<()>;

	fn visit_assert_exhaustion(
		&mut self,
		span: Span,
		call: WastInvoke<'_>,
		message: &str,
	) -> Result<()>;

	fn visit_assert_unlinkable(&mut self, span: Span, module: Wat<'_>, message: &str)
	-> Result<()>;

	fn visit_assert_exception(&mut self, span: Span, wast_execute: WastExecute<'_>) -> Result<()>;

	fn visit_assert_suspension(
		&mut self,
		span: Span,
		wast_execute: WastExecute<'_>,
		message: &str,
	) -> Result<()>;

	fn visit_thread(&mut self, wast_thread: WastThread<'_>) -> Result<()>;

	fn visit_wait(&mut self, span: Span, thread: Id<'_>) -> Result<()>;

	#[expect(
		clippy::too_many_lines,
		reason = "exhaustive match over wast directive variants"
	)]
	fn visit_directive(&mut self, directive: WastDirective<'_>) -> Result<()> {
		match directive {
			WastDirective::Module(quote_wat) => self.visit_module(quote_wat),
			WastDirective::ModuleDefinition(quote_wat) => self.visit_module_definition(quote_wat),
			WastDirective::ModuleInstance {
				span,
				instance,
				module,
			} => self.visit_module_instance(span, instance, module),
			WastDirective::AssertMalformed {
				span,
				module,
				message,
			}
			| WastDirective::AssertMalformedCustom {
				span,
				module,
				message,
			} => self.visit_assert_malformed(span, module, message),
			WastDirective::AssertInvalid {
				span,
				module,
				message,
			}
			| WastDirective::AssertInvalidCustom {
				span,
				module,
				message,
			} => self.visit_assert_invalid(span, module, message),
			WastDirective::Register { span, name, module } => {
				self.visit_register(span, name, module)
			}
			WastDirective::Invoke(wast_invoke) => self.visit_invoke(wast_invoke),
			WastDirective::AssertTrap {
				span,
				exec,
				message,
			} => self.visit_assert_trap(span, exec, message),
			WastDirective::AssertReturn {
				span,
				exec,
				results,
			} => self.visit_assert_return(span, exec, results),
			WastDirective::AssertExhaustion {
				span,
				call,
				message,
			} => self.visit_assert_exhaustion(span, call, message),
			WastDirective::AssertUnlinkable {
				span,
				module,
				message,
			} => self.visit_assert_unlinkable(span, module, message),
			WastDirective::AssertException { span, exec } => {
				self.visit_assert_exception(span, exec)
			}
			WastDirective::AssertSuspension {
				span,
				exec,
				message,
			} => self.visit_assert_suspension(span, exec, message),
			WastDirective::Thread(wast_thread) => self.visit_thread(wast_thread),
			WastDirective::Wait { span, thread } => self.visit_wait(span, thread),
		}
	}

	fn visit(&mut self, content: &str) -> Result<()> {
		let mut lexer = Lexer::new(content);

		lexer.allow_confusing_unicode(true);

		let buffer = ParseBuffer::new_with_lexer(lexer)?;
		let wast = parser::parse::<Wast<'_>>(&buffer)?;

		wast.directives
			.into_iter()
			.try_for_each(|directive| self.visit_directive(directive))
	}
}

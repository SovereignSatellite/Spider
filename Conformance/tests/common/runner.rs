use datatest_stable::Result;
use wast::{
	QuoteWat, Wast, WastDirective, WastExecute, WastInvoke, WastRet, WastThread, Wat,
	lexer::Lexer,
	parser::ParseBuffer,
	token::{Id, Span},
};

pub trait Runner {
	fn on_module(&mut self, quote_wat: QuoteWat) -> Result<()>;

	fn on_module_definition(&mut self, quote_wat: QuoteWat) -> Result<()>;

	fn on_module_instance(
		&mut self,
		span: Span,
		instance: Option<Id>,
		module: Option<Id>,
	) -> Result<()>;

	fn on_assert_malformed(&mut self, span: Span, module: QuoteWat, message: &str) -> Result<()>;

	fn on_assert_invalid(&mut self, span: Span, module: QuoteWat, message: &str) -> Result<()>;

	fn on_register(&mut self, span: Span, name: &str, module: Option<Id>) -> Result<()>;

	fn on_invoke(&mut self, wast_invoke: WastInvoke) -> Result<()>;

	fn on_assert_trap(&mut self, span: Span, exec: WastExecute, message: &str) -> Result<()>;

	fn on_assert_return(
		&mut self,
		span: Span,
		exec: WastExecute,
		results: Vec<WastRet>,
	) -> Result<()>;

	fn on_assert_exhaustion(&mut self, span: Span, call: WastInvoke, message: &str) -> Result<()>;

	fn on_assert_unlinkable(&mut self, span: Span, module: Wat, message: &str) -> Result<()>;

	fn on_assert_exception(&mut self, span: Span, exec: WastExecute) -> Result<()>;

	fn on_assert_suspension(&mut self, span: Span, exec: WastExecute, message: &str) -> Result<()>;

	fn on_thread(&mut self, wast_thread: WastThread) -> Result<()>;

	fn on_wait(&mut self, span: Span, thread: Id) -> Result<()>;

	fn on_directive(&mut self, directive: WastDirective) -> Result<()> {
		match directive {
			WastDirective::Module(quote_wat) => self.on_module(quote_wat),
			WastDirective::ModuleDefinition(quote_wat) => self.on_module_definition(quote_wat),
			WastDirective::ModuleInstance {
				span,
				instance,
				module,
			} => self.on_module_instance(span, instance, module),
			WastDirective::AssertMalformed {
				span,
				module,
				message,
			} => self.on_assert_malformed(span, module, message),
			WastDirective::AssertInvalid {
				span,
				module,
				message,
			} => self.on_assert_invalid(span, module, message),
			WastDirective::Register { span, name, module } => self.on_register(span, name, module),
			WastDirective::Invoke(wast_invoke) => self.on_invoke(wast_invoke),
			WastDirective::AssertTrap {
				span,
				exec,
				message,
			} => self.on_assert_trap(span, exec, message),
			WastDirective::AssertReturn {
				span,
				exec,
				results,
			} => self.on_assert_return(span, exec, results),
			WastDirective::AssertExhaustion {
				span,
				call,
				message,
			} => self.on_assert_exhaustion(span, call, message),
			WastDirective::AssertUnlinkable {
				span,
				module,
				message,
			} => self.on_assert_unlinkable(span, module, message),
			WastDirective::AssertException { span, exec } => self.on_assert_exception(span, exec),
			WastDirective::AssertSuspension {
				span,
				exec,
				message,
			} => self.on_assert_suspension(span, exec, message),
			WastDirective::Thread(wast_thread) => self.on_thread(wast_thread),
			WastDirective::Wait { span, thread } => self.on_wait(span, thread),
		}
	}

	fn run(&mut self, content: &str) -> Result<()> {
		let mut lexer = Lexer::new(content);

		lexer.allow_confusing_unicode(true);

		let buffer = ParseBuffer::new_with_lexer(lexer)?;
		let wast = wast::parser::parse::<Wast>(&buffer)?;

		wast.directives
			.into_iter()
			.try_for_each(|directive| self.on_directive(directive))
	}
}

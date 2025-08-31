use alloc::vec::Vec;
use data_flow_graph::{
	nested::{
		GammaIn, GammaOut, LambdaIn, LambdaOut, OmegaOut, RegionIn, RegionOut, ThetaIn, ThetaOut,
	},
	DataFlowGraph, Link, Node,
};
use hashbrown::HashMap;

use crate::{
	place::{Place, Table},
	reference_finder::{result_count_of, ReferenceFinder},
};

use self::{lifetime_finder::LifetimeFinder, local_provider::LocalProvider};

mod lifetime_finder;
mod local_finder;
mod local_provider;

// FIXME: This needs to be rewritten to be a backwards pass instead.
pub struct LocalAllocator {
	locals: Vec<u32>,

	finder: LifetimeFinder,
	provider: LocalProvider,
}

impl LocalAllocator {
	pub fn new() -> Self {
		Self {
			locals: Vec::new(),

			finder: LifetimeFinder::new(),
			provider: LocalProvider::new(),
		}
	}

	fn handle_operation(&mut self, node: &Node, id: u32, locals: &mut HashMap<Link, Place>) {
		let results = result_count_of(node);

		if results != 0 && self.locals.binary_search(&id).is_ok() {
			self.provider.pull_all_into(0..results, id, locals);
		}
	}

	fn handle_function_end(&mut self, input: u32, tables: &mut HashMap<u32, Table>) {
		let Some((name, len)) = self.provider.pop_function_scope() else {
			return;
		};

		tables.insert(input, Table { name, len });
	}

	fn handle_lambda_in(
		&mut self,
		id: u32,
		lambda_in: &LambdaIn,
		locals: &mut HashMap<Link, Place>,
	) {
		self.provider.push_function_scope();

		let dependencies = lambda_in.dependency_ports();

		self.provider.pull_all_into(dependencies, id, locals);

		assert!(
			self.provider.pop_function_scope().is_none(),
			"node {id} has too many upvalues"
		);

		self.provider.push_function_scope();

		let arguments = lambda_in.argument_ports();

		self.provider.pull_all_into(arguments, id, locals);
	}

	fn handle_lambda_out(
		&mut self,
		id: u32,
		lambda_out: &LambdaOut,
		tables: &mut HashMap<u32, Table>,
		locals: &mut HashMap<Link, Place>,
	) {
		let LambdaOut { input, .. } = *lambda_out;

		self.handle_function_end(input, tables);

		if self.locals.binary_search(&id).is_ok() {
			self.provider.pull_all_into(0..1, id, locals);
		}
	}

	fn handle_region_in(
		&mut self,
		graph: &DataFlowGraph,
		id: u32,
		region_in: &RegionIn,
		locals: &mut HashMap<Link, Place>,
	) {
		let GammaIn { arguments, .. } = graph.get(region_in.input).as_gamma_in().unwrap();

		self.provider.try_revive_all_into(arguments, id, locals);
		self.provider.push_local_scope();
	}

	fn handle_region_out(&mut self) {
		self.provider.pop_local_scope();
	}

	fn handle_gamma_in(&mut self) {
		self.provider.push_local_scope();
	}

	fn handle_gamma_out(
		&mut self,
		graph: &DataFlowGraph,
		id: u32,
		gamma_out: &GammaOut,
		locals: &mut HashMap<Link, Place>,
	) {
		let GammaOut { regions, .. } = gamma_out;
		let RegionOut { results, .. } = graph.get(regions[0]).as_region_out().unwrap();

		self.provider.pop_local_scope();
		self.provider.revive_all_into(results, id, locals);
	}

	fn handle_theta_in(&mut self, id: u32, theta_in: &ThetaIn, locals: &mut HashMap<Link, Place>) {
		let ThetaIn { arguments, .. } = theta_in;

		self.provider.try_revive_all_into(arguments, id, locals);
		self.provider.push_local_scope();
	}

	fn handle_theta_out(
		&mut self,
		id: u32,
		theta_out: &ThetaOut,
		locals: &mut HashMap<Link, Place>,
	) {
		let ThetaOut { results, .. } = theta_out;

		self.provider.pop_local_scope();
		self.provider.revive_all_into(results, id, locals);
	}

	fn handle_omega_in(&mut self, id: u32, locals: &mut HashMap<Link, Place>) {
		self.provider.push_function_scope();
		self.provider.pull_all_into(0..1, id, locals);
	}

	fn handle_omega_out(&mut self, omega_out: &OmegaOut, tables: &mut HashMap<u32, Table>) {
		let OmegaOut { input, .. } = *omega_out;

		self.handle_function_end(input, tables);
	}

	fn handle_node(
		&mut self,
		graph: &DataFlowGraph,
		id: u32,
		node: &Node,
		tables: &mut HashMap<u32, Table>,
		locals: &mut HashMap<Link, Place>,
	) {
		match node {
			Node::LambdaIn(lambda_in) => self.handle_lambda_in(id, lambda_in, locals),
			Node::LambdaOut(lambda_out) => self.handle_lambda_out(id, lambda_out, tables, locals),
			Node::RegionIn(region_in) => self.handle_region_in(graph, id, region_in, locals),
			Node::RegionOut(_) => self.handle_region_out(),
			Node::GammaIn(_) => self.handle_gamma_in(),
			Node::GammaOut(gamma_out) => self.handle_gamma_out(graph, id, gamma_out, locals),
			Node::ThetaIn(theta_in) => self.handle_theta_in(id, theta_in, locals),
			Node::ThetaOut(theta_out) => self.handle_theta_out(id, theta_out, locals),
			Node::OmegaIn(_) => self.handle_omega_in(id, locals),
			Node::OmegaOut(omega_out) => self.handle_omega_out(omega_out, tables),

			node => self.handle_operation(node, id, locals),
		}
	}

	pub fn run(
		&mut self,
		tables: &mut HashMap<u32, Table>,
		locals: &mut HashMap<Link, Place>,
		graph: &DataFlowGraph,
		reference_finder: &ReferenceFinder,
	) {
		local_finder::run(&mut self.locals, graph, reference_finder);

		self.finder
			.run(self.provider.lifetimes_mut(), graph, &self.locals);

		self.provider.push_function_scope();

		tables.clear();
		locals.clear();

		for (node, id) in graph.nodes().zip(0..) {
			self.provider.push_until(id);

			self.handle_node(graph, id, node, tables, locals);
		}

		assert!(
			self.provider.pop_function_scope().is_none(),
			"top level scope has too many locals"
		);
	}
}

use core::mem;

use ir_graph::{Link, Node, operation::Identity};

pub fn inline_once(nodes: &mut Vec<Node>, id: usize, body: Vec<Node>, inputs: &[Link]) {
	debug_assert!(
		matches!(
			body.last(),
			Some(Node::BranchResults(_) | Node::RepeatResults(_))
		),
		"the results boundary must be the body's last node"
	);

	let offset = u32::try_from(nodes.len()).unwrap();

	splice_body(nodes, body, offset, inputs);

	let last = nodes.len() - 1;
	nodes[id] = mem::replace(&mut nodes[last], Node::Trap);
}

fn splice_body(nodes: &mut Vec<Node>, body: Vec<Node>, offset: u32, inputs: &[Link]) {
	for node in body {
		nodes.push(relocate(node, offset, inputs));
	}
}

#[expect(
	clippy::wildcard_enum_match_arm,
	reason = "every non-boundary node relocates identically by shifting its outer links"
)]
fn relocate(node: Node, offset: u32, inputs: &[Link]) -> Node {
	match node {
		Node::BranchArguments(_) | Node::RepeatArguments(_) => Node::Identity(Identity {
			sources: inputs.iter().copied().collect(),
		}),
		Node::BranchResults(boundary) => shifted_identity(boundary.sources, offset),
		Node::RepeatResults(boundary) => shifted_identity(boundary.sources, offset),
		mut other => {
			other.for_each_mut_outer(|link| link.0 += offset);

			other
		}
	}
}

fn shifted_identity(sources: Vec<Link>, offset: u32) -> Node {
	Node::Identity(Identity {
		sources: sources
			.into_iter()
			.map(|link| Link(link.0 + offset, link.1))
			.collect(),
	})
}

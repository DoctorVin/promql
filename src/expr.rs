use vec::{instant_vec, LabelMatch};

#[derive(Debug)]
pub enum Op {
	Pow, // ^

	Mul, // *
	Div, // /
	Mod, // %

	Plus, // +
	Minus, // -

	Eq, // ==
	Ne, // !=
	Lt, // <
	Gt, // >
	Le, // <=
	Ge, // >=

	And, // and
	Unless, // unless

	Or, // or
}

#[derive(Debug)]
pub enum Node {
	Operator(Box<Node>, Op, Box<Node>),
	InstantVector(Vec<LabelMatch>),
}
impl Node {
	fn operator(x: Node, op: Op, y: Node) -> Node {
		Node::Operator(Box::new(x), op, Box::new(y))
	}
}

// ^ is right-associative, so we can actually keep it simple and recursive
named!(power <Node>, ws!(do_parse!(
	x: map!(instant_vec, Node::InstantVector) >>
	y: opt!(complete!(do_parse!(
		tag!("^") >>
		y: power >>
		(y)
	))) >>
	( match y {
		None => x,
		Some(y) => Node::operator(x, Op::Pow, y),
	} )
)));

// foo op bar op baz → Node[Node[foo op bar] op baz]
macro_rules! left_op {
	// $next is the parser for operator that takes precenence, or any other kind of non-operator token sequence
	($name:ident, $next:ident!($($next_args:tt)*), $op:ident!($($op_args:tt)*)) => (
		named!($name <Node>, ws!(do_parse!(
			x: $next!($($next_args)*) >>
			ops: many0!(do_parse!(
				op: $op!($($op_args)*) >>
				y: $next!($($next_args)*) >>
				((op, y))
			)) >>
			({
				let mut x = x;
				for (op, y) in ops {
					x = Node::operator(x, op, y);
				}
				x
			})
		)));
	);
	($name:ident, $next:ident, $op:ident!($($op_args:tt)*)) => ( left_op!(
		$name,
		call!($next),
		$op!($($op_args)*)
	); );
	($name:ident, $next:ident!($($next_args:tt)*), $op:ident) => ( left_op!(
		$name,
		$next!($($next_args)*),
		call!($op)
	); );
	($name:ident, $next:ident, $op:ident) => ( left_op!(
		$name,
		call!($next),
		call!($op)
	); );
}

left_op!(mul_div_mod, power, alt!(
	  tag!("*") => { |_| Op::Mul }
	| tag!("/") => { |_| Op::Div }
	| tag!("%") => { |_| Op::Mod }
));

left_op!(plus_minus, mul_div_mod, alt!(
	  tag!("+") => { |_| Op::Plus }
	| tag!("-") => { |_| Op::Minus }
));

// if you thing this kind of operator chaining makes little to no sense, think again: it actually matches 'foo' that is both '> bar' and '!= baz'.
// or, speaking another way: comparison operators are really just filters for values in a vector, and this is a chain of filters.
left_op!(comparison, plus_minus, alt!(
	  tag!("==") => { |_| Op::Eq }
	| tag!("!=") => { |_| Op::Ne }
	| tag!("<=") => { |_| Op::Le }
	| tag!(">=") => { |_| Op::Ge }
	| tag!("<")  => { |_| Op::Lt }
	| tag!(">")  => { |_| Op::Gt }
));

left_op!(and_unless, comparison, alt!(
	  tag!("and") => { |_| Op::And }
	| tag!("unless") => { |_| Op::Unless }
));

left_op!(or_op, and_unless, map!(tag!("or"), |_| Op::Or));

named!(pub expression <Node>, call!(or_op));

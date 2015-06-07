#![feature(plugin)]
#![plugin(clippy)]
#![deny(option_and_then_some)]
#![allow(redundant_closure)]

// the easiest case
fn and_then_should_be_map(x: Option<i32>) -> Option<i32> {
	x.and_then(Some) //~ERROR Consider using _.map(_)
}

// and an easy counter-example
fn really_needs_and_then(x: Option<i32>) -> Option<i32> {
	x.and_then(|o| if o < 32 { Some(o) } else { None })
}

// we don't yet care about Result, so this should compile
fn result_and_then_is_ok(x: Result<i32, ()>) -> Result<i32, ()> {
	x.and_then(Ok)
}

// this always returns None
fn to_none(_: i32) -> Option<i32> { None }

// helper function to add type information to f
fn check<F>(f: F, o: Option<i32>) where F: FnMut(i32) -> Option<i32> {
	o.and_then(f);
}

// need a main anyway, use it get rid of unused warnings too
fn main() {
	assert!(and_then_should_be_map(None).is_none());
	assert!(really_needs_and_then(Some(32)).is_none());
	assert!(result_and_then_is_ok(Ok(42)).is_ok());

	let x : Option<i32> = Some(42);
	x.and_then(to_none); // nonsense, but no error either
	// and the same with closure
	check(|_| None, x); // the same as above with closure
	
	x.and_then(|o| if o < 0 { Some(-o) } else { Some(o) }); //~ERROR Consider using _.map(_)
	x.and_then(|o| Some(o).and_then(|p| Some(p)));  
	//~^ERROR Consider using _.map(_)
					//~^^ERROR Consider using _.map(_)
}

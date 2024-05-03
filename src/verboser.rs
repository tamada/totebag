
pub trait Verboser {
    fn verbose(&self, message: String);
}

struct EmptyVerboser {
}

struct SimpleVerboser {
}

pub fn create_verboser(verbose: bool) -> Box<dyn Verboser> {
    if verbose {
        Box::new(SimpleVerboser{})
    } else {
        Box::new(EmptyVerboser{})
    }
}

impl Verboser for EmptyVerboser {
    fn verbose(&self, _msg: String) {
    }
}

impl Verboser for SimpleVerboser {
    fn verbose(&self, message: String) {
        println!("{}", message);
    }
}


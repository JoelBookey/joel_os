pub trait Program {
    fn run(&mut self) -> Result<(), &'static str>;
}

pub fn program_handler(prog: &mut impl Program) -> Result<(), &'static str> {
    prog.run()
}

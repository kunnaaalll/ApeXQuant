pub trait Optimizer {
    type Input;
    type Output;

    fn optimize(&mut self, input: Self::Input) -> Self::Output;
}

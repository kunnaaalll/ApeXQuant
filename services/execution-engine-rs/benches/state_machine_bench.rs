use criterion::{black_box, criterion_group, criterion_main, Criterion};
use execution_engine::state_machine::{StateMachine, states::OrderState, events::ExecutionEvent};

fn bench_state_transition(c: &mut Criterion) {
    c.bench_function("state transition pending -> submitted", |b| {
        b.iter(|| {
            StateMachine::transition(
                black_box(OrderState::Pending), 
                black_box(ExecutionEvent::Submit),
                black_box(1234567890)
            )
        })
    });
}

criterion_group!(benches, bench_state_transition);
criterion_main!(benches);

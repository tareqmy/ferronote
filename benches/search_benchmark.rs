use criterion::{Criterion, criterion_group, criterion_main};
use ferronote_search::Index;

fn build_mock_index(num_notes: usize) -> Index {
    let mut index = Index::new();
    for i in 0..num_notes {
        let title = format!("Test Note Number {}", i);
        let content = format!(
            "This is the content of test note {}. It has some random words like rusty, blazing, fast, terminal. And more text to make it realistic. We also need some lines to test highlighting.\nThis is a new line.\nAnother line.",
            i
        );
        index.add_note(format!("{}.md", title), content, 0);
    }
    index
}

fn bench_search(c: &mut Criterion) {
    let index = build_mock_index(10_000);

    c.bench_function("search 10000 notes", |b| {
        b.iter(|| {
            let _ = index.search("rusty terminal fast", "modified");
        })
    });
}

criterion_group!(benches, bench_search);
criterion_main!(benches);

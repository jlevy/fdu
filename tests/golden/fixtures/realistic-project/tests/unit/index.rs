use acorn::Index;

#[test]
fn a_new_index_uses_its_explicit_root() {
    let index = Index::new("fixtures/workspace");
    assert_eq!(index.root().to_string_lossy(), "fixtures/workspace");
    assert!(index.entries().is_empty());
}

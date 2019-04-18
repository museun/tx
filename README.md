# tx

## A simple 'Transaction' style smart pointer

Any changes made to the wrapped type will be reverted if `Tx::commit` wasn't called
when this is dropped. Changes back be rolled back to the last commit with `Tx::rollback`

## Example
```rust
#[derive(Clone)]
struct S { d: Vec<u32> }
impl S {
    fn tx(&mut self) -> Tx<'_, Self> {
        Tx::new(self)
    }
    fn add(&mut self, item: u32) {
        self.d.push(item);
    }
}

let mut s = S{ d: vec![] };
assert_eq!(s.d, vec![]);
{
    let mut s = s.tx();
    s.add(1);
    assert_eq!(s.d, vec![1]);
}
assert_eq!(s.d, vec![]);
{
    let mut s = s.tx();
    s.add(1);
    s.commit();
    assert_eq!(s.d, vec![1]);
}
assert_eq!(s.d, vec![1]);
{
    let mut s = s.tx();
    s.add(2);
    assert_eq!(s.d, vec![1, 2]);
    s.commit();
    s.add(3);
    assert_eq!(s.d, vec![1, 2, 3]);
    s.rollback();
    assert_eq!(s.d, vec![1, 2]);
}
assert_eq!(s.d, vec![1, 2]);
```

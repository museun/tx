/*!
# A simple 'Transaction' style smart pointer

Any changes made to the wrapped type will be reverted if [`Tx::commit`](./struct.Tx.html#method.commit) wasn't called
when this is dropped. Changes back be rolled back to the last commit with [`Tx::rollback`](./struct.Tx.html#method.rollback)

# Example
```
# use tx::Tx;
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
*/

/// A 'transaction' pointer
pub struct Tx<'a, T>(&'a mut T, T, bool);

impl<'a, T: Clone> Tx<'a, T> {
    /// Creates a new `Tx` by mutably borrowing a type
    pub fn new(d: &'a mut T) -> Self {
        let clone = d.clone();
        Self(d, clone, false)
    }
    /// Commits the changes
    pub fn commit(&mut self) {
        let Tx(scratch, initial, save) = self;
        std::mem::replace(initial, scratch.clone());
        *save = true;
    }
    /// Roll back to previous commit (or the initial state)
    ///
    /// This also acts like creating a new "subtranscation"
    pub fn rollback(&mut self) {
        let Tx(scratch, initial, save) = self;
        std::mem::replace(*scratch, initial.clone());
        *save = false
    }
}

impl<'a, T> Drop for Tx<'a, T> {
    fn drop(&mut self) {
        let Tx(scratch, initial, save) = self;
        if !*save {
            std::mem::swap(initial, *scratch);
        }
    }
}

impl<'a, T> std::ops::Deref for Tx<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> std::ops::DerefMut for Tx<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T> AsRef<T> for Tx<'a, T> {
    fn as_ref(&self) -> &T {
        &*self
    }
}

impl<'a, T> std::borrow::Borrow<T> for Tx<'a, T> {
    fn borrow(&self) -> &T {
        &*self
    }
}

impl<'a, T> std::borrow::BorrowMut<T> for Tx<'a, T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction() {
        #[derive(Clone, Debug)]
        struct S {
            d: Vec<u32>,
        }
        impl S {
            fn tx(&mut self) -> Tx<'_, Self> {
                Tx::new(self)
            }
            fn add(&mut self, item: u32) {
                self.d.push(item);
            }
            fn pop(&mut self) {
                self.d.pop();
            }
        }

        let mut s = S { d: vec![] };
        assert_eq!(s.d, vec![]);
        s.add(1);
        assert_eq!(s.d, vec![1]);
        {
            let mut s = s.tx();
            s.add(2);
            assert_eq!(s.d, vec![1, 2]);
        }
        assert_eq!(s.d, vec![1]);
        {
            let mut s = s.tx();
            s.add(2);
            assert_eq!(s.d, vec![1, 2]);
            s.commit();
            assert_eq!(s.d, vec![1, 2]);
        }
        assert_eq!(s.d, vec![1, 2]);
        {
            let mut s = s.tx();
            s.add(3);
            assert_eq!(s.d, vec![1, 2, 3]);
            s.rollback();
            assert_eq!(s.d, vec![1, 2]);
        }
        assert_eq!(s.d, vec![1, 2]);
        {
            let mut s = s.tx();
            s.pop();
            s.pop();
            assert_eq!(s.d, vec![]);
        }
        assert_eq!(s.d, vec![1, 2]);
        {
            let mut s = s.tx();
            s.pop();
            s.commit();
            assert_eq!(s.d, vec![1]);
            s.rollback(); // acts like a new transction
            s.pop();
            assert_eq!(s.d, vec![]);
        }
        assert_eq!(s.d, vec![1]);
    }
}

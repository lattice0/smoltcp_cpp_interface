/*
use managed::ManagedSlice;

pub struct RingBuffer<'a, T: 'a> {
    storage: ManagedSlice<'a, T>,
}

impl<'a, T: 'a> RingBuffer<'a, T> {
    pub fn new<S>(storage: S) -> RingBuffer<'a, T>
    where
        S: Into<ManagedSlice<'a, T>>,
    {
        RingBuffer {
            storage: storage.into(),
        }
    }
}
*/
/*
struct A<'a, 'b> {
    a: ManagedSlice<'a, RingBuffer<'b, u8>>,
}

impl<'a, 'b: 'a> A<'a, 'b> {
    pub fn new<T>(a: T) -> A<'a, 'b>
    where
        T: Into<ManagedSlice<'a, RingBuffer<'b, u8>>>,
    {
        let a = a.into();
        A { a: a }
    }
    pub fn push(&mut self) {
        let r = RingBuffer::new(vec![0, 1]);
        match self.a {
            ManagedSlice::Owned(ref mut ring) => {
                fn put<'b, 'c>(
                    index: usize,
                    slot: &mut RingBuffer<'b, u8>,
                    mut element: u8,
                ) {
                    *slot = element
                }
                ring.push(0);
                let index = ring.len() - 1;
                return put(index, &mut ring[index], 0);
            }
            _ => {}
        }
    }
}
*/
fn main() {
    //let mut s = A::new(vec![]);
    //let mut u: u32 = 5;
    //s.add(Some(&mut u));
    //let x = RingBuffer::new(vec![0,1]);
    //let r = RingBuffer::new(vec![0,1]);
    {
        
    }
    /*
    {
        match r.storage {
            ManagedSlice::Owned(mut ringBuffer) => {
                ringBuffer.push(0);
            }
            _ => {

            }
        }
    }
    */
}

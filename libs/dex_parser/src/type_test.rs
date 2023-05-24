use std::{cell::RefCell, rc::Rc};

struct A {
  data: [u8; 10],
  b: B,
}

struct B {
  env: Option<Rc<RefCell<dyn Env>>>,
}

trait Env {
  fn get_data(&self, idx: usize) -> u8;
}

impl Env for A {
  fn get_data(&self, idx: usize) -> u8 {
    self.data[idx]
  }
}

impl Env for B {
  fn get_data(&self, idx: usize) -> u8 {
    self.env.as_ref().unwrap().borrow().get_data(idx)
  }
}

impl A {
  fn new(data: [u8; 10]) -> Rc<RefCell<Self>> {
    let a = Rc::new(RefCell::new(Self {
      data,
      b: B { env: None },
    }));
    let b = B {
      env: Some(a.clone()),
    };
    a.borrow_mut().b = b;
    a
  }
}

#[cfg(test)]
mod test {
  #[test]
  fn test_new() {
    let a = super::A::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 0]);
    let b = &a.borrow().b;
    let c = b.env.as_ref().unwrap().borrow().get_data(0);
    assert!(c == 1)
  }
}


pub(crate) struct CartesianProductIterator<'a, I1, I2> {
  v1: &'a [I1],
  v2: &'a [I2],
  v1_idx: usize,
  v2_idx: usize
}

impl <'a, I1, I2> CartesianProductIterator<'a, I1, I2> {
  pub fn new(v1: &'a [I1], v2: &'a [I2]) -> Self {
    CartesianProductIterator { v1, v2, v1_idx: 0, v2_idx: 0 }
  }
}

impl <'a, I1: 'a, I2> Iterator for CartesianProductIterator<'a, I1, I2> {
  type Item = (&'a I1, &'a I2);

  fn next(&mut self) -> Option<Self::Item> {
    if self.v1.is_empty() || self.v2.is_empty() || self.v1_idx == self.v1.len() &&
      self.v2_idx == self.v2.len() {
      None
    } else if self.v2_idx == self.v2.len() {
      self.v2_idx = 1;
      self.v1_idx += 1;
      if self.v1_idx == self.v1.len() {
        None
      } else {
        Some((self.v1.get(self.v1_idx).unwrap(), self.v2.get(self.v2_idx - 1).unwrap()))
      }
    } else {
      self.v2_idx += 1;
      Some((self.v1.get(self.v1_idx).unwrap(), self.v2.get(self.v2_idx - 1).unwrap()))
    }
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use super::CartesianProductIterator;

  #[test]
  fn cartesian_product_iterator_empty_array_tests() {
    expect!(CartesianProductIterator::new(&Vec::<usize>::new(), &Vec::<usize>::new()).next()).to(be_none());
    expect!(CartesianProductIterator::new(&vec![1], &Vec::<usize>::new()).next()).to(be_none());
    expect!(CartesianProductIterator::new(&Vec::<usize>::new(), &vec![1]).next()).to(be_none());
  }

  #[test]
  fn cartesian_product_iterator_tests() {
    let vec1 = vec![1];
    let vec2 = vec![2];
    let mut i1 = CartesianProductIterator::new(&vec1, &vec2);
    expect!(i1.next()).to(be_some().value((&1, &2)));
    expect!(i1.next()).to(be_none());

    let vec3 = vec![1, 2];
    let mut i2 = CartesianProductIterator::new(&vec3, &vec2);
    expect!(i2.next()).to(be_some().value((&1, &2)));
    expect!(i2.next()).to(be_some().value((&2, &2)));
    expect!(i2.next()).to(be_none());

    let mut i3 = CartesianProductIterator::new(&vec1, &vec3);
    expect!(i3.next()).to(be_some().value((&1, &1)));
    expect!(i3.next()).to(be_some().value((&1, &2)));
    expect!(i3.next()).to(be_none());

    let vec4 = vec![2, 3];
    let mut i4 = CartesianProductIterator::new(&vec3, &vec4);
    expect!(i4.next()).to(be_some().value((&1, &2)));
    expect!(i4.next()).to(be_some().value((&1, &3)));
    expect!(i4.next()).to(be_some().value((&2, &2)));
    expect!(i4.next()).to(be_some().value((&2, &3)));
    expect!(i4.next()).to(be_none());
  }
}

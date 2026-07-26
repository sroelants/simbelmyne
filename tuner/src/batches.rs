use crate::data_entry::DataEntry;

/// A Batcher yields randomly shuffled batches of the requested batch size,
/// until all the data is exhauste (one epoch).
pub struct Batcher<'a> {
  entries: &'a [DataEntry],
  indices: Vec<usize>,
  batch_size: u32,
  batches_seen: u32,
}

impl<'a> Batcher<'a> {
  pub fn new(entries: &'a [DataEntry], batch_size: u32) -> Self {
    // TODO: Shuffle these indeces!
    let indices: Vec<usize> = (0..entries.len()).collect();

    Batcher {
      entries,
      indices,
      batch_size,
      batches_seen: 0,
    }
  }

  // NOTE: Can't make this an iterator, because the iterator interface won't
  // let you yield references to internal data (understandably).
  // Could probably store the entries in an Arc or Rc and make this work just
  // fine
  pub fn next(&'a mut self) -> Option<Batch<'a>> {
    if (self.batches_seen + 1) * self.batch_size <= self.entries.len() as u32 {
      let batch_start = (self.batches_seen * self.batch_size) as usize;
      let batch_end = batch_start + self.batch_size as usize;
      self.batches_seen += 1;

      let batch = Batch {
        indices: &self.indices[batch_start..batch_end],
        entries: self.entries,
      };

      Some(batch)
    } else {
      None
    }
  }
}

/// A wrapper around a randomly shuffled set of data entries, ready to be
/// iterated over.
pub struct Batch<'a> {
  indices: &'a [usize],
  entries: &'a [DataEntry],
}

impl<'a> IntoIterator for Batch<'a> {
  type Item = &'a DataEntry;
  type IntoIter = BatchIter<'a>;

  fn into_iter(self) -> Self::IntoIter {
    BatchIter {
      batch: self,
      idx: 0,
    }
  }
}

/// Iterator that yields references to the batch entries
pub struct BatchIter<'a> {
  batch: Batch<'a>,
  idx: usize,
}

impl<'a> Iterator for BatchIter<'a> {
  type Item = &'a DataEntry;

  fn next(&mut self) -> Option<Self::Item> {
    if self.idx < self.batch.indices.len() {
      let next_idx = self.batch.indices[self.idx];
      self.idx += 1;

      Some(&self.batch.entries[next_idx])
    } else {
      None
    }
  }
}

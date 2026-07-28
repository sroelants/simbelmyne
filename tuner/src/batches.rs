use crate::data_entry::DataEntry;

/// A Batcher yields randomly shuffled batches of the requested batch size,
/// until all the data is exhauste (one epoch).
pub struct Batcher<'a> {
  entries: &'a [DataEntry],
  indices: Vec<usize>,
  batch_size: u32,
}

impl<'a> Batcher<'a> {
  pub fn new(entries: &'a mut [DataEntry], batch_size: u32) -> Self {
    // TODO: Shuffle these indices!
    let mut indices: Vec<usize> = (0..entries.len()).collect();
    fastrand::shuffle(indices.as_mut());
    fastrand::shuffle(entries);

    Batcher {
      entries,
      indices,
      batch_size,
    }
  }

  pub fn iter(&'a self) -> BatcherIter<'a> {
    BatcherIter {
      batcher: &self,
      batches_yielded: 0,
    }
  }
}

pub struct BatcherIter<'a> {
  batcher: &'a Batcher<'a>,
  batches_yielded: u32,
}

impl<'a> Iterator for BatcherIter<'a> {
  type Item = Batch<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    if (self.batches_yielded + 1) * self.batcher.batch_size
      <= self.batcher.entries.len() as u32
    {
      let batch_start =
        (self.batches_yielded * self.batcher.batch_size) as usize;

      let batch_end = batch_start + self.batcher.batch_size as usize;

      self.batches_yielded += 1;

      let batch: Batch<'a> = Batch {
        indices: &self.batcher.indices[batch_start..batch_end],
        entries: &self.batcher.entries[batch_start..batch_end],
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
  pub indices: &'a [usize],
  pub entries: &'a [DataEntry],
}

impl<'a> Batch<'a> {
  pub fn iter(&'a self) -> BatchIter<'a> {
    BatchIter {
      batch: self,
      idx: 0,
    }
  }

  pub fn size(&self) -> usize {
    self.indices.len()
  }
}

/// Iterator that yields references to the batch entries
pub struct BatchIter<'a> {
  batch: &'a Batch<'a>,
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

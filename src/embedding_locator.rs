/// Trait to locate the indices where embedding should be performed.
///
/// This trait defines a method that returns an iterator over valid indices
/// in a host container (`host`) where embedding operations can be applied.
///
/// # Example
///
/// ```rust
/// use stegano_rs::embedding_locator::EmbeddingLocator;
/// #[derive(Clone)]
/// struct EveryOther;
///
/// impl EmbeddingLocator for EveryOther {
///     fn iter_indices<'a>(&'a self, host_len: usize) -> Box<dyn Iterator<Item = usize> + 'a> {
///         Box::new((0..host_len).step_by(2))
///     }
/// }
///
/// let locator = EveryOther;
/// let indices: Vec<usize> = locator.iter_indices(10).collect();
/// assert_eq!(indices, vec![0, 2, 4, 6, 8]);
/// ```  
pub trait EmbeddingLocator : EmbeddingLocatorClone{
    /// Returns an iterator over the valid indices in the `host`.
    ///
    /// # Arguments
    ///
    /// * `host_len` - The length of the host container in which to locate indices.
    ///
    /// # Returns
    ///
    /// An iterator over valid indices (`usize`) within the host.
    ///
    /// These indices correspond to positions where embedding operations can be performed.
    fn iter_indices<'a>(&'a self, host_len: usize) -> Box<dyn Iterator<Item = usize> + 'a>;
}
// Trick to allow cloning of a Box<dyn EmbeddingLocator>
pub trait EmbeddingLocatorClone {
    fn clone_box(&self) -> Box<dyn EmbeddingLocator>;
}

impl<T> EmbeddingLocatorClone for T
where
    T: 'static + EmbeddingLocator + Clone,
{
    fn clone_box(&self) -> Box<dyn EmbeddingLocator> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn EmbeddingLocator> {
    fn clone(&self) -> Box<dyn EmbeddingLocator> {
        self.clone_box()
    }
}


/// Implementation of `EmbeddingLocator` that performs a linear traversal
/// over all indices from 0 up to `host_len - 1`.
///
/// This strategy returns an iterator over all possible indices in the host container,
/// without skipping or filtering any.
///
/// # Example
///
/// ```rust
/// use stegano_rs::embedding_locator::EmbeddingLocator;
/// use stegano_rs::embedding_locator::LinearTraversal;
/// let locator = LinearTraversal;
/// let indices: Vec<usize> = locator.iter_indices(5).collect();
/// assert_eq!(indices, vec![0, 1, 2, 3, 4]);
/// ```
#[derive(Clone)]
pub struct LinearTraversal;

impl EmbeddingLocator for LinearTraversal {
    fn iter_indices<'a>(&'a self, host_len: usize) -> Box<dyn Iterator<Item = usize> + 'a> {
        Box::new(0..host_len)
    }
}

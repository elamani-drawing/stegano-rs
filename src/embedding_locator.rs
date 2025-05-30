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
/// impl<'a> EmbeddingLocator<'a> for EveryOther {
///     fn iter_indices(&'a self, host_len: usize) -> Box<dyn Iterator<Item = usize> + 'a> {
///         Box::new((0..host_len).step_by(2))
///     }
/// }
///
/// let locator = EveryOther;
/// let indices: Vec<usize> = locator.iter_indices(10).collect();
/// assert_eq!(indices, vec![0, 2, 4, 6, 8]);
/// ```  
pub trait EmbeddingLocator<'a> {
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
    fn iter_indices(&'a self, host_len: usize) -> Box<dyn Iterator<Item = usize> + 'a>;
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
pub struct LinearTraversal ;

impl<'a>  EmbeddingLocator<'a>  for LinearTraversal  {
    fn iter_indices(&'a self, host_len: usize) -> Box<dyn Iterator<Item = usize> + 'a> {
        Box::new(0..host_len)
    }
}

/// Traversal strategy based on a heatmap slice and a threshold.
/// Only indices where the heatmap value is >= threshold are returned.
///
/// # Example
///
/// ```rust
/// use stegano_rs::embedding_locator::{EmbeddingLocator, HeatmapTraversal};
///
/// let heatmap = [10, 50, 200, 30, 255];
/// let traversal = HeatmapTraversal {
///     heatmap: &heatmap,
///     threshold: 100,
/// };
/// let host_len = 5;
///
/// let indices: Vec<usize> = traversal.iter_indices(host_len).collect();
/// assert_eq!(indices, vec![2, 4]); // Only indices 2 and 4 have values >= 100
/// ```
#[derive(Debug, Clone)]
pub struct HeatmapTraversal<'a> {
    /// Reference to the heatmap slice used to guide embedding positions.
    pub heatmap: &'a [u8],
    /// Threshold to filter heatmap values.
    pub threshold: u8,
}

impl<'a> EmbeddingLocator<'a> for HeatmapTraversal<'a> {
    /// Returns an iterator over valid indices in the host buffer where
    /// the corresponding heatmap value is greater or equal to the threshold.
    ///
    /// # Arguments
    ///
    /// * `host_len` - The length of the host buffer. Indices >= host_len are ignored.
    ///
    /// # Returns
    ///
    /// An iterator over indices (`usize`) within the host satisfying the threshold condition.
    fn iter_indices(&self, host_len: usize) -> Box<dyn Iterator<Item = usize> + '_> {
        Box::new(
            self.heatmap
                .iter()
                .enumerate()
                .filter_map(move |(i, &val)| if i < host_len && val >= self.threshold { Some(i) } else { None })
        )
    }
}

/// Traversal strategy that returns a predefined list of positions.
///
/// This struct holds a slice of indices specifying exactly which positions
/// in the host buffer should be used for embedding.
///
/// # Example
///
/// ```rust
/// use stegano_rs::embedding_locator::{EmbeddingLocator, PositionListTraversal};
///
/// let positions = [1, 3, 5, 7];
/// let traversal = PositionListTraversal {
///     positions: &positions,
/// };
/// let host_len = 6;
///
/// let indices: Vec<usize> = traversal.iter_indices(host_len).collect();
/// assert_eq!(indices, vec![1, 3, 5]); // Position 7 is out of host bounds and ignored
/// ```
#[derive(Debug, Clone)]
pub struct PositionListTraversal<'a> {
    /// Reference to a slice containing positions to embed into.
    pub positions: &'a [usize],
}

impl<'a> EmbeddingLocator<'a> for PositionListTraversal<'a> {
    /// Returns an iterator over the predefined positions that are valid within the host buffer.
    ///
    /// # Arguments
    ///
    /// * `host_len` - The length of the host buffer. Positions outside this range are filtered out.
    ///
    /// # Returns
    ///
    /// An iterator over valid indices (`usize`) within the host buffer.
    fn iter_indices(&self, host_len: usize) -> Box<dyn Iterator<Item = usize> + '_> {
        Box::new(self.positions.iter().copied().filter(move |&pos| pos < host_len))
    }
}

//! Collection utilities for CENTRA-NF

/// Hitung elemen dalam slice
pub fn count<T>(items: &[T]) -> usize {
    items.len()
}

/// Cari elemen, return index
pub fn find<T: PartialEq>(items: &[T], target: &T) -> Option<usize> {
    items.iter().position(|x| x == target)
}

/// Filter koleksi dengan predikat
pub fn filter<T, F>(items: &[T], mut pred: F) -> Vec<&T>
where F: FnMut(&T) -> bool {
    items.iter().filter(|&x| pred(x)).collect()
}

/// Map koleksi
pub fn map<T, U, F>(items: &[T], mut f: F) -> Vec<U>
where F: FnMut(&T) -> U {
    items.iter().map(|x| f(x)).collect()
}

/// Unik (distinct)
pub fn unique<T: Eq + std::hash::Hash + Clone>(items: &[T]) -> Vec<T> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    items.iter().cloned().filter(|x| seen.insert(x.clone())).collect()
}

/// Chunks (bagi slice jadi potongan)
pub fn chunks<T>(items: &[T], size: usize) -> Vec<&[T]> {
    items.chunks(size).collect()
}

/// Zip dua slice
pub fn zip<T: Clone, U: Clone>(a: &[T], b: &[U]) -> Vec<(T, U)> {
    a.iter().cloned().zip(b.iter().cloned()).collect()
}

/// Sum (jumlah total)
pub fn sum<T>(items: &[T]) -> T
where T: std::iter::Sum<T> + Copy {
    items.iter().copied().sum()
}

/// Mean (rata-rata)
pub fn mean(items: &[f64]) -> Option<f64> {
    if items.is_empty() { None } else { Some(items.iter().sum::<f64>() / items.len() as f64) }
}

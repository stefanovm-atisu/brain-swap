//! Byte comparison for splices (TECHSPEC 4.8 R1).

/// Asserts that `after` is `before` with one span of whole lines swapped for
/// `expected_lines` (each with its line end), that span holding no more lines
/// than `expected_lines`, and every byte outside it identical.
pub fn assert_only_lines_changed(before: &[u8], after: &[u8], expected_lines: &[&str]) {
    let b: Vec<&[u8]> = before.split_inclusive(|&c| c == b'\n').collect();
    let a: Vec<&[u8]> = after.split_inclusive(|&c| c == b'\n').collect();
    let pre = b.iter().zip(&a).take_while(|(x, y)| x == y).count();
    let suf = b[pre..]
        .iter()
        .rev()
        .zip(a[pre..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let changed = String::from_utf8_lossy(&a[pre..a.len() - suf].concat()).into_owned();
    assert_eq!(changed, expected_lines.concat(), "changed lines differ");
    assert!(
        b.len() - suf - pre <= expected_lines.len(),
        "more lines removed than expected: {:?}",
        String::from_utf8_lossy(&b[pre..b.len() - suf].concat())
    );
}

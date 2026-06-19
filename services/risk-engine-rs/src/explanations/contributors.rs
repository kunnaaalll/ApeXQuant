use super::reasons::Reason;

pub fn find_largest_contributor(reasons: &[Reason]) -> Option<String> {
    if reasons.is_empty() {
        return None;
    }

    // Since reasons are already sorted by severity descending in reasons.rs,
    // the first one with severity > 0 is the largest contributor.
    if reasons[0].severity > 0 {
        Some(reasons[0].category.clone())
    } else {
        None
    }
}

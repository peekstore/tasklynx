use crate::model::{Status, Task};
use std::collections::HashSet;

const STOPWORDS: &[&str] = &[
    "a", "an", "and", "the", "to", "for", "of", "in", "on", "with", "please", "task",
];

pub fn similarity(left: &str, right: &str) -> f64 {
    let left_raw = left.trim().to_lowercase();
    let right_raw = right.trim().to_lowercase();
    let left = normalize(left);
    let right = normalize(right);
    if left.is_empty() || right.is_empty() {
        return if left_raw == right_raw { 1.0 } else { 0.0 };
    }
    if left == right {
        return 1.0;
    }

    let left_tokens: HashSet<&str> = left.split_whitespace().collect();
    let right_tokens: HashSet<&str> = right.split_whitespace().collect();
    let intersection = left_tokens.intersection(&right_tokens).count();
    let union = left_tokens.union(&right_tokens).count();
    let jaccard = intersection as f64 / union as f64;

    let max_length = left.chars().count().max(right.chars().count());
    let edit_similarity = if max_length == 0 {
        1.0
    } else {
        1.0 - levenshtein(&left, &right) as f64 / max_length as f64
    };

    jaccard.max(edit_similarity)
}

pub fn similar_to<'a>(
    task: &Task,
    tasks: &'a [Task],
    branch: &str,
    all: bool,
    threshold: f64,
) -> Vec<(&'a Task, f64)> {
    let mut similar: Vec<(&Task, f64)> = tasks
        .iter()
        .filter(|other| {
            other.id != task.id && other.status != Status::Done && (all || other.branch == branch)
        })
        .filter_map(|other| {
            let score = similarity(&task.title, &other.title);
            (score >= threshold).then_some((other, score))
        })
        .collect();
    similar.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.title.cmp(&b.0.title))
    });
    similar
}

pub fn similar_pairs<'a>(
    tasks: &'a [Task],
    branch: &str,
    all: bool,
    threshold: f64,
) -> Vec<(&'a Task, &'a Task, f64)> {
    let candidates: Vec<&Task> = tasks
        .iter()
        .filter(|task| task.status != Status::Done && (all || task.branch == branch))
        .collect();
    let mut pairs = Vec::new();
    for (index, left) in candidates.iter().enumerate() {
        for right in candidates.iter().skip(index + 1) {
            let score = similarity(&left.title, &right.title);
            if score >= threshold {
                pairs.push((*left, *right, score));
            }
        }
    }
    pairs.sort_by(|a, b| {
        b.2.partial_cmp(&a.2)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.title.cmp(&b.0.title))
    });
    pairs
}

fn normalize(title: &str) -> String {
    let lowered = title.to_lowercase();
    lowered
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| word.len() > 1 && !STOPWORDS.contains(word))
        .map(normalize_word)
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_word(word: &str) -> &str {
    match word {
        "bugs" | "issue" | "issues" | "defect" | "defects" | "error" | "errors" | "problem"
        | "problems" => "bug",
        "fixes" | "fixing" | "repair" | "repairs" | "resolve" | "resolves" => "fix",
        "adds" | "adding" | "implement" | "implements" | "implementation" | "create"
        | "creates" => "add",
        "removes" | "delete" | "deletes" => "remove",
        "tests" | "testing" | "tested" => "test",
        "doc" | "docs" | "documentation" => "docs",
        "authentication" | "authenticate" => "auth",
        "signin" => "login",
        "reviews" => "review",
        "improvement" | "improvements" => "improve",
        "updates" => "update",
        _ => word,
    }
}

fn levenshtein(left: &str, right: &str) -> usize {
    let left: Vec<char> = left.chars().collect();
    let right: Vec<char> = right.chars().collect();
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    let mut current = vec![0; right.len() + 1];
    for (i, left_char) in left.iter().enumerate() {
        current[0] = i + 1;
        for (j, right_char) in right.iter().enumerate() {
            let cost = usize::from(left_char != right_char);
            current[j + 1] = (previous[j + 1] + 1)
                .min(current[j] + 1)
                .min(previous[j] + cost);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[right.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Priority;

    fn task(title: &str) -> Task {
        Task::new(
            title.into(),
            "default".into(),
            "main".into(),
            Priority::Normal,
            vec![],
            None,
        )
    }

    #[test]
    fn normalizes_stopwords_and_synonyms() {
        assert_eq!(similarity("Fix the login bug", "Fix login issue"), 1.0);
        assert!(similarity("Fix login bug", "Fix login error") > 0.9);
        assert!(similarity("Fix login bug", "Write release notes") < 0.4);
    }

    #[test]
    fn similar_to_skips_done_and_other_branches() {
        let target = task("Fix login bug");
        let close = task("Fix login issue");
        let mut done = task("Fix login bug");
        done.status = Status::Done;
        let mut other = task("Fix login bug");
        other.branch = "feature".into();
        let tasks = vec![target.clone(), close.clone(), done, other];

        let matches = similar_to(&target, &tasks, "main", false, 0.7);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].0.id, close.id);

        let all = similar_to(&target, &tasks, "main", true, 0.7);
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn similar_pairs_ranks_matches() {
        let tasks = vec![
            task("Fix login bug"),
            task("Fix login issue"),
            task("Write release notes"),
        ];
        let pairs = similar_pairs(&tasks, "main", false, 0.7);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0.title, "Fix login bug");
        assert_eq!(pairs[0].1.title, "Fix login issue");
    }
}

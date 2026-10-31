use crate::model::Priority;

#[derive(Debug)]
pub struct Template {
    pub name: &'static str,
    pub title: &'static str,
    pub priority: Priority,
    pub tags: &'static [&'static str],
    pub description: &'static str,
    pub estimate_minutes: Option<u32>,
}

const TEMPLATES: &[Template] = &[
    Template {
        name: "review",
        title: "Review changes",
        priority: Priority::High,
        tags: &["review"],
        description:
            "Review checklist:\n\n- [ ] Read the diff\n- [ ] Check tests\n- [ ] Leave feedback",
        estimate_minutes: Some(30),
    },
    Template {
        name: "bug",
        title: "Fix bug",
        priority: Priority::High,
        tags: &["bug"],
        description: "Steps to reproduce:\n\nExpected:\n\nActual:",
        estimate_minutes: Some(60),
    },
    Template {
        name: "feature",
        title: "Implement feature",
        priority: Priority::Normal,
        tags: &["feature"],
        description: "Goal:\n\nAcceptance criteria:\n- [ ] ",
        estimate_minutes: Some(120),
    },
    Template {
        name: "test",
        title: "Add tests",
        priority: Priority::Normal,
        tags: &["test"],
        description: "Coverage goal:\n\n- [ ] ",
        estimate_minutes: Some(45),
    },
    Template {
        name: "chore",
        title: "Chore",
        priority: Priority::Normal,
        tags: &["chore"],
        description: "Task:\n\nDone when:\n- [ ] ",
        estimate_minutes: Some(30),
    },
];

pub fn find(name: &str) -> Option<&'static Template> {
    TEMPLATES
        .iter()
        .find(|template| template.name.eq_ignore_ascii_case(name))
}

pub fn names() -> String {
    TEMPLATES
        .iter()
        .map(|template| template.name)
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_templates_case_insensitively() {
        let review = find("Review").unwrap();
        assert_eq!(review.name, "review");
        assert_eq!(review.priority, Priority::High);
        assert_eq!(review.tags, &["review"]);
        assert!(names().contains("bug"));
        assert!(find("missing").is_none());
    }
}

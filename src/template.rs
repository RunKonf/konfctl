use std::collections::HashMap;

/// Replace `{{{VAR}}}` placeholders in text with values from the variables map.
#[allow(clippy::implicit_hasher)]
pub fn substitute_variables(text: &str, variables: &HashMap<String, String>) -> String {
    let mut result = text.to_string();
    for (key, value) in variables {
        let pattern = format!("{{{{{{{key}}}}}}}");
        result = result.replace(&pattern, value);
    }
    result
}

/// Find any unresolved `{{{VAR}}}` placeholders remaining in text.
/// Returns the variable names (without braces) that were not substituted.
pub fn find_unresolved_variables(text: &str) -> Vec<String> {
    let mut vars = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i + 6 < len {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' && bytes[i + 2] == b'{' {
            if let Some(end) = text[i + 3..].find("}}}") {
                let name = &text[i + 3..i + 3 + end];
                if !name.is_empty()
                    && name.bytes().all(|b| b.is_ascii_uppercase() || b == b'_')
                    && seen.insert(name.to_string())
                {
                    vars.push(name.to_string());
                }
                i = i + 3 + end + 3;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    vars
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    #[test]
    fn substitute_single_variable() {
        let v = vars(&[("SPONSOR_NAME", "Acme Corp")]);
        assert_eq!(
            substitute_variables("Hello {{{SPONSOR_NAME}}}!", &v),
            "Hello Acme Corp!"
        );
    }

    #[test]
    fn substitute_multiple_variables() {
        let v = vars(&[
            ("CONTACT_NAMES", "Jane and Bob"),
            ("CONFERENCE_TITLE", "CloudDays 2026"),
        ]);
        let text = "Dear {{{CONTACT_NAMES}}}, join {{{CONFERENCE_TITLE}}}!";
        assert_eq!(
            substitute_variables(text, &v),
            "Dear Jane and Bob, join CloudDays 2026!"
        );
    }

    #[test]
    fn substitute_repeated_variable() {
        let v = vars(&[("NAME", "Acme")]);
        assert_eq!(
            substitute_variables("{{{NAME}}} is {{{NAME}}}", &v),
            "Acme is Acme"
        );
    }

    #[test]
    fn substitute_no_match() {
        let v = vars(&[("OTHER", "value")]);
        assert_eq!(
            substitute_variables("Hello {{{SPONSOR_NAME}}}!", &v),
            "Hello {{{SPONSOR_NAME}}}!"
        );
    }

    #[test]
    fn substitute_empty_map() {
        let v = HashMap::new();
        assert_eq!(
            substitute_variables("Hello {{{NAME}}}!", &v),
            "Hello {{{NAME}}}!"
        );
    }

    #[test]
    fn substitute_preserves_markdown() {
        let v = vars(&[
            ("CONTACT_NAMES", "Jane"),
            ("CONFERENCE_URL", "https://example.com"),
        ]);
        let md = "Dear **{{{CONTACT_NAMES}}}**,\n\nVisit [us]({{{CONFERENCE_URL}}}).";
        assert_eq!(
            substitute_variables(md, &v),
            "Dear **Jane**,\n\nVisit [us](https://example.com)."
        );
    }

    #[test]
    fn find_unresolved_empty() {
        assert!(find_unresolved_variables("Hello world").is_empty());
    }

    #[test]
    fn find_unresolved_single() {
        assert_eq!(find_unresolved_variables("Hello {{{NAME}}}!"), vec!["NAME"]);
    }

    #[test]
    fn find_unresolved_multiple() {
        let result = find_unresolved_variables("{{{A}}} and {{{B}}} with {{{C}}}");
        assert_eq!(result, vec!["A", "B", "C"]);
    }

    #[test]
    fn find_unresolved_deduplicates() {
        let result = find_unresolved_variables("{{{NAME}}} is {{{NAME}}}");
        assert_eq!(result, vec!["NAME"]);
    }

    #[test]
    fn find_unresolved_ignores_double_braces() {
        assert!(find_unresolved_variables("Hello {{NAME}}!").is_empty());
    }

    #[test]
    fn find_unresolved_ignores_lowercase() {
        assert!(find_unresolved_variables("Hello {{{name}}}!").is_empty());
    }

    #[test]
    fn find_unresolved_after_substitution() {
        let v = vars(&[("SPONSOR_NAME", "Acme")]);
        let text = "Hello {{{SPONSOR_NAME}}}, {{{MISSING_VAR}}}!";
        let resolved = substitute_variables(text, &v);
        let unresolved = find_unresolved_variables(&resolved);
        assert_eq!(unresolved, vec!["MISSING_VAR"]);
    }
}

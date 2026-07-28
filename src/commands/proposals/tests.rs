use super::display::Filters;
use super::interactive::apply_filters;
use crate::types::{Proposal, ProposalFormat, ProposalSortBy, ProposalStatus, SortOrder};

fn make_proposal(id: &str, title: &str, status: &str, format: &str) -> Proposal {
    serde_json::from_value(serde_json::json!({
        "_id": id,
        "title": title,
        "status": status,
        "format": format,
        "speakers": [],
        "topics": [],
        "reviews": [],
    }))
    .unwrap()
}

fn make_proposal_with_speaker(id: &str, title: &str, status: &str, speaker: &str) -> Proposal {
    serde_json::from_value(serde_json::json!({
        "_id": id,
        "title": title,
        "status": status,
        "speakers": [{"_id": "s1", "name": speaker}],
        "topics": [],
        "reviews": [],
    }))
    .unwrap()
}

fn make_proposal_with_reviews(id: &str, title: &str, scores: &[(f64, f64, f64)]) -> Proposal {
    let reviews: Vec<serde_json::Value> = scores
        .iter()
        .map(|(c, r, s)| {
            serde_json::json!({
                "score": {"content": c, "relevance": r, "speaker": s},
                "reviewer": {"name": "Reviewer"}
            })
        })
        .collect();
    serde_json::from_value(serde_json::json!({
        "_id": id,
        "title": title,
        "status": "submitted",
        "speakers": [],
        "topics": [],
        "reviews": reviews,
    }))
    .unwrap()
}

fn test_proposals() -> Vec<Proposal> {
    vec![
        make_proposal("1", "Kubernetes Intro", "submitted", "presentation_40"),
        make_proposal("2", "Service Mesh", "accepted", "presentation_20"),
        make_proposal("3", "Lightning Demo", "submitted", "lightning_10"),
        make_proposal("4", "Workshop K8s", "rejected", "workshop_120"),
        make_proposal("5", "Observability", "confirmed", "presentation_40"),
    ]
}

#[test]
fn filter_by_status_submitted() {
    let proposals = test_proposals();
    let filters = Filters {
        statuses: vec![ProposalStatus::Submitted],
        ..Filters::default()
    };
    let result = apply_filters(&proposals, &filters);
    assert_eq!(result.len(), 2);
    assert!(result.iter().all(|p| p.status == ProposalStatus::Submitted));
}

#[test]
fn filter_by_multiple_statuses() {
    let proposals = test_proposals();
    let filters = Filters {
        statuses: vec![ProposalStatus::Accepted, ProposalStatus::Confirmed],
        ..Filters::default()
    };
    let result = apply_filters(&proposals, &filters);
    assert_eq!(result.len(), 2);
}

#[test]
fn filter_by_format() {
    let proposals = test_proposals();
    let filters = Filters {
        statuses: vec![],
        formats: vec![ProposalFormat::Presentation40],
        ..Filters::default()
    };
    let result = apply_filters(&proposals, &filters);
    assert_eq!(result.len(), 2);
    assert!(
        result
            .iter()
            .all(|p| p.format == Some(ProposalFormat::Presentation40))
    );
}

#[test]
fn filter_empty_statuses_shows_all() {
    let proposals = test_proposals();
    let filters = Filters {
        statuses: vec![],
        formats: vec![],
        ..Filters::default()
    };
    let result = apply_filters(&proposals, &filters);
    assert_eq!(result.len(), 5);
}

#[test]
fn filter_no_match_returns_empty() {
    let proposals = test_proposals();
    let filters = Filters {
        statuses: vec![ProposalStatus::Waitlisted],
        ..Filters::default()
    };
    let result = apply_filters(&proposals, &filters);
    assert!(result.is_empty());
}

#[test]
fn sort_by_title_asc() {
    let proposals = test_proposals();
    let filters = Filters {
        statuses: vec![],
        formats: vec![],
        sort_by: ProposalSortBy::Title,
        sort_order: SortOrder::Asc,
        ..Filters::default()
    };
    let result = apply_filters(&proposals, &filters);
    let titles: Vec<&str> = result.iter().map(|p| p.title.as_str()).collect();
    assert_eq!(
        titles,
        vec![
            "Kubernetes Intro",
            "Lightning Demo",
            "Observability",
            "Service Mesh",
            "Workshop K8s"
        ]
    );
}

#[test]
fn sort_by_title_desc() {
    let proposals = test_proposals();
    let filters = Filters {
        statuses: vec![],
        formats: vec![],
        sort_by: ProposalSortBy::Title,
        sort_order: SortOrder::Desc,
        ..Filters::default()
    };
    let result = apply_filters(&proposals, &filters);
    let titles: Vec<&str> = result.iter().map(|p| p.title.as_str()).collect();
    assert_eq!(titles[0], "Workshop K8s");
    assert_eq!(titles[4], "Kubernetes Intro");
}

#[test]
fn sort_by_speaker() {
    let proposals = vec![
        make_proposal_with_speaker("1", "Talk A", "submitted", "Charlie"),
        make_proposal_with_speaker("2", "Talk B", "submitted", "Alice"),
        make_proposal_with_speaker("3", "Talk C", "submitted", "Bob"),
    ];
    let filters = Filters {
        statuses: vec![],
        formats: vec![],
        sort_by: ProposalSortBy::Speaker,
        sort_order: SortOrder::Asc,
        ..Filters::default()
    };
    let result = apply_filters(&proposals, &filters);
    let speakers: Vec<&str> = result.iter().map(|p| p.speakers[0].name.as_str()).collect();
    assert_eq!(speakers, vec!["Alice", "Bob", "Charlie"]);
}

#[test]
fn sort_by_rating_desc() {
    let proposals = vec![
        make_proposal_with_reviews("1", "Low rated", &[(1.0, 1.0, 1.0)]),
        make_proposal_with_reviews("2", "High rated", &[(5.0, 5.0, 5.0)]),
        make_proposal_with_reviews("3", "Medium rated", &[(3.0, 3.0, 3.0)]),
    ];
    let filters = Filters {
        statuses: vec![],
        formats: vec![],
        sort_by: ProposalSortBy::Rating,
        sort_order: SortOrder::Desc,
        ..Filters::default()
    };
    let result = apply_filters(&proposals, &filters);
    let titles: Vec<&str> = result.iter().map(|p| p.title.as_str()).collect();
    assert_eq!(titles, vec!["High rated", "Medium rated", "Low rated"]);
}

#[test]
fn filter_and_sort_combined() {
    let proposals = test_proposals();
    let filters = Filters {
        statuses: vec![ProposalStatus::Submitted],
        formats: vec![],
        sort_by: ProposalSortBy::Title,
        sort_order: SortOrder::Asc,
        ..Filters::default()
    };
    let result = apply_filters(&proposals, &filters);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].title, "Kubernetes Intro");
    assert_eq!(result[1].title, "Lightning Demo");
}

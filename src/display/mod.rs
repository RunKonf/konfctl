mod proposal;
mod sponsor;
mod status;

pub use proposal::{pad_and_colorize_status, print_proposal_detail, render_proposal_detail};
pub use sponsor::{
    SPONSOR_TABLE_HEADER, format_sponsor_row, print_sponsor_detail, print_sponsor_history,
    print_sponsor_list, render_sponsor_detail,
};
pub use status::print_status;

pub fn print_agent_list<T: serde::Serialize>(
    data: T,
    total: usize,
    returned: usize,
) -> anyhow::Result<()> {
    let out = serde_json::json!({
        "data": data,
        "_meta": {
            "total": total,
            "returned": returned,
            "truncated": returned < total,
            "hint": "Use --limit or other filters to narrow results"
        }
    });
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}

pub fn print_json_list<T, C, F>(
    mut all: Vec<T>,
    limit: Option<usize>,
    compact: bool,
    json: bool,
    to_compact: F,
) -> anyhow::Result<Option<Vec<T>>>
where
    T: serde::Serialize,
    C: serde::Serialize,
    F: Fn(&T) -> C,
{
    let total = all.len();
    let effective_limit = limit.unwrap_or_else(|| if crate::is_agent() { 50 } else { usize::MAX });
    let returned = std::cmp::min(total, effective_limit);
    all.truncate(returned);

    if compact {
        let compact_data: Vec<C> = all.iter().map(to_compact).collect();
        if crate::is_agent() {
            print_agent_list(compact_data, total, returned)?;
        } else {
            println!("{}", serde_json::to_string_pretty(&compact_data)?);
        }
        return Ok(None);
    }

    if json || crate::is_agent() {
        if crate::is_agent() {
            print_agent_list(&all, total, returned)?;
        } else {
            println!("{}", serde_json::to_string_pretty(&all)?);
        }
        return Ok(None);
    }

    Ok(Some(all))
}

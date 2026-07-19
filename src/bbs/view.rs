use salyut_bbs::protocol::{Board, Post, PostSummary, Proposal, ProposalState};

use crate::web;

const PAGE: &str = include_str!("../../templates/bbs.html");

pub(super) fn index(boards: &[Board]) -> String {
    let boards = boards.iter().map(board_card).collect::<String>();
    page(
        "Message board",
        &format!(
            "<h1>Bulletin Board System</h1>\
             <p>Browse posts here, or log in over SSH and run <code>bbs</code> \
             to post, reply, and vote.</p><hr>\
             <h2>Boards</h2><ul class=\"boards\">{boards}</ul>"
        ),
    )
}

pub(super) fn board(board: &Board, posts: &[PostSummary]) -> String {
    let posts = if posts.is_empty() {
        "<p class=\"empty\">No posts yet.</p>".to_owned()
    } else {
        format!(
            "<ol class=\"posts\">{}</ol>",
            posts.iter().map(post_row).collect::<String>()
        )
    };
    page(
        &board.name,
        &format!(
            "<h1>{}</h1><p>{}</p><hr><h2>Posts</h2>{posts}",
            web::escape(&board.name),
            web::escape(&board.description),
        ),
    )
}

pub(super) fn post(post: &Post) -> String {
    let locked = if post.locked {
        "<p class=\"locked\">[Locked]</p>"
    } else {
        ""
    };
    let proposal = post
        .proposal
        .as_ref()
        .map(render_proposal)
        .unwrap_or_default();
    let poll = post
        .poll
        .as_ref()
        .map(|poll| {
            let options = poll
                .options
                .iter()
                .map(|option| {
                    let percent =
                        u64::from(option.votes) * 100 / u64::from(poll.total_votes.max(1));
                    format!(
                        "<li>{} — {} vote(s), {percent}%<br>\
                         <div class=\"poll-meter\" role=\"img\" aria-label=\"{percent} percent\">\
                         <span style=\"width: {percent}%\"></span></div></li>",
                        web::escape(&option.label),
                        option.votes,
                    )
                })
                .collect::<String>();
            let voting = if post
                .proposal
                .as_ref()
                .is_some_and(|proposal| proposal.state == ProposalState::Voting)
            {
                "Log in over SSH and run bbs to vote."
            } else {
                "Voting is closed."
            };
            format!(
                "<section class=\"poll\"><hr><h2>Poll results</h2><ul>{options}</ul>\
                 <p>{} total vote(s). {voting}</p></section>",
                poll.total_votes,
            )
        })
        .unwrap_or_default();
    let replies = if post.replies.is_empty() {
        "<p>No replies yet.</p>".to_owned()
    } else {
        post.replies
            .iter()
            .map(|reply| {
                format!(
                    "<article class=\"reply\" id=\"reply-{id}\"><p class=\"byline\">\
                     <a href=\"#reply-{id}\">#{id}</a> · @{author} · {date}</p>\
                     <pre>{body}</pre></article>",
                    id = reply.id,
                    author = web::escape(&reply.author),
                    date = reply.updated_at.format("%Y-%m-%d %H:%M UTC"),
                    body = web::escape(&reply.body),
                )
            })
            .collect()
    };
    let reply_status = if post.locked {
        "Replies are closed."
    } else {
        "Log in over SSH and run <code>bbs</code> to reply."
    };
    page(
        &post.title,
        &format!(
            "<h1>{title}</h1>{locked}<p class=\"byline\">\
             Posted by @{author} in {board} on {date} · #{id}</p>\
             <hr><pre>{body}</pre>{proposal}{poll}<section class=\"replies\"><hr>\
             <h2>Replies</h2>{replies}<p>{reply_status}</p></section>",
            id = post.id,
            title = web::escape(&post.title),
            author = web::escape(&post.author),
            board = web::escape(&post.board.name),
            date = post.updated_at.format("%Y-%m-%d %H:%M UTC"),
            body = web::escape(&post.body),
        ),
    )
}

fn board_card(board: &Board) -> String {
    let restriction = board
        .write_group
        .as_ref()
        .map(|group| {
            format!(
                " <small>(starting threads requires the {} group)</small>",
                web::escape(group)
            )
        })
        .unwrap_or_default();
    format!(
        "<li><a href=\"/bbs/boards/{slug}\">[{name}]</a> — \
         {description}{restriction}</li>",
        slug = web::escape(&board.slug),
        name = web::escape(&board.name),
        description = web::escape(&board.description),
    )
}

fn post_row(post: &PostSummary) -> String {
    let poll = if post.is_poll { " ◉" } else { "" };
    let proposal = post
        .proposal_state
        .map(|state| format!(" [{}]", state.label()))
        .unwrap_or_default();
    let locked = if post.locked { " [locked]" } else { "" };
    format!(
        "<li><a href=\"/bbs/posts/{id}\">{title}{poll}</a>{proposal}{locked} — \
         <span>@{author}, {date}, #{id} · {replies} repl{suffix}</span></li>",
        id = post.id,
        title = web::escape(&post.title),
        author = web::escape(&post.author),
        date = post.updated_at.format("%Y-%m-%d"),
        replies = post.reply_count,
        suffix = if post.reply_count == 1 { "y" } else { "ies" },
    )
}

fn render_proposal(proposal: &Proposal) -> String {
    let timing = if proposal.state == ProposalState::Voting {
        format!(
            "Voting closes {}.",
            proposal.closes_at.format("%Y-%m-%d %H:%M UTC")
        )
    } else {
        proposal.closed_at.map_or_else(String::new, |closed_at| {
            format!("Voting closed {}.", closed_at.format("%Y-%m-%d %H:%M UTC"))
        })
    };
    let history = proposal
        .events
        .iter()
        .map(|event| {
            let actor = event.actor.as_ref().map_or_else(
                || "system".to_owned(),
                |actor| {
                    event.actor_uid.map_or_else(
                        || format!("@{actor}"),
                        |uid| format!("@{actor} (uid {uid})"),
                    )
                },
            );
            let transition = event.from_state.map_or_else(
                || event.to_state.label().to_owned(),
                |from| format!("{} → {}", from.label(), event.to_state.label()),
            );
            let reason = event
                .reason
                .as_ref()
                .map(|reason| format!(" — {}", web::escape(reason)))
                .unwrap_or_default();
            format!(
                "<li>{} · {} · {}{reason}</li>",
                event.created_at.format("%Y-%m-%d %H:%M UTC"),
                web::escape(&transition),
                web::escape(&actor),
            )
        })
        .collect::<String>();
    format!(
        "<section class=\"proposal\"><hr><h2>Proposal: {}</h2><p>{timing}</p>\
         <h3>History</h3><ol>{history}</ol></section>",
        proposal.state.label(),
    )
}

pub(super) fn page(title: &str, content: &str) -> String {
    web::page(PAGE, title, content)
}

#[cfg(test)]
mod tests {
    use super::post;

    #[test]
    fn escapes_authored_content() {
        let fixture = serde_json::from_value(serde_json::json!({
            "id": 1,
            "board": {
                "id": 1,
                "slug": "general",
                "name": "General",
                "description": "",
                "kind": "discussion",
                "write_group": null
            },
            "author": "alice",
            "title": "<Hello>",
            "body": "<script>",
            "locked": false,
            "replies": [{
                "id": 2,
                "author": "bob",
                "body": "<reply>",
                "created_at": "2026-07-19T12:00:00Z",
                "updated_at": "2026-07-19T12:00:00Z"
            }],
            "poll": null,
            "proposal": null,
            "created_at": "2026-07-19T12:00:00Z",
            "updated_at": "2026-07-19T12:00:00Z"
        }))
        .unwrap();
        let html = post(&fixture);
        assert!(html.contains("<title>&lt;Hello&gt; · salyut.one BBS</title>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&lt;reply&gt;"));
        assert!(html.contains("href=\"/bbs\""));
    }
}

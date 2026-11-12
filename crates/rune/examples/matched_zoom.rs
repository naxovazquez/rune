//! A board zoomed and panned by `live_scale` / `live_offset`, like a canvas, with cards that open
//! into a detail panel over the window. Opening flies a card from its zoomed rect to the panel;
//! closing flies it back onto the board, which mounts again with it, and lands where the card
//! shows rather than at its unzoomed layout.
//!
//! Headless trace and motion audit (`scripts/motion-audit/run.sh` records both):
//!   cargo run -p rune --example matched_zoom -- --script scripts/motion-audit/scenarios/matched-zoom.json --trace t.jsonl --trace-all
use rune::{AppSpec, Color, Cx, El, Live, TextStyle, col, el, oklch, text};

struct State {
    open: Option<usize>,
    zoom: Live<f32>,
    pan: Live<[f32; 2]>,
}

const CARDS: [(&str, [f32; 2]); 3] = [("Ideas", [60.0, 60.0]), ("Projects", [380.0, 60.0]), ("Issues", [220.0, 260.0])];

fn card(i: usize, title: &str) -> El {
    col()
        .key(format!("card-{i}"))
        .matched(format!("card-{i}"))
        .padding(14.0)
        .radius(12.0)
        .fill(oklch(0.32, 0.03, 250.0))
        .border(1.0, oklch(0.46, 0.03, 250.0))
        .child(text(title).font(TextStyle::new(15.0, 600.0)).color(Color::WHITE))
}

fn view(s: &State, _: &mut Cx<State>) -> El {
    let root = col().w_full().h_full().fill(oklch(0.18, 0.01, 250.0));
    if let Some(i) = s.open {
        let panel = card(i, CARDS[i].0)
            .absolute(40.0, 40.0)
            .size(560.0, 320.0)
            .on_press(|s: &mut State| s.open = None);
        return root.child(panel);
    }
    let mut board = el().key("board").size(640.0, 400.0).origin(0.0, 0.0).live_offset(&s.pan).live_scale(&s.zoom);
    for (i, (title, [x, y])) in CARDS.iter().enumerate() {
        board = board.child(card(i, title).absolute(*x, *y).size(220.0, 110.0).on_press(move |s: &mut State| s.open = Some(i)));
    }
    root.child(board)
}

fn main() {
    rune::run(AppSpec::new(
        "Matched zoom",
        (640.0, 400.0),
        || State { open: None, zoom: Live::new(0.6), pan: Live::new([90.0, 50.0]) },
        view,
    ));
}

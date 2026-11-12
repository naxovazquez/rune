use super::*;
use crate::{Color, Key, Live, Mods};

/// A board zoomed by a `live_scale`, like a canvas, with a tile on it that opens over the
/// window. With `keep_board`, the board stays mounted while the tile is open; otherwise the
/// open tile replaces it, as a canvas gives way to a detail view.
struct Board {
    open: bool,
    keep_board: bool,
    zoom: Live<f32>,
    pan: Live<[f32; 2]>,
}

impl Board {
    fn new(keep_board: bool) -> Self {
        Self { open: false, keep_board, zoom: Live::new(0.5), pan: Live::new([20.0, 10.0]) }
    }
}

fn view(s: &Board, _: &mut Cx<Board>) -> El {
    let tile = || el().key("tile").matched("tile").fill(Color::WHITE);
    let mut board = el().key("board").size(400.0, 300.0).origin(0.0, 0.0).live_offset(&s.pan).live_scale(&s.zoom);
    if !s.open {
        board = board.child(tile().size(100.0, 50.0).absolute(120.0, 80.0));
    }
    let root = col().size(400.0, 300.0);
    match (s.open, s.keep_board) {
        (false, _) | (true, true) => root.child(board),
        (true, false) => root,
    }
    .child(if s.open { tile().size(300.0, 200.0).absolute(40.0, 40.0) } else { el() })
}

fn close(s: &mut Board, _: &KeyEvent, _: &mut Tx) -> bool {
    s.open = false;
    true
}

fn near(a: [f32; 4], b: [f32; 4], tol: f32) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() <= tol)
}

/// Opens the tile, closes it again, and returns where it rested on the board, where it was seen
/// open, and every frame of its flight home (60 Hz).
fn fly_home(keep_board: bool) -> ([f32; 4], [f32; 4], Vec<[f32; 4]>) {
    let mut rt = Runtime::new(Board::new(keep_board), view, (400.0, 300.0), Env::new(true, 1.0));
    rt.on_key = Some(close);
    rt.frame(0.0);
    for _ in 0..30 { rt.frame(1.0 / 60.0); }
    let rest = rt.presented("tile").expect("tile").0;
    rt.update(|s, _| s.open = true);
    for _ in 0..120 { rt.frame(1.0 / 60.0); }
    let open = rt.presented("tile").expect("tile").0;
    rt.key(KeyEvent { key: Key::Escape, mods: Mods::default() });
    let frames = (0..120).map(|_| {
        rt.frame(1.0 / 60.0);
        rt.presented("tile").expect("tile").0
    }).collect();
    (rest, open, frames)
}

#[test]
fn a_tile_on_a_zoomed_board_rests_at_its_zoomed_rect() {
    let (rest, open, _) = fly_home(false);
    assert_eq!(rest, [80.0, 50.0, 50.0, 25.0], "board offset plus half its layout");
    assert_eq!(open, [40.0, 40.0, 300.0, 200.0]);
}

/// The board mounts with the flight, its zoom not yet counted as still: the tile must still fly
/// to the size and place it lands at, rather than to its unzoomed layout and then jump.
#[test]
fn a_flight_home_onto_a_board_that_mounts_with_it_flies_to_where_the_tile_lands() {
    let (rest, open, frames) = fly_home(false);
    assert!(near(frames[0], open, 12.0), "starts where it was seen: {:?} vs {open:?}", frames[0]);
    for (k, r) in frames.iter().enumerate() {
        assert!(r[2] >= rest[2] - 1.0 && r[3] >= rest[3] - 1.0, "frame {k}: never smaller than it lands: {r:?}");
    }
    assert!(near(frames[36], rest, 2.0), "0.6s in it is about to land where it rests: {:?} vs {rest:?}", frames[36]);
    assert_eq!(frames[119], rest);
}

/// Opening, the tile leaves the zoomed board at the size and place it showed there, not at its
/// unzoomed layout.
#[test]
fn a_flight_off_a_zoomed_board_leaves_where_the_tile_showed() {
    for keep_board in [false, true] {
        let mut rt = Runtime::new(Board::new(keep_board), view, (400.0, 300.0), Env::new(true, 1.0));
        rt.frame(0.0);
        for _ in 0..30 { rt.frame(1.0 / 60.0); }
        let rest = rt.presented("tile").expect("tile").0;
        rt.update(|s, _| s.open = true);
        rt.frame(1.0 / 120.0);
        let first = rt.presented("tile").expect("tile").0;
        assert!(near(first, rest, 6.0), "keep_board {keep_board}: leaves where it showed: {first:?} vs {rest:?}");
        for _ in 0..120 { rt.frame(1.0 / 60.0); }
        assert_eq!(rt.presented("tile").expect("tile").0, [40.0, 40.0, 300.0, 200.0]);
    }
}

/// The board stays mounted with its zoom still: the flight must start where the tile was seen
/// open, not that rect run through the board's zoom.
#[test]
fn a_flight_home_onto_a_still_zoomed_board_starts_where_it_was_seen() {
    let (rest, open, frames) = fly_home(true);
    assert!(near(frames[0], open, 12.0), "starts where it was seen: {:?} vs {open:?}", frames[0]);
    assert!(near(frames[36], rest, 2.0), "0.6s in it is about to land where it rests: {:?} vs {rest:?}", frames[36]);
    assert_eq!(frames[119], rest);
}

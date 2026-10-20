use engine_native::Engine;

fn main() {
    for seats in [5usize, 6, 8] {
        let engine = Engine::new(20261018, seats);
        let mut mismatches = Vec::new();
        for a in 0..seats as u8 {
            for b in 0..seats as u8 {
                let actual = engine.state_core().seat_distance(a, b);
                let clockwise = (b as usize + seats - a as usize) % seats;
                let anticlockwise = (a as usize + seats - b as usize) % seats;
                let expected = clockwise.min(anticlockwise);
                if actual as usize != expected {
                    mismatches.push((a, b, actual, expected));
                }
            }
        }
        println!("seats={seats} ordered_pairs={} mismatches={} first_mismatches={:?}", seats * seats, mismatches.len(), &mismatches[..mismatches.len().min(10)]);
        for (a, b) in [(0, 1), (0, 3), (0, 4)] {
            let actual = engine.state_core().seat_distance(a, b);
            let clockwise = (b as usize + seats - a as usize) % seats;
            let anticlockwise = (a as usize + seats - b as usize) % seats;
            println!("  seat_distance({a},{b})={actual} expected={}", clockwise.min(anticlockwise));
        }
        if seats == 8 { assert!(mismatches.is_empty(), "8-seat baseline should match circular distances"); }
        else { assert!(!mismatches.is_empty(), "5/6-seat counterexample should reproduce"); }
    }
    let engine = Engine::new(20261018, 4);
    let mut seats: Vec<u8> = (0..4).collect();
    seats.sort_by_key(|seat| engine.state_core().seat_distance(0, *seat));
    println!("4-seat stable sort by shortest distance from seat 0 = {seats:?}; clockwise seat order = [0, 1, 2, 3]");
    assert_eq!(seats, [0, 1, 3, 2]);
    println!("Core distance arithmetic and 4-seat distance-order counterexamples confirmed.");
}

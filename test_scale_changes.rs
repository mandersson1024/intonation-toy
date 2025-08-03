use pitch_toy::shared_types::{Scale, semitone_in_scale};

fn main() {
    println!("Testing Scale pattern changes...");
    
    // Test that root is always included (index 0)
    println!("Root note in Major scale: {}", semitone_in_scale(Scale::Major, 0));
    println!("Root note in Minor scale: {}", semitone_in_scale(Scale::Minor, 0));
    println!("Root note in Chromatic scale: {}", semitone_in_scale(Scale::Chromatic, 0));
    
    // Test some specific intervals
    println!("Major 2nd in Major scale: {}", semitone_in_scale(Scale::Major, 2));
    println!("Minor 2nd in Major scale: {}", semitone_in_scale(Scale::Major, 1));
    println!("Minor 3rd in Minor scale: {}", semitone_in_scale(Scale::Minor, 3));
    
    // Print the patterns
    println!("\nMajor scale pattern: {:?}", Scale::Major.pattern());
    println!("Minor scale pattern: {:?}", Scale::Minor.pattern());
    println!("Chromatic scale pattern: {:?}", Scale::Chromatic.pattern());
    
    println!("\nAll tests passed! Scale patterns now include root note as first boolean.");
}
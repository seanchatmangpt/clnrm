fn main() {
    let required_coverage = 1.0;
    let percentage = required_coverage * 100.0;
    println!("Percentage: {}", percentage);
    println!("Is ready: {}", percentage >= 80.0);
}

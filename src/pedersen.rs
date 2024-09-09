use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::ristretto::CompressedRistretto;
use rand::rngs::OsRng;
use merlin::Transcript; // Add merlin crate to Cargo.toml

pub fn create_pedersen_commitment(amount: u64, blinding_factor: Scalar) -> CompressedRistretto {
    let pc_gens = PedersenGens::default();
    let m = Scalar::from(amount);

    // Compute the commitment: C = g^m * h^r
    let commitment = pc_gens.commit(m, blinding_factor);

    // Return the commitment as CompressedRistretto
    commitment.compress()
}

pub fn create_range_proof(amount: u64) -> (RangeProof, CompressedRistretto) {
    let mut rng = OsRng;

    // Generators for Pedersen commitments
    let pc_gens = PedersenGens::default();

    // Bulletproof generators, can prove up to 64-bit numbers
    let bp_gens = BulletproofGens::new(64, 1);

    // Amount to be committed (amount is secret)
    let value = amount;

    // Blinding factor (random scalar)
    let blinding_factor = Scalar::random(&mut rng);

    // Create the Pedersen commitment
    let commitment = create_pedersen_commitment(value, blinding_factor);

    // Create a Transcript for the proof
    let mut transcript = Transcript::new(b"RangeProof");

    // Create the range proof
    let (proof, _) = RangeProof::prove_single(
        &bp_gens,
        &pc_gens,
        &mut transcript,
        value,
        &blinding_factor,
        64,  // 64-bit range
    ).unwrap();

    // Return the proof and the commitment
    (proof, commitment)
}

pub fn verify_range_proof(proof: RangeProof, commitment: CompressedRistretto) -> bool {
    let pc_gens = PedersenGens::default();
    let bp_gens = BulletproofGens::new(64, 1);

    // Create a Transcript for the verification
    let mut transcript = Transcript::new(b"RangeProof");

    // Verify the proof
    let result = proof.verify_single(
        &bp_gens,
        &pc_gens,
        &mut transcript,
        &commitment,
        64,
    );

    result.is_ok()
}

fn main() {
    let amount = 100;

    // Create the Pedersen commitment and range proof
    let (proof, commitment) = create_range_proof(amount);

    // Output the commitment and proof
    println!("Pedersen Commitment for {} tokens: {:?}", amount, commitment);
    println!("Range Proof: {:?}", proof);

    // Verify the range proof
    let is_valid = verify_range_proof(proof, commitment);
    println!("Is the proof valid? {}", is_valid);
}

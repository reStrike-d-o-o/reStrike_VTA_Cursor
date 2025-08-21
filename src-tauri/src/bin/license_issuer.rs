use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use rand::rngs::OsRng;
use ring::signature::{self, KeyPair};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LicensePayload {
    product_id: String,
    machine_hash: String,
    issued_at: i64,
    expires_at: Option<i64>,
    plan: String,
    features: Vec<String>,
    nonce: String,
    version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LicenseToken {
    payload: LicensePayload,
    signature: String,
}

fn to_epoch_days_from_now(months: i64) -> Option<i64> {
    if months <= 0 { return None; }
    let now = Utc::now().timestamp();
    // Approximate month as 30 days
    Some(now + months * 30 * 86_400)
}

fn read_arg(name: &str) -> Option<String> {
    for a in std::env::args().skip(1) {
        if let Some((k, v)) = a.split_once('=') {
            if k == name { return Some(v.to_string()); }
        }
        if a == name { return Some(String::new()); }
    }
    None
}

fn save_to(path: &str, content: &str) {
    let p = PathBuf::from(path);
    if let Some(parent) = p.parent() { let _ = fs::create_dir_all(parent); }
    fs::write(&p, content).expect("Failed to write file");
    println!("Wrote {} ({} bytes)", path, content.len());
}

fn cmd_gen() {
    // Generate Ed25519 keypair using ring
    let rng = ring::rand::SystemRandom::new();
    let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng).expect("keygen failed");
    let keypair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).expect("from pkcs8");
    let public_key = keypair.public_key().as_ref().to_vec();
    let sk_b64 = general_purpose::STANDARD.encode(pkcs8_bytes.as_ref());
    let pk_b64 = general_purpose::STANDARD.encode(&public_key);
    println!("Private key (PKCS8, base64):\n{}", sk_b64);
    println!("Public key (base64, embed into app):\n{}", pk_b64);
    if let Some(out) = read_arg("out") { save_to(&out, &sk_b64); }
}

fn cmd_pub() {
    let sk_b64 = read_arg("sk").expect("Provide sk=<base64-pkcs8>");
    let sk = general_purpose::STANDARD.decode(sk_b64).expect("decode sk");
    let keypair = signature::Ed25519KeyPair::from_pkcs8(&sk).expect("from pkcs8");
    let pk_b64 = general_purpose::STANDARD.encode(keypair.public_key().as_ref());
    println!("Public key (base64):\n{}", pk_b64);
}

fn cmd_issue() {
    let product_id = read_arg("product").unwrap_or_else(|| "re-strike-vta".into());
    let machine_hash = read_arg("mh").expect("Provide mh=<machine_hash>");
    let plan = read_arg("plan").unwrap_or_else(|| "12m".into());
    let months: i64 = match plan.as_str() {
        "1m" => 1,
        "12m" => 12,
        "36m" => 36,
        "60m" => 60,
        "perpetual" => 0,
        _ => 12,
    };
    let expires_at = if months == 0 { None } else { to_epoch_days_from_now(months) };
    let features = vec![];
    let nonce = uuid::Uuid::new_v4().to_string();
    let payload = LicensePayload {
        product_id,
        machine_hash,
        issued_at: Utc::now().timestamp(),
        expires_at,
        plan,
        features,
        nonce,
        version: 1,
    };
    let payload_bytes = serde_json::to_vec(&payload).unwrap();
    let sk_b64 = read_arg("sk").expect("Provide sk=<base64-pkcs8>");
    let sk = general_purpose::STANDARD.decode(sk_b64).expect("decode sk");
    let keypair = signature::Ed25519KeyPair::from_pkcs8(&sk).expect("from pkcs8");
    let sig = keypair.sign(&payload_bytes);
    let token = LicenseToken { payload, signature: general_purpose::STANDARD.encode(sig.as_ref()) };
    let token_str = serde_json::to_string_pretty(&token).unwrap();
    println!("{}", token_str);
    if let Some(out) = read_arg("out") { save_to(&out, &token_str); }
}

fn cmd_fingerprint() {
    // Input: raw machine UID (e.g., from customer). Output: machine_hash used by app
    let uid = read_arg("uid").expect("Provide uid=<machine_uid>");
    let mut hasher = Sha256::new();
    hasher.update(uid.as_bytes());
    hasher.update(b"rst_vta_license_v1");
    let mh = format!("{:x}", hasher.finalize());
    println!("{}", mh);
}

fn main() {
    let cmd = std::env::args().nth(1).unwrap_or_else(|| "help".into());
    match cmd.as_str() {
        "gen" => cmd_gen(),                  // generate new keypair
        "pub" => cmd_pub(),                  // derive public key from private
        "issue" => cmd_issue(),              // issue signed token
        "fingerprint" => cmd_fingerprint(),  // compute machine_hash from UID
        _ => {
            eprintln!("Usage:\n  license-issuer gen [out=path]\n  license-issuer pub sk=<base64-pkcs8>\n  license-issuer issue sk=<base64-pkcs8> mh=<machine_hash> [product=...] [plan=1m|12m|36m|60m|perpetual] [out=path]\n  license-issuer fingerprint uid=<machine_uid>");
        }
    }
}



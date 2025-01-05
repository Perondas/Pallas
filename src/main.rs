use anyhow::{Context, Result};
use clap::Parser;
use hemtt_pbo::BISignVersion::V3;
use hemtt_pbo::ReadablePbo;
use hemtt_signing::BIPrivateKey;
use indicatif::ProgressBar;
use pallas::args::Args;
use pallas::helpers::addon_helpers::{check_contains_ebo, get_pbos_in_dir};
use pallas::helpers::fs_helpers::get_folder_name;
use pallas::helpers::mod_helpers::get_mod_dirs;
use pallas::state::State;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

fn main() -> Result<()> {
    let Args { source_dir } = Args::parse();

    let entries = std::fs::read_dir(&source_dir).context("Failed to read the source directory")?;

    let mods = get_mod_dirs(entries);

    let mut state = State::load(&source_dir).unwrap_or_default();

    println!("Found {:?} mods", state);

    for mod_path in mods.iter().map(|e| e.path()) {
        println!("Processing {:?}", mod_path.as_path(),);

        if check_contains_ebo(&mod_path) {
            eprintln!("Mod {} contains EBO files", get_folder_name(&mod_path)?);
            continue;
        }

        let pbos = match get_pbos_to_sign(&mod_path, &mut state) {
            Ok(pbos) => pbos,
            Err(e) => {
                eprintln!("Failed to get PBOS to sign: {:?}", e);
                continue;
            }
        };

        if pbos.is_empty() {
            println!("No new PBOS to sign in {:?}", mod_path.as_path());
            continue;
        }

        if let Err(e) = sign_pbos(&mod_path, pbos) {
            eprintln!("Failed to sign PBOS: {:?}", e);
            continue;
        }

        state.update(
            get_folder_name(&mod_path)?.to_string(),
            SystemTime::now() - Duration::from_secs(1),
        );

        println!("Done processing {:?}\n", mod_path.as_path());
    }

    state.save(&source_dir)?;

    println!("Done");

    Ok(())
}

fn get_pbos_to_sign(
    mod_folder_path: &Path,
    state: &mut State,
) -> Result<Vec<PathBuf>, anyhow::Error> {
    let addons_folder_path = mod_folder_path.join("addons");

    if !addons_folder_path.exists() {
        return Ok(vec![]);
    }

    let pbo_paths = get_pbos_in_dir(&addons_folder_path)?;

    if let Some(last_seen_modification) =
        state.modified(&get_folder_name(mod_folder_path)?)
    {
        let unseen_modifications = pbo_paths
            .iter()
            .filter_map(|p| p.metadata().ok().map(|m| m.modified().ok()))
            .flatten()
            .any(|m| m >= last_seen_modification);

        println!("Unseen modifications: {}", unseen_modifications);
        println!("Last seen modification: {:?}", last_seen_modification);

        if !unseen_modifications {
            return Ok(vec![]);
        }
    }

    Ok(pbo_paths)
}

fn sign_pbos(mod_folder_path: &Path, pbo_paths: Vec<PathBuf>) -> Result<()> {
    println!("Signing {} PBOs", pbo_paths.len());

    let authority =
        "pallas_".to_string() + get_folder_name(mod_folder_path)?.trim_start_matches('@');

    let key = BIPrivateKey::generate(2048, &authority).expect("can't generate private key");

    let pb = ProgressBar::new(pbo_paths.len() as u64);

    let k2 = key.clone();
    let a2 = authority.clone();
    let pb2 = pb.clone();
    rayon::scope(move |s| {
        for pbo_path in pbo_paths {
            let key = k2.clone();
            let authority = a2.clone();
            let pb = pb2.clone();
            s.spawn(move |_| {
                if let Err(e) = sign_pbo(&pbo_path, &key, &authority) {
                    eprintln!("Failed to sign {:?}: {:#?}", pbo_path, e);
                }
                pb.inc(1);
            });
        }
    });
    pb.finish();

    let public_key_path = mod_folder_path
        .join("keys")
        .join(format!("{}.bikey", authority));
    let mut public_key_file = File::create(&public_key_path)?;
    key.to_public_key().write(&mut public_key_file)?;

    Ok(())
}

fn sign_pbo(pbo_path: &Path, key: &BIPrivateKey, authority: &str) -> Result<()> {
    let file = File::open(pbo_path)?;
    let mut pbo = ReadablePbo::from(file)?;

    let signature = key.sign(&mut pbo, V3)?;

    let signature_path = pbo_path.with_extension(format!("pbo.{}.bisign", authority));

    let mut signature_file = File::create(&signature_path)?;
    signature.write(&mut signature_file)?;
    Ok(())
}

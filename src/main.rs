mod options;

use std::fs::{self, File};
use std::io::BufReader;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use log::{debug, info, warn};
use regex::{Captures, Regex};
use serde_json::Value;
use structopt::StructOpt;

use crate::options::Options;

fn main() -> Result<()> {
    env_logger::init();

    let options = Options::from_args();
    let results = run(&options.input)?;
    debug!("{} files read", results.len());
    for result in results {
        if result.is_ok() {
            write_file(&result?, &options);
        } else {
            warn!("{:?}", result.err());
        }
    }
    Ok(())
}

fn load_file(path: &PathBuf) -> Result<FileData> {
    let file = File::open(path)?;
    let reader: BufReader<File> = BufReader::new(file);
    Ok(FileData {
        value: serde_json::from_reader(reader)?,
        filename: path
            .file_name()
            .expect("Expect file should have a filename")
            .to_owned(),
    })
}

fn run(path: &Path) -> Result<Vec<Result<FileData>>> {
    let mut results: Vec<Result<FileData>> = Vec::new();
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            debug!("Reading path: {:?}", &path);
            if !path.is_dir() {
                results.push(load_file(&path));
            }
        }
    } else {
        results.push(load_file(&path.to_path_buf()));
    }
    if results.len() < 1 {
        return Err(anyhow!("Input was not a folder"));
    }
    Ok(results)
}

struct FileData {
    value: Value,
    filename: OsString,
}

/// Take
fn run_regex(data: String) -> String {
    lazy_static! {
        static ref DICE_OR_DAMAGE: Regex = Regex::new(r#"\{@(?:(?:dice)|(?:damage)) ((?:(?:(?:\d+)?d\d+(?: ?[\+-] ?\d+)?)(?: ?[\+-]? ?))+)\}"#).unwrap();
        static ref COMPUTED_DICE: Regex = Regex::new(r#"\{@dice (?:(?:\d+)?d\d+(?: ?[\+-] ?\d+)?)\|(\d+)\}"#).unwrap();
        static ref MULTIPLICATIVE_DICE: Regex = Regex::new(r#"\{@dice (\d*d\d+ × (?:\d*d?\d*))\}"#).unwrap();
        static ref SCALE: Regex = Regex::new(
            r#"\{@(?:(?:scaledamage)|(?:scaledice)) (?:(?:(?:(?:\d+)?d\d+(?: ?[\+-] ?\d+)?)(?: ?[\+-]? ?))+)(?:(?:(?:;(?:\d+)?d\d+(?: ?[\+-] ?\d+)?)(?: ?[\+-]? ?))+)?\|(?:(?:\d+-\d+)|(?:(?:,?\d+)*))\|((?:(?:(?:\d+)?d\d+(?: ?[\+-] ?\d+)?)(?: ?[\+-]? ?))+)(?:\|[\w\s]+)?\}"#,
        ).unwrap();

        static ref HIT: Regex = Regex::new(r#"\{@h(?:it)? ((?:\+|-)?\d+)\}"#).unwrap();
        static ref CHANCE: Regex = Regex::new(r#"\{@chance (\d+)\}"#).unwrap(); // 33 percent
        static ref RECHARGE: Regex = Regex::new(r#"\{@recharge(?: (\d))?\}"#).unwrap(); // Recharge 4-6, Recharge 5-6  or Recharge 6 (exactly 6!)
        static ref HEALTH: Regex = Regex::new(r#"\{@h\}(\d+)?"#).unwrap();
        static ref DC: Regex = Regex::new(r#"\{@dc (\d+)\}"#).unwrap();

        static ref BOLD: Regex = Regex::new(r#"\{@b(?:old)? ([\w\s'().?!\-]+)\}"#).unwrap(); // Make this ** **
        static ref ITALIC: Regex = Regex::new(r#"\{@i(?:talic)? ([\w\s'().?!\-]+)\}"#).unwrap(); // Make this _ _
        static ref STRIKE: Regex = Regex::new(r#"\{@s(?:trike)? ([\w\s'().?!\-]+)\}"#).unwrap(); // Make this ~ ~

        static ref NOTE: Regex = Regex::new(r#"\{@note ([\w\s.!?,\|='\{@\}]+)\}"#).unwrap();

        // TODO: Add background,race,optfeature,condition,disease,reward,trap,hazard,feat,psionic,object,boon,cult,variant,vehicle,
        // e.g. {@spell name} {@spell name|source}
        // Item, creature, spell, skill, sense, background, race, optional feature, condition, disease, reward, trap, hazard, feat, psionic, object, boon, cult, variant, vehicle, table, deity, action, language
        static ref UNLABELLED_GENERIC_LINK: Regex =
            Regex::new(r#"\{@(?:(?:spell)|(?:item)|(?:creature)|(?:background)|(?:race)|(?:optfeature)|(?:condition)|(?:disease)|(?:reward)|(?:alert)|(?:psionic)|(?:object)|(?:boon)|(?:hazard)|(?:variantrule)|(?:vehicle)|(?:table)|(?:action)|(?:sense)|(?:skill)) ([\w\s'()\-+,]+)(?:\|[\w\s'()\-+,]*)?\}"#).unwrap();

        // e.g. {@spell name|source|the actual text to display}
        // Item, creature, spell, skill, sense, background, race, optional feature, condition, disease, reward, trap, hazard, feat, psionic, object, boon, cult, variant, vehicle, table, deity, action, language
        static ref LABELLED_GENERIC_LINK: Regex =
            Regex::new(r#"\{@(?:(?:spell)|(?:item)|(?:creature)|(?:background)|(?:race)|(?:optfeature)|(?:condition)|(?:disease)|(?:reward)|(?:alert)|(?:psionic)|(?:object)|(?:boon)|(?:hazard)|(?:variantrule)|(?:vehicle)|(?:table)|(?:action)) (?:[\w\s'()\-+,]*\|){2}([\w\s'()\-+,]+)\}"#).unwrap();
        static ref FILTER: Regex = Regex::new(r#"\{@filter ([\w\s'()\-/+]+)(?:\|[\w\s'!=;()&\[\]/+]+)*\}"#).unwrap();

        // TODO: deity
        // TODO: class
        // TODO: check if we can handle book_or_adventure separaely
        static ref book_or_adventure: Regex =
            Regex::new(r#"\{@(?:(?:book)|(?:adventure)) ([\w\s'()\-+]+)(?:\|[\w\s'()\-+\d]+)*\}"#).unwrap();

        static ref MELEE: Regex = Regex::new(r#"\{@atk m\}"#).unwrap(); // _Melee Attack_
        static ref MELEE_WEAPON: Regex = Regex::new(r#"\{@atk mw\}"#).unwrap(); // _Melee Weapon Attack_
        static ref MELEE_SPELL: Regex = Regex::new(r#"\{@atk ms\}"#).unwrap(); // _Melee or Ranged Weapon Attack_
        static ref MELEE_OR_RANGED_WEAPON: Regex = Regex::new(r#"\{@atk mw,rw\}"#).unwrap(); // _Melee or Ranged Weapon Attack_
        static ref RANGED_SPELL: Regex = Regex::new(r#"\{@atk rs\}"#).unwrap(); // _Ranged Spell Attack_
        static ref RANGED_WEAPON: Regex = Regex::new(r#"\{@atk rw\}"#).unwrap(); // _Ranged Weapon Attack_
        static ref MELEE_SPELL_OR_RANGED_SPELL: Regex = Regex::new(r#"\{@atk ms,rs\}"#).unwrap(); // _Melee or Ranged Spell  Attack_

    }
    let data = DICE_OR_DAMAGE.replace_all(&data, "**${1}**");
    let data = COMPUTED_DICE.replace_all(&data, "$1");
    let data = MULTIPLICATIVE_DICE.replace_all(&data, "$1");
    let data = SCALE.replace_all(&data, "$1");
    let data = HIT.replace_all(&data, "_${1}_");
    let data = CHANCE.replace_all(&data, "${1} percent");
    let data = HEALTH.replace_all(&data, |caps: &Captures| match caps.get(1) {
        None => "".to_owned(),
        Some(damage) => damage.as_str().to_owned(),
    });
    // {@recharge 4} -> Recharge 4-6; {@recharge 5} -> Recharge 5-6; {@recharge} -> Recharge 6
    let data = RECHARGE.replace_all(&data, |caps: &Captures| match caps.get(1) {
        None => "Recharge 6".to_owned(),
        Some(recharge_count) => format!("Recharge {}-6", recharge_count.as_str()),
    });
    let data = DC.replace_all(&data, "DC ${1}");

    let data = BOLD.replace_all(&data, "**${1}**");
    let data = ITALIC.replace_all(&data, "_${1}_");
    let data = STRIKE.replace_all(&data, "~$1~");
    let data = NOTE.replace_all(&data, "$1");
    let data = UNLABELLED_GENERIC_LINK.replace_all(&data, "_${1}_");
    let data = LABELLED_GENERIC_LINK.replace_all(&data, "_${1}_");
    let data = FILTER.replace_all(&data, "${1}");

    let data = MELEE.replace_all(&data, "Melee Attack");
    let data = MELEE_WEAPON.replace_all(&data, "Melee Weapon Attack");
    let data = MELEE_SPELL.replace_all(&data, "Melee Spell Attack");
    let data = MELEE_OR_RANGED_WEAPON.replace_all(&data, "Melee or Ranged Weapon Attack");
    let data = RANGED_SPELL.replace_all(&data, "Ranged Spell Attack");
    let data = RANGED_WEAPON.replace_all(&data, "Ranged Weapon Attack");
    let data = MELEE_SPELL_OR_RANGED_SPELL.replace_all(&data, "Melee or Ranged Spell Attack");
    data.into_owned()
}

fn write_file(data_wrapper: &FileData, options: &Options) {
    let data = &data_wrapper.value[&options.key];
    if data.as_null().is_some() {
        info!("Skipping {}", &data_wrapper.filename.to_string_lossy());
        return;
    }
    let data = data.to_string();
    let data = run_regex(data);
    let mut path = Path::new(&options.output).to_path_buf();
    path.push(&data_wrapper.filename);
    info!("Writing file to {}", &path.to_string_lossy());
    fs::write(&path, data).expect(&format!(
        "Unable to write file to {}",
        &path.to_string_lossy()
    ));
}

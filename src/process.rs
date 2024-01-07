use anyhow::{anyhow, Result};
use csv::WriterBuilder;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{event, Level};

#[derive(Debug, Deserialize, Serialize, Default)]
enum SkillType {
    #[default]
    #[serde(rename = "MeleeCombatSkill")]
    Melee,
    #[serde(rename = "RangedCombatSkill")]
    Ranged,
    #[serde(rename = "UtilitySkill")]
    Utility,
    #[serde(rename = "SpellSkill")]
    Spell,
    #[serde(rename = "HealingSkill")]
    Healing,
}

#[derive(Debug, Deserialize, Serialize, Default)]
enum DamageType {
    #[default]
    #[serde(rename = "Physical/Slashing")]
    PhysicalSlashing,
    Magical,
}

#[derive(Debug, Deserialize, Serialize)]
enum Reagents {
    SparklingPowder,
    Blood,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
enum ProficiencyLevel {
    #[serde(rename = "novice")]
    Novice,
    #[serde(rename = "adept")]
    Adept,
    #[serde(rename = "master")]
    Master,
}

#[derive(Debug, Deserialize, Serialize)]
enum Unit {
    #[serde(rename = "meters")]
    Meters,
    #[serde(rename = "degrees")]
    Degrees,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
enum Debuff {
    RiskOfCounterAttack,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct SkillRequirements {
    #[serde(alias = "requirements.actions")]
    actions: u8,
    #[serde(alias = "requirements.conditions")]
    conditions: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct DebuffDataCsv {
    #[serde(rename = "debuff.description")]
    description: String,
    #[serde(rename = "debuff.multiplier")]
    multiplier: f32,
    #[serde(rename = "debuff.tickDuration")]
    tick_duration: u8,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct Trigger {
    skill: Skill,
    probability: f32,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct Proficiency {
    description: String,
    #[serde(rename = "damageMultiplier")]
    damage_multiplier: f32,
    #[serde(rename = "cooldownFactors")]
    cooldown_factors: u8,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct SkillData {
    value: f32,
    explanation: String,
    unit: Option<Unit>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct DebuffData {
    description: String,
    multiplier: f32,
    #[serde(rename = "tickDuration")]
    tick_duration: u8,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct SkillNotNested {
    #[serde(rename = "abilityName")]
    ability_name: String,
    #[serde(rename = "type")]
    skill_type: SkillType,
    #[serde(rename = "shortDescription")]
    short_description: String,
    #[serde(rename = "extendedDescription")]
    extended_description: String,
    narrative: String,
    #[serde(rename = "cooldownSeconds")]
    cooldown_seconds: u8,
    #[serde(rename = "damageType")]
    damage_type: DamageType,
    #[serde(rename = "requiredSkill")]
    required_skill: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Skill {
    #[serde(flatten)]
    _not_nested_data: SkillNotNested,

    requirements: SkillRequirements,
    #[serde(rename = "baseDamageMultiplier")]
    base_damage_multiplier: SkillData,
    #[serde(rename = "immediateDamagePerUse")]
    immediate_damage_per_use: SkillData,
    #[serde(rename = "effectRange")]
    effect_range: SkillData,
    #[serde(rename = "areaDamageArc")]
    area_damage_arc: SkillData,
    #[serde(rename = "proficiencyLevels")]
    proficiency_levels: HashMap<ProficiencyLevel, Proficiency>,
    debuffs: HashMap<Debuff, DebuffData>,
    #[serde(rename = "requiredReagents")]
    required_reagents: Vec<Reagents>,
    aspects: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct SkillCsv {
    #[serde(flatten)]
    _not_nested_data: SkillNotNested,
    #[serde(rename = "requiredReagents", serialize_with = "serialize_reagents")]
    required_reagents: Vec<Reagents>,
    #[serde(serialize_with = "serialize_aspects")]
    aspects: Vec<String>,
}

impl std::borrow::Borrow<str> for Reagents {
    fn borrow(&self) -> &str {
        match self {
            Reagents::SparklingPowder => "sparkling_powder",
            Reagents::Blood => "blood",
        }
    }
}

impl std::fmt::Display for Reagents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reagents::SparklingPowder => write!(f, "sparkling_powder"),
            Reagents::Blood => write!(f, "blood"),
        }
    }
}

fn serialize_reagents<S>(vec: &[Reagents], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&vec.join("|"))
}

fn serialize_aspects<S>(vec: &[String], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&vec.join("|"))
}

pub fn deserialize_json() -> Result<(Skill, PathBuf)> {
    event!(Level::INFO, "Deserializing JSON");
    let Some(path) = FileDialog::new()
        .add_filter("text/json", &["json"])
        .pick_file()
    else {
        return Err(anyhow!("No file selected."));
    };
    let file = std::fs::read_to_string(path.clone())?;
    event!(Level::DEBUG, "{file}");

    let file: Skill = serde_json::from_str(&file.trim())?;
    event!(Level::DEBUG, "{file:#?}");

    Ok((file, path))
}

pub fn serialize_csv_to_file(skill: Skill, path: &mut PathBuf) -> Result<()> {
    event!(Level::INFO, "Serializing CSV");
    path.set_extension("csv");
    let mut writer = WriterBuilder::new().from_writer(vec![]);

    let csv = SkillCsv::from(skill);
    writer.serialize(csv._not_nested_data)?;
    // writer.write_record(["requiredReagents", "aspects"])?;
    let not_nested = String::from_utf8(writer.into_inner()?)?;

    // event!(Level::INFO, "{not_nested}");

    let reagents = serialize_csv_vector(&csv.required_reagents)?;
    println!("reagents : {}", reagents);
    let aspects = serialize_csv_vector(&csv.aspects)?;
    println!("as : {}", aspects);

    let result = [not_nested, reagents, aspects].into_iter().fold(
        (vec![], vec![]),
        |(mut header, mut row), str| {
            let split: Vec<&str> = str.split("\n").collect();
            header.push(split[0].to_string());
            row.push(split[1].to_string());
            (header, row)
        },
    );
    event!(Level::INFO, "{:#?}", result);
    println!("{:#?}", result);

    Ok(())
}

fn serialize_csv_vector<T>(data: &[T]) -> Result<Vec<T>>
where
    T: serde::Serialize + std::fmt::Display,
{
    let mut writer = WriterBuilder::new().has_headers(false).from_writer(vec![]);
    writer.serialize(data)?;
    writer.flush();

    Ok(writer.into_inner()?)
}

impl From<Skill> for SkillCsv {
    fn from(skill: Skill) -> Self {
        Self {
            _not_nested_data: skill._not_nested_data,
            required_reagents: skill.required_reagents,
            aspects: skill.aspects,
        }
    }
}

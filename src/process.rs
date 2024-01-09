use anyhow::{anyhow, Result};
use csv::WriterBuilder;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

pub fn to_json() -> Result<String> {
    use flatten_json_object::{ArrayFormatting, Flattener};
    use json_objects_to_csv::Json2Csv;

    event!(Level::DEBUG, "Reading file");
    let Some(mut path) = FileDialog::new()
        .add_filter("text/json", &["json"])
        .pick_file()
    else {
        return Err(anyhow!("No file selected."));
    };
    event!(Level::INFO, "Reading from {:#?}.", path);
    let file = std::fs::read_to_string(path.clone())?;
    event!(Level::DEBUG, "{file:#?}");

    let flattener = Flattener::new()
        .set_key_separator(".")
        .set_array_formatting(ArrayFormatting::Plain)
        .set_preserve_empty_arrays(true)
        .set_preserve_empty_objects(true);
    let mut output = vec![];
    let writer = WriterBuilder::new().from_writer(&mut output);
    Json2Csv::new(flattener).convert_from_reader(file.as_bytes(), writer)?;
    let output = std::str::from_utf8(&output)?;

    event!(Level::DEBUG, "{output:#?}");

    path.set_extension("csv");
    std::fs::write(path.clone(), output)?;
    Ok(path.to_str().unwrap_or_default().to_string())
}

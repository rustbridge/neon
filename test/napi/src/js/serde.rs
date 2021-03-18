// Pokedex example from https://app.quicktype.io/

use neon::prelude::*;
use serde::{Deserialize, Serialize};

/// A collection of pokémon
#[derive(Serialize, Deserialize)]
pub struct PokedexSchema {
    /// All pokémon contained in the pokédex
    pokemon: Vec<Pokemon>,
}

/// A 'pocket monster.' One must catch them all.
#[derive(Serialize, Deserialize)]
pub struct Pokemon {
    avg_spawns: f64,
    /// The flavor of candy preferred by this pokémon
    candy: String,
    candy_count: Option<i64>,
    egg: Egg,
    height: String,
    /// A unique identifier for this pokémon.
    /// Higher ids generally imply rarer and more evolved pokémon.
    id: i64,
    /// Photographic evidence of this pokémon's existence
    img: String,
    multipliers: Option<Vec<f64>>,
    name: String,
    next_evolution: Option<Vec<Evolution>>,
    num: String,
    prev_evolution: Option<Vec<Evolution>>,
    spawn_chance: f64,
    spawn_time: String,
    #[serde(rename = "type")]
    pokemon_type: Vec<Type>,
    /// Types of pokémon that cause extra damage to this pokémon
    weaknesses: Vec<Type>,
    weight: String,
}

/// A description of an evolutionary stage of a pokémon
#[derive(Serialize, Deserialize)]
pub struct Evolution {
    /// The name of the Pokémon to or from which the containing Pokémon evolves
    name: String,
    /// The number of the pokémon to or from which the containing pokémon evolves
    num: String,
}

#[derive(Serialize, Deserialize)]
pub enum Egg {
    #[serde(rename = "Not in Eggs")]
    NotInEggs,
    #[serde(rename = "Omanyte Candy")]
    OmanyteCandy,
    #[serde(rename = "10 km")]
    The10Km,
    #[serde(rename = "2 km")]
    The2Km,
    #[serde(rename = "5 km")]
    The5Km,
}

#[derive(Serialize, Deserialize)]
pub enum Type {
    Bug,
    Dark,
    Dragon,
    Electric,
    Fairy,
    Fighting,
    Fire,
    Flying,
    Ghost,
    Grass,
    Ground,
    Ice,
    Normal,
    Poison,
    Psychic,
    Rock,
    Steel,
    Water,
}

pub fn parse_pokedex(mut cx: FunctionContext) -> JsResult<JsValue> {
    let pokedex = cx.argument::<JsString>(0)?.value(&mut cx);
    let pokedex: PokedexSchema =
        serde_json::from_str(&pokedex).or_else(|err| cx.throw_error(err.to_string()))?;

    cx.to_js_value(&pokedex).or_throw(&mut cx)
}

pub fn stringify_pokedex(mut cx: FunctionContext) -> JsResult<JsString> {
    let pokedex = cx.argument::<JsObject>(0)?;
    let pokedex: PokedexSchema = cx.from_js_value(pokedex).or_throw(&mut cx)?;
    let s = serde_json::to_string(&pokedex).or_else(|err| cx.throw_error(err.to_string()))?;

    Ok(cx.string(s))
}
